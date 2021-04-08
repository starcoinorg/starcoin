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
    if !output.stdout.is_empty() {
        println!(
            "git status output:\n {}",
            String::from_utf8(output.stdout.clone()).unwrap()
        )
    }
    assert!(
        output.stdout.is_empty(),
        "Git repository should be in a clean state"
    );
    assert!(output.status.success());
}

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
