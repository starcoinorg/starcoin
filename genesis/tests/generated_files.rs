// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;
use std::process::Command;

fn assert_that_version_control_has_no_unstaged_changes() {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .unwrap();
    let output_string = String::from_utf8(output.stdout).unwrap();
    // remove .cargo/config.toml from output
    let output_string = output_string.replace("M .cargo/config.toml", "");
    let output_string = output_string.trim().to_string();
    if !output_string.is_empty() {
        println!("git status output:\n {}", output_string)
    }
    assert!(
        output_string.is_empty(),
        "Git repository should be in a clean state"
    );
    assert!(output.status.success());
}

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
