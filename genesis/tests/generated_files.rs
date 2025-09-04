// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;
use test_helper::assert_that_version_control_has_no_unstaged_changes;

// TODO: better way to do this maybe?
#[test]
fn test_that_generated_file_are_up_to_date_in_git() {
    // Better not run the `stdlib` tool when the repository is not in a clean state.
    assert_that_version_control_has_no_unstaged_changes();
    let path = PathBuf::from("../target/debug/starcoin-genesis")
        .canonicalize()
        .unwrap();
    assert!(Command::new(path)
        .current_dir("../")
        .status()
        .unwrap()
        .success());

    // Running the stdlib tool should not create unstaged changes.
    assert_that_version_control_has_no_unstaged_changes();
}
