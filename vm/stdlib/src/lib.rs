// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use include_dir::{include_dir, Dir};
use log::LevelFilter;
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use starcoin_config::ChainNetwork;
pub use starcoin_config::StdlibVersion;
use starcoin_move_compiler::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use starcoin_vm_types::bytecode_verifier::{verify_module, DependencyChecker};
use starcoin_vm_types::file_format::CompiledModule;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub mod init_scripts;
pub mod transaction_scripts;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

pub const NO_USE_COMPILED: &str = "MOVE_NO_USE_COMPILED";

pub const TRANSACTION_SCRIPTS: &str = "transaction_scripts";

pub const INIT_SCRIPTS: &str = "init_scripts";
/// The output path under which compiled files will be put
pub const COMPILED_OUTPUT_PATH: &str = "compiled";
/// The latest output path under which compiled files will be put
pub const LATEST_COMPILED_OUTPUT_PATH: &str = "compiled/latest";
/// The output path for the compiled stdlib
pub const STDLIB_DIR_NAME: &str = "stdlib";
/// The extension for compiled files
pub const COMPILED_EXTENSION: &str = "mv";

/// The output path for stdlib documentation.
pub const STD_LIB_DOC_DIR: &str = "modules/doc";
/// The output path for transaction script documentation.
pub const TRANSACTION_SCRIPTS_DOC_DIR: &str = "transaction_scripts/doc";
pub const COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/latest/transaction_scripts/abi";

pub const ERROR_DESC_DIR: &str = "error_descriptions";
pub const ERROR_DESC_FILENAME: &str = "error_descriptions";
pub const ERROR_DESC_EXTENSION: &str = "errmap";
pub const ERROR_DESCRIPTIONS: &[u8] =
    std::include_bytes!("../compiled/latest/error_descriptions/error_descriptions.errmap");

// The current stdlib that is freshly built. This will never be used in deployment so we don't need
// to pull the same trick here in order to include this in the Rust binary.
static FRESH_MOVELANG_STDLIB: Lazy<Vec<CompiledModule>> =
    Lazy::new(|| build_stdlib().values().cloned().collect());

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The compiled library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
pub const COMPILED_MOVE_CODE_DIR: Dir = include_dir!("compiled");

const COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/latest/transaction_scripts";
pub const LATEST_VERSION: &str = "latest";

static CHAIN_NETWORK_STDLIB_VERSIONS: Lazy<Vec<StdlibVersion>> = Lazy::new(|| {
    vec![
        ChainNetwork::DEV.stdlib_version(),
        ChainNetwork::HALLEY.stdlib_version(),
        ChainNetwork::PROXIMA.stdlib_version(),
        ChainNetwork::MAIN.stdlib_version(),
    ]
});

static COMPILED_MOVELANG_STDLIB: Lazy<HashMap<(StdlibVersion, StdlibType), Vec<CompiledModule>>> =
    Lazy::new(|| {
        let mut map = HashMap::new();
        for version in &*CHAIN_NETWORK_STDLIB_VERSIONS {
            let sub_dir = format!("{}/{}", version.as_string(), STDLIB_DIR_NAME);
            let mut modules: Vec<(String, CompiledModule)> = COMPILED_MOVE_CODE_DIR
                .get_dir(Path::new(sub_dir.as_str()))
                .unwrap()
                .files()
                .iter()
                .map(|file| {
                    (
                        file.path().to_str().unwrap().to_string(),
                        CompiledModule::deserialize(&file.contents()).unwrap(),
                    )
                })
                .collect();

            // We need to verify modules based on their dependency order.
            modules.sort_by_key(|(module_name, _)| module_name.clone());

            let mut verified_modules = vec![];
            for (_, module) in modules.into_iter() {
                verify_module(&module).expect("stdlib module failed to verify");
                DependencyChecker::verify_module(&module, &verified_modules)
                    .expect("stdlib module dependency failed to verify");
                verified_modules.push(module)
            }
            map.insert((*version, StdlibType::Stdlib), verified_modules);
        }
        map
    });

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum StdlibType {
    Stdlib,
    InitScripts,
    TransactionScripts,
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
pub fn stdlib_modules(option: StdLibOptions) -> &'static [CompiledModule] {
    match option {
        StdLibOptions::Fresh => &*FRESH_MOVELANG_STDLIB,
        StdLibOptions::Compiled(version) => &*COMPILED_MOVELANG_STDLIB
            .get(&(version, StdlibType::Stdlib))
            .expect("compiled modules should not be none"),
    }
}

pub fn filter_compiled_mv_files(
    dir_iter: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = String> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == COMPILED_EXTENSION {
            path.into_os_string().into_string().ok()
        } else {
            None
        }
    })
}

pub fn compiled_stdlib_files(path: &Path) -> Vec<String> {
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_compiled_mv_files(dirfiles).collect::<Vec<_>>()
}

/// A predicate detailing whether the compiled versions of scripts and the stdlib should be used or
/// not. The default is that the compiled versions of the stdlib and transaction scripts should be
/// used.
pub fn use_compiled() -> bool {
    std::env::var(NO_USE_COMPILED).is_err()
}

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = String> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == MOVE_EXTENSION {
            path.into_os_string().into_string().ok()
        } else {
            None
        }
    })
}

pub fn stdlib_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles).collect::<Vec<_>>()
}

pub fn transaction_script_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(TRANSACTION_SCRIPTS);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles).collect::<Vec<_>>()
}

pub fn init_script_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(INIT_SCRIPTS);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles).collect::<Vec<_>>()
}

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE), None).unwrap();
    let mut modules = BTreeMap::new();
    for (i, compiled_unit) in compiled_units.into_iter().enumerate() {
        let name = compiled_unit.name();
        match compiled_unit {
            CompiledUnit::Module { module, .. } => {
                verify_module(&module).expect("stdlib module failed to verify");
                DependencyChecker::verify_module(&module, modules.values())
                    .expect("stdlib module dependency failed to verify");
                // Tag each module with its index in the module dependency order. Needed for
                // when they are deserialized and verified later on.
                modules.insert(format!("{:02}_{}", i, name), module);
            }
            CompiledUnit::Script { .. } => panic!("Unexpected Script in stdlib"),
        }
    }
    modules
}

pub fn compile_script(source_file_str: String) -> Vec<u8> {
    let (_, mut compiled_program) = move_compile(
        &[source_file_str],
        &stdlib_files(),
        Some(Address::LIBRA_CORE),
        None,
    )
    .unwrap();
    let mut script_bytes = vec![];
    assert_eq!(compiled_program.len(), 1);
    match compiled_program.pop().unwrap() {
        CompiledUnit::Module { .. } => panic!("Unexpected module when compiling script"),
        CompiledUnit::Script { script, .. } => script.serialize(&mut script_bytes).unwrap(),
    };
    script_bytes
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

pub fn build_transaction_script_abi() {
    for txn_script_file in transaction_script_files() {
        build_abi(
            COMPILED_TRANSACTION_SCRIPTS_ABI_DIR,
            &[txn_script_file],
            STD_LIB_DIR,
            COMPILED_TRANSACTION_SCRIPTS_DIR,
        )
    }
}

fn build_abi(output_path: &str, sources: &[String], dep_path: &str, compiled_script_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
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

pub fn build_transaction_script_doc() {
    for txn_script_file in transaction_script_files() {
        build_doc(
            TRANSACTION_SCRIPTS_DOC_DIR,
            STD_LIB_DOC_DIR,
            &[txn_script_file],
            STD_LIB_DIR,
        )
    }
    for init_script_file in init_script_files() {
        build_doc(
            TRANSACTION_SCRIPTS_DOC_DIR,
            STD_LIB_DOC_DIR,
            &[init_script_file],
            STD_LIB_DIR,
        )
    }
}

fn build_doc(output_path: &str, doc_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
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

fn build_error_code_map(output_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_errmapgen = true;
    options.errmapgen.output_file = output_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}
