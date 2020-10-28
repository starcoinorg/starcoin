// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use starcoin_move_compiler::check_compiled_module_compat;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::language_storage::ModuleId;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use stdlib::{
    build_stdlib, build_stdlib_doc, build_stdlib_error_code_map, build_transaction_script_abi,
    build_transaction_script_doc, compile_script, filter_move_files, save_binary,
    COMPILED_EXTENSION, COMPILED_OUTPUT_PATH, COMPILED_TRANSACTION_SCRIPTS_ABI_DIR, INIT_SCRIPTS,
    LATEST_COMPILED_OUTPUT_PATH, STDLIB_DIR_NAME, STD_LIB_DOC_DIR, TRANSACTION_SCRIPTS,
    TRANSACTION_SCRIPTS_DOC_DIR,
};

fn compile_scripts(script_dir: &Path) {
    let script_source_files = datatest_stable::utils::iterate_directory(script_dir);
    let script_files = filter_move_files(script_source_files);
    for script_file in script_files {
        let compiled_script = compile_script(script_file.clone());
        let mut output_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
        output_path.push(script_file.clone());
        output_path.set_extension(COMPILED_EXTENSION);
        File::create(output_path)
            .unwrap()
            .write_all(&compiled_script)
            .unwrap();
    }
}

fn latest_compiled_modules() -> BTreeMap<ModuleId, CompiledModule> {
    let mut module_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    compiled_modules(&mut module_path)
}

fn compiled_modules(stdlib_path: &mut PathBuf) -> BTreeMap<ModuleId, CompiledModule> {
    let mut compiled_modules = BTreeMap::new();
    stdlib_path.push(STDLIB_DIR_NAME);
    for f in datatest_stable::utils::iterate_directory(&stdlib_path) {
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

fn incremental_update_with_version(
    pre_dir: &mut PathBuf,
    dest_dir: PathBuf,
    sub_dir: String,
    new_modules: &BTreeMap<String, CompiledModule>,
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

        if !update_modules.is_empty() {
            let mut full_path = dest_dir;
            full_path.push(sub_dir);

            println!(
                "update modules : {} write to path : {:?}, pre version path : {:?}",
                update_modules.len(),
                full_path,
                pre_dir
            );
            if full_path.exists() {
                std::fs::remove_dir_all(&full_path).unwrap();
            }
            std::fs::create_dir_all(&full_path).unwrap();
            for (name, module) in update_modules {
                let mut bytes = Vec::new();
                module.serialize(&mut bytes).unwrap();
                full_path.push(name);
                full_path.set_extension(COMPILED_EXTENSION);
                save_binary(full_path.as_path(), &bytes);
                full_path.pop();
            }
        }
    }
}

fn full_update_with_version(version_number: &str) -> PathBuf {
    let options = fs_extra::dir::CopyOptions::new();

    let mut stdlib_src = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    stdlib_src.push(STDLIB_DIR_NAME);

    let mut init_scripts_src = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    init_scripts_src.push(INIT_SCRIPTS);

    let mut txn_scripts_src = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    txn_scripts_src.push(TRANSACTION_SCRIPTS);

    let mut dest = PathBuf::from(COMPILED_OUTPUT_PATH);
    dest.push(version_number);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).unwrap();
    }
    std::fs::create_dir_all(&dest).unwrap();
    fs_extra::dir::copy(stdlib_src, &dest, &options).unwrap();
    fs_extra::dir::copy(init_scripts_src, &dest, &options).unwrap();
    fs_extra::dir::copy(txn_scripts_src, &dest, &options).unwrap();
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

    compile_scripts(Path::new(INIT_SCRIPTS));
    compile_scripts(Path::new(TRANSACTION_SCRIPTS));

    // Generate documentation
    std::fs::remove_dir_all(&STD_LIB_DOC_DIR).unwrap_or(());
    std::fs::create_dir_all(&STD_LIB_DOC_DIR).unwrap();
    build_stdlib_doc();

    // Generate script ABIs
    std::fs::remove_dir_all(&COMPILED_TRANSACTION_SCRIPTS_ABI_DIR).unwrap_or(());
    std::fs::create_dir_all(&COMPILED_TRANSACTION_SCRIPTS_ABI_DIR).unwrap();
    build_transaction_script_abi();

    std::fs::remove_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap_or(());
    std::fs::create_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap();
    build_transaction_script_doc();

    build_stdlib_error_code_map();
}

fn pre_version(version: &str) -> Option<String> {
    let tmp: Vec<&str> = version.clone().rsplit(".").collect();
    let num = tmp.get(0).unwrap().parse::<u64>().unwrap();
    if num > 0 {
        let append_whitespace = format!("{} ", version.clone());
        let pre_version = append_whitespace.replace(
            format!(".{} ", num).as_str(),
            format!(".{}", num - 1).as_str(),
        );
        return Some(pre_version);
    }
    None
}

// Generates the compiled stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    // pass argument 'version' to generate new release
    // for example, "cargo run -- --version 0.1"
    let cli = App::new("stdlib")
        .name("Move standard library")
        .author("The Starcoin Core Contributors")
        .arg(
            Arg::with_name("version")
                .short("v")
                .long("version")
                .takes_value(true)
                .value_name("VERSION")
                .help("version number for compiled stdlib: major.minor, don't forget to record the release note"),
        )
        .arg(
            Arg::with_name("no-check-compatibility")
                .long("no-check-compatibility")
                .help("don't check compatibility between the old and new standard library"),
        );

    let matches = cli.get_matches();
    let mut generate_new_version = false;
    let mut version_number = "0.0".to_string();
    let pre_version = if matches.is_present("version") {
        generate_new_version = true;
        version_number = matches.value_of("version").unwrap().to_string();
        pre_version(&version_number)
    } else {
        None
    };

    let no_check_compatibility = matches.is_present("no-check-compatibility");

    // Make sure that the current directory is `vm/stdlib` from now on.
    let exec_path = std::env::args().next().expect("path of the executable");
    let base_path = std::path::Path::new(&exec_path)
        .parent()
        .unwrap()
        .join("../../vm/stdlib");
    std::env::set_current_dir(&base_path).expect("failed to change directory");

    let mut txn_scripts_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    txn_scripts_path.push(TRANSACTION_SCRIPTS);
    std::fs::create_dir_all(&txn_scripts_path).unwrap();

    let mut init_scripts_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    init_scripts_path.push(INIT_SCRIPTS);
    std::fs::create_dir_all(&init_scripts_path).unwrap();

    // Write the stdlib blob
    let mut module_path = PathBuf::from(LATEST_COMPILED_OUTPUT_PATH);
    module_path.push(STDLIB_DIR_NAME);
    let new_modules = build_stdlib();

    let mut is_compatible = true;
    if !no_check_compatibility {
        let old_compiled_modules = latest_compiled_modules();
        for module in new_modules.values() {
            // extract new linking/layout API and check compatibility with old
            let new_module_id = module.self_id();
            if let Some(old_module) = old_compiled_modules.get(&new_module_id) {
                let compatibility = check_compiled_module_compat(old_module, module);
                if is_compatible && !compatibility {
                    println!("Stdlib {:?} is incompatible!", new_module_id);
                    is_compatible = false
                }
            }
        }
    }

    if no_check_compatibility || is_compatible {
        if generate_new_version {
            let dest_dir = full_update_with_version(&version_number);

            if let Some(pre_version) = pre_version {
                let mut pre_version_dir = PathBuf::from(COMPILED_OUTPUT_PATH);
                pre_version_dir.push(pre_version.clone());
                let sub_dir = format!("{}-{}", pre_version, version_number);
                incremental_update_with_version(
                    &mut pre_version_dir,
                    dest_dir,
                    sub_dir,
                    &new_modules,
                );
            }
        }

        replace_stdlib_by_path(&mut module_path, new_modules);
    }
}
