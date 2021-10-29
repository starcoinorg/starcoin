// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use itertools::Itertools;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_move_compiler::check_compiled_module_compat;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::transaction::{
    parse_transaction_argument, ScriptFunction, TransactionArgument,
};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{
    language_storage::ModuleId,
    transaction::{Module, Package},
};
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};
use stdlib::{
    build_script_abis, build_stdlib, build_stdlib_doc, build_stdlib_error_code_map,
    load_compiled_modules, load_latest_stable_compiled_modules, save_binary, COMPILED_EXTENSION,
    COMPILED_OUTPUT_PATH, COMPILED_SCRIPTS_ABI_DIR, LATEST_COMPILED_OUTPUT_PATH, STDLIB_DIR_NAME,
    STD_LIB_DOC_DIR,
};

fn compiled_modules(stdlib_path: &mut PathBuf) -> BTreeMap<ModuleId, CompiledModule> {
    let mut compiled_modules = BTreeMap::new();
    stdlib_path.push(STDLIB_DIR_NAME);
    for f in stdlib::iterate_directory(stdlib_path) {
        let mut bytes = Vec::new();
        File::open(f)
            .expect("Failed to open module bytecode file")
            .read_to_end(&mut bytes)
            .expect("Failed to read module bytecode file");
        let m = CompiledModule::deserialize(&bytes).expect("Failed to deserialize module bytecode");
        compiled_modules.insert(m.self_id(), m);
    }
    compiled_modules
}
//TODO refactor this with module diff.
fn incremental_update_with_version(
    pre_dir: &mut PathBuf,
    dest_dir: PathBuf,
    sub_dir: String,
    new_modules: &BTreeMap<String, CompiledModule>,
    init_script: Option<ScriptFunction>,
) {
    if pre_dir.exists() {
        let pre_compiled_modules = compiled_modules(pre_dir);
        let mut update_modules: BTreeMap<String, CompiledModule> = BTreeMap::new();
        for (key, module) in new_modules {
            let module_id = module.self_id();
            let is_new = if let Some(old_module) = pre_compiled_modules.get(&module_id) {
                old_module != module
            } else {
                true
            };

            if is_new {
                update_modules.insert(key.clone(), module.clone());
            }
        }

        let mut base_path = dest_dir;
        base_path.push(sub_dir);

        println!(
            "update modules : {} write to path : {:?}, pre version path : {:?}",
            update_modules.len(),
            base_path,
            pre_dir
        );
        if base_path.exists() {
            std::fs::remove_dir_all(&base_path).unwrap();
        }
        std::fs::create_dir_all(&base_path).unwrap();
        if !update_modules.is_empty() {
            let mut std_path = base_path.clone();
            std_path.push(STDLIB_DIR_NAME);
            std::fs::create_dir_all(&std_path).unwrap();
            let mut modules = Vec::new();
            for (name, module) in update_modules {
                let mut bytes = Vec::new();
                module.serialize(&mut bytes).unwrap();
                std_path.push(name);
                std_path.set_extension(COMPILED_EXTENSION);
                save_binary(std_path.as_path(), &bytes);
                modules.push(Module::new(bytes));
                std_path.pop();
            }
            let mut package = Package::new_with_modules(modules).unwrap();
            if let Some(script_function) = init_script {
                package.set_init_script(script_function);
            }

            let package_hash = package.crypto_hash();
            let mut package_path = base_path;
            package_path.push("stdlib");
            package_path.set_extension("blob");
            let blob = bcs_ext::to_bytes(&package).unwrap();
            save_binary(package_path.as_path(), &blob);
            println!("new package hash : {:?}", package_hash);
        }
    }
}

fn full_update_with_version(version_number: u64) -> PathBuf {
    let options = fs_extra::dir::CopyOptions::new();

    let mut stdlib_src = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    stdlib_src.push(STDLIB_DIR_NAME);

    // into version dir.
    let mut dest = PathBuf::from(COMPILED_OUTPUT_PATH);
    dest.push(format!("{}", version_number));

    // create if not exists.
    {
        if !dest.exists() {
            std::fs::create_dir_all(&dest).unwrap();
        }
    }

    {
        dest.push(STDLIB_DIR_NAME);
        if dest.exists() {
            std::fs::remove_dir_all(&dest).unwrap();
        }
        dest.pop();
    }
    fs_extra::dir::copy(stdlib_src, &dest, &options).unwrap();
    dest
}

fn replace_stdlib_by_path(
    module_path: &mut PathBuf,
    new_modules: BTreeMap<String, CompiledModule>,
) {
    if module_path.exists() {
        std::fs::remove_dir_all(&module_path).unwrap();
    }
    std::fs::create_dir_all(&module_path).unwrap();
    for (name, module) in new_modules {
        let mut bytes = Vec::new();
        module.serialize(&mut bytes).unwrap();
        module_path.push(name);
        module_path.set_extension(COMPILED_EXTENSION);
        save_binary(module_path.as_path(), &bytes);
        module_path.pop();
    }

    // Generate documentation
    std::fs::remove_dir_all(&STD_LIB_DOC_DIR).unwrap_or(());
    std::fs::create_dir_all(&STD_LIB_DOC_DIR).unwrap();
    build_stdlib_doc();

    // Generate script ABIs
    std::fs::remove_dir_all(&COMPILED_SCRIPTS_ABI_DIR).unwrap_or(());
    std::fs::create_dir_all(&COMPILED_SCRIPTS_ABI_DIR).unwrap();
    build_script_abis();

    build_stdlib_error_code_map();
}

// Generates the compiled stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    // pass argument 'version' to generate new release
    // for example, "cargo run -- --version 1"
    let cli = App::new("stdlib")
        .name("Move standard library")
        .author("The Starcoin Core Contributors")
        .after_help("this command can be used to generate an incremental package, with init script included.")
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .takes_value(true)
                .value_name("VERSION")
                .help("version number for compiled stdlib, for example 1. don't forget to record the release note"),
        )
        .arg(
            Arg::with_name("pre-version")
                .short("p")
                .long("pre-version")
                .takes_value(true)
                .value_name("PRE_VERSION")
                .help("pre version of stdlib to generate diff and check compatibility with"))
        .arg(
            Arg::with_name("no-check-compatibility")
                .short("n")
                .long("no-check-compatibility")
                .help("don't check compatibility between the old and new standard library"),
        )
        .arg(
        Arg::with_name("init-script-module")
            .short("m")
            .long("init-script-module")
            .takes_value(true)
            .value_name("MODULE")
            .help("module name of init script function"),
        ).arg(
        Arg::with_name("init-script-function")
            .short("f")
            .long("init-script-function")
            .takes_value(true)
            .value_name("FUNC")
            .help("function name of init script function"),
        ).arg(
        Arg::with_name("init-script-type-args")
            .short("t")
            .long("init-script-type-args")
            .multiple(true)
            .takes_value(true)
            .value_name("TYPE_ARGS")
            .help("type args of init script function"),
        ).arg(
        Arg::with_name("init-script-args")
            .short("a")
            .long("init-script-args")
            .multiple(true)
            .takes_value(true)
            .value_name("ARGS")
            .help("args of init script function"),
        );

    let matches = cli.get_matches();
    let mut generate_new_version = false;
    let mut version_number: u64 = 0;
    if matches.is_present("version") {
        generate_new_version = true;
        version_number = matches.value_of("version").unwrap().parse::<u64>().unwrap();
    }

    let pre_version = if version_number > 0 {
        Some(if matches.is_present("pre-version") {
            matches
                .value_of("pre-version")
                .unwrap()
                .parse::<u64>()
                .unwrap()
        } else {
            version_number - 1
        })
    } else {
        None
    };

    let no_check_compatibility = matches.is_present("no-check-compatibility");
    let has_init_script =
        matches.is_present("init-script-module") && matches.is_present("init-script-function");
    let init_script = if has_init_script {
        let module_name = matches.value_of("init-script-module").unwrap();
        let function_name = matches.value_of("init-script-function").unwrap();
        let type_args = if matches.is_present("init-script-type-args") {
            let type_args_str: Vec<&str> = matches
                .values_of("init-script-type-args")
                .unwrap()
                .collect();
            let type_args: Vec<TypeTag> = type_args_str
                .iter()
                .map(|s| parse_type_tag(*s).unwrap())
                .collect();
            type_args
        } else {
            vec![]
        };
        let args = if matches.is_present("init-script-args") {
            let args_strings: Vec<&str> = matches.values_of("init-script-args").unwrap().collect();
            let args: Vec<TransactionArgument> = args_strings
                .iter()
                .map(|s| parse_transaction_argument(*s).unwrap())
                .collect();
            args
        } else {
            vec![]
        };
        println!("type_args {:?}", type_args);
        println!("args {:?}", args);

        let init_script = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new(module_name).unwrap()),
            Identifier::new(function_name).unwrap(),
            type_args,
            convert_txn_args(&args),
        );
        Some(init_script)
    } else {
        None
    };

    // Make sure that the current directory is `vm/stdlib` from now on.
    let exec_path = std::env::args().next().expect("path of the executable");
    let base_path = std::path::Path::new(&exec_path)
        .parent()
        .unwrap()
        .join("../../vm/stdlib");
    std::env::set_current_dir(&base_path).expect("failed to change directory");

    // Write the stdlib blob
    let mut module_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    module_path.push(STDLIB_DIR_NAME);
    let new_modules = build_stdlib();

    if !no_check_compatibility {
        if let Some((pre_stable_version, pre_stable_modules)) = pre_version
            .map(StdlibVersion::Version)
            .map(|v| (v, load_compiled_modules(v)))
            .or_else(load_latest_stable_compiled_modules)
        {
            println!(
                "Check compat with pre stable version: {}",
                pre_stable_version
            );
            let pre_stable_modules = pre_stable_modules
                .into_iter()
                .map(|module| (module.self_id(), module))
                .collect::<BTreeMap<_, _>>();
            let incompatible_module_ids = new_modules
                .values()
                .into_iter()
                .filter_map(|module| {
                    let module_id = module.self_id();
                    if let Some(old_module) = pre_stable_modules.get(&module_id) {
                        let compatibility = check_compiled_module_compat(old_module, module);
                        if !compatibility {
                            Some(module_id)
                        } else {
                            None
                        }
                    } else {
                        println!("Module {:?} is new module.", module_id);
                        None
                    }
                })
                .collect::<Vec<_>>();
            if !incompatible_module_ids.is_empty() {
                eprintln!(
                    "Modules {} is incompatible with version: {}!",
                    incompatible_module_ids
                        .into_iter()
                        .map(|module_id| module_id.to_string())
                        .join(","),
                    pre_stable_version
                );
                std::process::exit(1);
            }
        }
    }

    replace_stdlib_by_path(&mut module_path, new_modules.clone());

    if generate_new_version {
        let dest_dir = full_update_with_version(version_number);
        if let Some(pre_version) = pre_version {
            let mut pre_version_dir = PathBuf::from(COMPILED_OUTPUT_PATH);
            pre_version_dir.push(format!("{}", pre_version));
            let sub_dir = format!("{}-{}", pre_version, version_number);
            incremental_update_with_version(
                &mut pre_version_dir,
                dest_dir,
                sub_dir,
                &new_modules,
                init_script,
            );
        }
    }
}
