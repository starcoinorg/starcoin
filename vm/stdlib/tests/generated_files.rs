// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_compiler::construct_pre_compiled_lib;
use move_compiler::shared::PackagePaths;
use starcoin_move_compiler::starcoin_framework_named_addresses;
use std::path::PathBuf;
use std::process::Command;
use stdlib::stdlib_files;
use test_helper::assert_that_version_control_has_no_unstaged_changes;

// TODO: better way to do this maybe?
#[test]
fn test_that_generated_file_are_up_to_date_in_git() {
    // Better not run the `stdlib` tool when the repository is not in a clean state.
    assert_that_version_control_has_no_unstaged_changes();
    // Directly use the compiled debug binary to run, avoid compile again.
    let path = PathBuf::from("../../target/debug/stdlib")
        .canonicalize()
        .unwrap();
    assert!(Command::new(path)
        .current_dir("../..")
        .status()
        .unwrap()
        .success());

    // Running the stdlib tool should not create unstaged changes.
    assert_that_version_control_has_no_unstaged_changes();
}

#[test]
fn test_stdlib_pre_compiled() {
    let sources = stdlib_files();
    let program_res = construct_pre_compiled_lib(
        vec![PackagePaths {
            name: None,
            paths: sources,
            named_address_map: starcoin_framework_named_addresses(),
        }],
        None,
        move_compiler::Flags::empty(),
    )
    .unwrap();
    assert!(program_res.is_ok());
}
