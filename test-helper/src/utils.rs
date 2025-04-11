// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

pub fn assert_that_version_control_has_no_unstaged_changes() {
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
