// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use once_cell::sync::Lazy;
use starcoin_move_compiler::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use starcoin_vm_types::bytecode_verifier::{verify_module, DependencyChecker};
use starcoin_vm_types::file_format::CompiledModule;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use include_dir::{include_dir, Dir};

pub mod init_scripts;
pub mod transaction_scripts;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

pub const NO_USE_STAGED: &str = "MOVE_NO_USE_STAGED";

pub const TRANSACTION_SCRIPTS: &str = "transaction_scripts";

pub const INIT_SCRIPTS: &str = "init_scripts";
/// The output path under which staged files will be put
pub const STAGED_OUTPUT_PATH: &str = "staged";
/// The output path for the staged stdlib
pub const STAGED_STDLIB_PATH: &str = "stdlib";
/// The extension for staged files
pub const STAGED_EXTENSION: &str = "mv";

// The current stdlib that is freshly built. This will never be used in deployment so we don't need
// to pull the same trick here in order to include this in the Rust binary.
static FRESH_MOVELANG_STDLIB: Lazy<Vec<CompiledModule>> =
    Lazy::new(|| build_stdlib().values().cloned().collect());

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The compiled library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
const COMPILED_STDLIB_DIR: Dir = include_dir!("staged/stdlib");

// The staged version of the move standard library.
// Similarly to genesis, we keep a compiled version of the standard library and scripts around, and
// only periodically update these. This has the effect of decoupling the current leading edge of
// compiler development from the current stdlib used in genesis/scripts.  In particular, changes in
// the compiler will not affect the script hashes or stdlib until we have tested the changes to our
// satisfaction. Then we can generate a new staged version of the stdlib/scripts (and will need to
// regenerate genesis). The staged version of the stdlib/scripts are used unless otherwise
// specified either by the MOVE_NO_USE_STAGED env var, or by passing the "StdLibOptions::Fresh"
// option to `stdlib_modules`.
static STAGED_MOVELANG_STDLIB: Lazy<Vec<CompiledModule>> = Lazy::new(|| {
    let mut modules: Vec<(String, CompiledModule)> = COMPILED_STDLIB_DIR
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
    modules.sort_by_key(|(path, _)| {
        let splits: Vec<_> = path.split('_').collect();
        assert!(splits.len() == 2, "Invalid module name encountered");
        splits[0].parse::<u64>().unwrap()
    });

    let mut verified_modules = vec![];
    for (_, module) in modules.into_iter() {
        verify_module(&module).expect("stdlib module failed to verify");
        DependencyChecker::verify_module(&module, &verified_modules)
            .expect("stdlib module dependency failed to verify");
        verified_modules.push(module)
    }
    verified_modules
});

/// An enum specifying whether the staged stdlib/scripts should be used or freshly built versions
/// should be used.
#[derive(Debug, Eq, PartialEq)]
pub enum StdLibOptions {
    Staged,
    Fresh,
}

/// Returns a reference to the standard library. Depending upon the `option` flag passed in
/// either a staged version of the standard library will be returned or a new freshly built stdlib
/// will be used.
pub fn stdlib_modules(option: StdLibOptions) -> &'static [CompiledModule] {
    match option {
        StdLibOptions::Staged => &*STAGED_MOVELANG_STDLIB,
        StdLibOptions::Fresh => &*FRESH_MOVELANG_STDLIB,
    }
}

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
/// The defualt is to return a staged version of the stdlib unless it is otherwise specified by the
/// `MOVE_NO_USE_STAGED` environment variable.
pub fn env_stdlib_modules() -> &'static [CompiledModule] {
    let option = if use_staged() {
        StdLibOptions::Staged
    } else {
        StdLibOptions::Fresh
    };
    stdlib_modules(option)
}

/// A predicate detailing whether the staged versions of scripts and the stdlib should be used or
/// not. The default is that the staged versions of the stdlib and transaction scripts should be
/// used.
pub fn use_staged() -> bool {
    std::env::var(NO_USE_STAGED).is_err()
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

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE)).unwrap();
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
                modules.insert(format!("{}_{}", i, name), module);
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
