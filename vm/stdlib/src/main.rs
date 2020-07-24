// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use stdlib::{
    build_stdlib, build_stdlib_doc, build_transaction_script_doc, compile_script,
    filter_move_files, save_binary, INIT_SCRIPTS, STAGED_EXTENSION, STAGED_OUTPUT_PATH,
    STAGED_STDLIB_PATH, STD_LIB_DOC_DIR, TRANSACTION_SCRIPTS, TRANSACTION_SCRIPTS_DOC_DIR,
};

fn compile_scripts(script_dir: &Path) {
    let script_source_files = datatest_stable::utils::iterate_directory(script_dir);
    let script_files = filter_move_files(script_source_files);
    for script_file in script_files {
        let compiled_script = compile_script(script_file.clone());
        let mut output_path = PathBuf::from(STAGED_OUTPUT_PATH);
        output_path.push(script_file.clone());
        output_path.set_extension(STAGED_EXTENSION);
        File::create(output_path)
            .unwrap()
            .write_all(&compiled_script)
            .unwrap();
    }
}

// Generates the staged stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let mut txn_scripts_path = PathBuf::from(STAGED_OUTPUT_PATH);
    txn_scripts_path.push(TRANSACTION_SCRIPTS);
    std::fs::create_dir_all(&txn_scripts_path).unwrap();

    let mut init_scripts_path = PathBuf::from(STAGED_OUTPUT_PATH);
    init_scripts_path.push(INIT_SCRIPTS);
    std::fs::create_dir_all(&init_scripts_path).unwrap();

    // Write the stdlib blob
    let mut module_path = PathBuf::from(STAGED_OUTPUT_PATH);
    module_path.push(STAGED_STDLIB_PATH);
    std::fs::remove_dir_all(&module_path).unwrap();
    std::fs::create_dir_all(&module_path).unwrap();
    for (name, module) in build_stdlib().into_iter() {
        let mut bytes = Vec::new();
        module.serialize(&mut bytes).unwrap();
        module_path.push(name);
        module_path.set_extension(STAGED_EXTENSION);
        save_binary(module_path.as_path(), &bytes);
        module_path.pop();
    }

    compile_scripts(Path::new(INIT_SCRIPTS));
    compile_scripts(Path::new(TRANSACTION_SCRIPTS));

    // Generate documentation
    std::fs::remove_dir_all(&STD_LIB_DOC_DIR).unwrap_or(());
    std::fs::create_dir_all(&STD_LIB_DOC_DIR).unwrap();
    build_stdlib_doc();

    std::fs::remove_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap_or(());
    std::fs::create_dir_all(&TRANSACTION_SCRIPTS_DOC_DIR).unwrap();
    build_transaction_script_doc();
}
