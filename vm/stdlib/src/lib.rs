// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::{bail, ensure, format_err, Result};
use include_dir::{include_dir, Dir};
use log::{info, LevelFilter};
use move_bytecode_verifier::{dependencies, verify_module};
use move_compiler::command_line::compiler::construct_pre_compiled_lib_from_compiler;
use move_compiler::FullyCompiledProgram;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_move_compiler::compiled_unit::{CompiledUnit, NamedCompiledModule};
use starcoin_move_compiler::diagnostics::{
    report_diagnostics_to_color_buffer, unwrap_or_report_diagnostics,
};
use starcoin_move_compiler::shared::Flags;
pub use starcoin_move_compiler::{starcoin_framework_named_addresses, Compiler};
use starcoin_vm_types::file_format::CompiledModule;
pub use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::transaction::{Module, Package, ScriptFunction};
use std::str::FromStr;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

mod compat;
pub use compat::*;
pub use starcoin_move_compiler::utils::iterate_directory;

pub const STD_LIB_DIR: &str = "sources";

pub const NO_USE_COMPILED: &str = "MOVE_NO_USE_COMPILED";

/// The output path under which compiled files will be put
pub const COMPILED_OUTPUT_PATH: &str = "compiled";
/// The latest output path under which compiled files will be put
pub const LATEST_COMPILED_OUTPUT_PATH: &str = "compiled/latest";
/// The output path for the compiled stdlib
pub const STDLIB_DIR_NAME: &str = "stdlib";
/// The extension for compiled files
pub const COMPILED_EXTENSION: &str =
    starcoin_move_compiler::move_command_line_common::files::MOVE_COMPILED_EXTENSION;

/// The output path for stdlib documentation.
pub const STD_LIB_DOC_DIR: &str = "compiled/latest/doc";
pub const COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/latest/transaction_scripts/abi";
// use same dir as scripts abi
pub const COMPILED_SCRIPTS_ABI_DIR: &str = "compiled/latest/transaction_scripts/abi";

pub const ERROR_DESC_DIR: &str = "error_descriptions";
pub const ERROR_DESC_FILENAME: &str = "error_descriptions";
pub const ERROR_DESC_EXTENSION: &str = "errmap";
pub const ERROR_DESCRIPTIONS: &[u8] =
    std::include_bytes!("../compiled/latest/error_descriptions/error_descriptions.errmap");

pub const STDLIB_DIR: Dir = include_dir!("sources");

// The current stdlib that is freshly built. This will never be used in deployment so we don't need
// to pull the same trick here in order to include this in the Rust binary.
static FRESH_MOVELANG_STDLIB: Lazy<Vec<Vec<u8>>> = Lazy::new(|| {
    build_stdlib()
        .values()
        .map(|m| {
            let mut blob = vec![];
            m.serialize(&mut blob).unwrap();
            blob
        })
        .collect()
});

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The compiled library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
pub const COMPILED_MOVE_CODE_DIR: Dir = include_dir!("compiled");

const COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/latest/transaction_scripts";
pub const LATEST_VERSION: &str = "latest";

pub static STDLIB_VERSIONS: Lazy<Vec<StdlibVersion>> = Lazy::new(|| {
    let mut versions = COMPILED_MOVE_CODE_DIR
        .dirs()
        .iter()
        .map(|dir| {
            StdlibVersion::from_str(dir.path().file_name().unwrap().to_str().unwrap()).unwrap()
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions
});

static COMPILED_STDLIB: Lazy<HashMap<StdlibVersion, Vec<Vec<u8>>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for version in &*STDLIB_VERSIONS {
        let modules = read_compiled_modules(*version);
        verify_compiled_modules(&modules);
        map.insert(*version, modules);
    }
    map
});

pub const SCRIPT_HASH_LENGTH: usize = HashValue::LENGTH;

pub static PRECOMPILED_STARCOIN_FRAMEWORK: Lazy<FullyCompiledProgram> = Lazy::new(|| {
    let sources = stdlib_files();
    let compiler = Compiler::new(&sources, &[])
        .set_flags(Flags::empty().set_sources_shadow_deps(false))
        .set_named_address_values(starcoin_framework_named_addresses());
    let program_res = construct_pre_compiled_lib_from_compiler(compiler).unwrap();
    match program_res {
        Ok(df) => {
            let compiled = df.compiled;
            {
                let compiler = Compiler::new(&[], &sources)
                    .set_flags(Flags::empty().set_sources_shadow_deps(false))
                    .set_named_address_values(starcoin_framework_named_addresses());
                let mut program_as_lib = construct_pre_compiled_lib_from_compiler(compiler)
                    .unwrap()
                    .unwrap();
                program_as_lib.compiled = compiled;
                program_as_lib
            }
        }
        Err((files, errors)) => {
            eprintln!("!!!Starcoin Framework failed to compile!!!");
            move_compiler::diagnostics::report_diagnostics(&files, errors)
        }
    }
});

/// Return all versions of stdlib, include latest.
pub fn stdlib_versions() -> Vec<StdlibVersion> {
    STDLIB_VERSIONS.clone()
}

/// Return the latest stable version of stdlib.
pub fn stdlib_latest_stable_version() -> Option<StdlibVersion> {
    STDLIB_VERSIONS
        .iter()
        .filter(|version| !version.is_latest())
        .last()
        .cloned()
}

/// An enum specifying whether the compiled stdlib/scripts should be used or freshly built versions
/// should be used.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum StdLibOptions {
    Compiled(StdlibVersion),
    Fresh,
}

/// Returns a reference to the standard library. Depending upon the `option` flag passed in
/// either a compiled version of the standard library will be returned or a new freshly built stdlib
/// will be used.
pub fn stdlib_modules(option: StdLibOptions) -> &'static [Vec<u8>] {
    match option {
        StdLibOptions::Fresh => &*FRESH_MOVELANG_STDLIB,
        StdLibOptions::Compiled(version) => &*COMPILED_STDLIB
            .get(&version)
            .unwrap_or_else(|| panic!("Stdlib version {:?} not exist.", version)),
    }
}

pub fn stdlib_package(
    stdlib_option: StdLibOptions,
    init_script: Option<ScriptFunction>,
) -> Result<Package> {
    let modules = stdlib_modules(stdlib_option);
    module_to_package(modules.to_vec(), init_script)
}

fn module_to_package(
    modules: Vec<Vec<u8>>,
    init_script: Option<ScriptFunction>,
) -> Result<Package> {
    Package::new(modules.into_iter().map(Module::new).collect(), init_script)
}

pub fn restore_stdlib_in_dir(dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut deps = vec![];
    for dep in STDLIB_DIR.files() {
        let path = dir.join(dep.path());
        std::fs::write(path.as_path(), dep.contents())?;
        deps.push(path.display().to_string());
    }
    Ok(deps)
}

pub(crate) fn stdlib_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);

    let dirfiles = starcoin_move_compiler::utils::iterate_directory(&path);
    starcoin_move_compiler::utils::filter_move_files(dirfiles).collect::<Vec<_>>()
}

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let compiled_units = {
        let (files, units_res) = Compiler::new(&stdlib_files(), &[])
            .set_named_address_values(starcoin_framework_named_addresses())
            .build()
            .unwrap();
        let (units, warnings) = unwrap_or_report_diagnostics(&files, units_res);
        println!(
            "{}",
            String::from_utf8_lossy(&report_diagnostics_to_color_buffer(&files, warnings))
        );
        units
    };

    let mut modules = BTreeMap::new();
    for (i, compiled_unit) in compiled_units.into_iter().enumerate() {
        let compiled_unit = compiled_unit.into_compiled_unit();

        let name = compiled_unit.name();
        match compiled_unit {
            CompiledUnit::Module(NamedCompiledModule { module, .. }) => {
                verify_module(&module).expect("stdlib module failed to verify");
                dependencies::verify_module(&module, modules.values())
                    .expect("stdlib module dependency failed to verify");
                // Tag each module with its index in the module dependency order. Needed for
                // when they are deserialized and verified later on.
                modules.insert(format!("{:02}_{}", i, name), module);
            }
            CompiledUnit::Script(_) => panic!("Unexpected Script in stdlib"),
        }
    }
    modules
}

pub fn save_binary(path: &Path, binary: &[u8]) {
    if path.exists() {
        let mut bytes = vec![];
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();
        if Sha256::digest(binary) == Sha256::digest(&bytes) {
            return;
        }
    }

    File::create(path).unwrap().write_all(binary).unwrap();
}

pub fn build_stdlib_doc() {
    build_doc(STD_LIB_DOC_DIR, "", stdlib_files().as_slice(), "")
}

pub fn build_script_abis() {
    stdlib_files().par_iter().for_each(|file| {
        build_abi(
            COMPILED_SCRIPTS_ABI_DIR,
            &[file.clone()],
            STD_LIB_DIR,
            COMPILED_TRANSACTION_SCRIPTS_DIR,
        )
    });
}

#[allow(clippy::field_reassign_with_default)]
fn build_abi(output_path: &str, sources: &[String], dep_path: &str, compiled_script_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    options.move_named_address_values = starcoin_framework_named_addresses()
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_abigen = true;
    options.abigen.output_directory = output_path.to_string();
    options.abigen.compiled_script_directory = compiled_script_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

#[allow(clippy::field_reassign_with_default)]
fn build_doc(output_path: &str, doc_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.move_named_address_values = starcoin_framework_named_addresses()
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    options.verbosity_level = LevelFilter::Warn;
    options.run_docgen = true;
    options.docgen.include_impl = true;
    options.docgen.include_private_fun = true;
    options.docgen.specs_inlined = false;
    if !doc_path.is_empty() {
        options.docgen.doc_path = vec![doc_path.to_string()];
    }
    options.docgen.output_directory = output_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn build_stdlib_error_code_map() {
    let mut path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    path.push(ERROR_DESC_DIR);
    fs::create_dir_all(&path).unwrap();
    path.push(ERROR_DESC_FILENAME);
    path.set_extension(ERROR_DESC_EXTENSION);
    build_error_code_map(path.to_str().unwrap(), stdlib_files().as_slice(), "")
}

#[allow(clippy::field_reassign_with_default)]
fn build_error_code_map(output_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    options.move_named_address_values = starcoin_framework_named_addresses()
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_errmapgen = true;
    options.errmapgen.output_file = output_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn load_latest_stable_compiled_modules() -> Option<(StdlibVersion, Vec<CompiledModule>)> {
    stdlib_latest_stable_version().map(|version| (version, load_compiled_modules(version)))
}

pub fn load_latest_compiled_modules() -> Vec<CompiledModule> {
    load_compiled_modules(StdlibVersion::Latest)
}

/// read module blobs from dir.
pub fn read_compiled_modules(stdlib_version: StdlibVersion) -> Vec<Vec<u8>> {
    let sub_dir = format!("{}/{}", stdlib_version.as_string(), STDLIB_DIR_NAME);
    let mut modules: Vec<(String, _)> = COMPILED_MOVE_CODE_DIR
        .get_dir(Path::new(sub_dir.as_str()))
        .unwrap()
        .files()
        .iter()
        .map(|file| {
            (
                file.path().to_str().unwrap().to_string(),
                file.contents().to_vec(),
            )
        })
        .collect();
    // We need to verify modules based on their dependency order.
    modules.sort_by_key(|(module_name, _)| module_name.clone());
    modules.into_iter().map(|v| v.1).collect()
}

/// verify modules blob.
pub fn verify_compiled_modules(modules: &[Vec<u8>]) -> Vec<CompiledModule> {
    let mut verified_modules = vec![];
    for module in modules {
        let module = CompiledModule::deserialize(module).expect("module deserialize should be ok");
        verify_module(&module).expect("stdlib module failed to verify");
        dependencies::verify_module(&module, &verified_modules)
            .expect("stdlib module dependency failed to verify");
        verified_modules.push(module)
    }
    verified_modules
}

pub fn load_compiled_modules(stdlib_version: StdlibVersion) -> Vec<CompiledModule> {
    let modules = read_compiled_modules(stdlib_version);
    verify_compiled_modules(modules.as_slice())
}

pub fn modules_diff(
    first_modules: &[CompiledModule],
    second_modules: &[CompiledModule],
) -> Vec<CompiledModule> {
    let mut update_modules = vec![];
    let first_modules = first_modules
        .iter()
        .map(|module| (module.self_id(), module.clone()))
        .collect::<BTreeMap<_, _>>();
    for module in second_modules {
        let module_id = module.self_id();
        let is_new = if let Some(old_module) = first_modules.get(&module_id) {
            old_module != module
        } else {
            true
        };
        if is_new {
            update_modules.push(module.clone());
        }
    }
    update_modules
}

pub fn load_upgrade_package(
    current_version: StdlibVersion,
    new_version: StdlibVersion,
) -> Result<Option<Package>> {
    let package = match (current_version, new_version) {
        (StdlibVersion::Version(previous_version), StdlibVersion::Version(new_version)) => {
            ensure!(
                previous_version < new_version,
                "previous version should < new version"
            );

            let package_file = format!(
                "{}/{}-{}/stdlib.blob",
                new_version, previous_version, new_version
            );
            let package = COMPILED_MOVE_CODE_DIR
                .get_file(package_file)
                .map(|file| {
                    bcs_ext::from_bytes::<Package>(file.contents())
                        .expect("Decode package should success")
                })
                .ok_or_else(|| {
                    format_err!(
                        "Can not find upgrade package between version {} and {}",
                        current_version,
                        new_version
                    )
                })?;
            Some(package)
        }
        (current_version @ StdlibVersion::Version(_), StdlibVersion::Latest) => {
            let current_modules = load_compiled_modules(current_version);
            let latest_modules = load_latest_compiled_modules();
            let diff = modules_diff(&current_modules, &latest_modules);
            if diff.is_empty() {
                None
            } else {
                Some(module_to_package(
                    diff.into_iter()
                        .map(|m| {
                            let mut blob = vec![];
                            m.serialize(&mut blob).unwrap();
                            blob
                        })
                        .collect(),
                    None,
                )?)
            }
        }
        (StdlibVersion::Latest, _) => {
            bail!("Current version is latest, can not upgrade.");
        }
    };
    info!(
        "load_upgrade_package({:?},{:?}), hash: {:?}",
        current_version,
        new_version,
        package.as_ref().map(|package| package.crypto_hash())
    );
    Ok(package)
}
