// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;
use tempfile::tempdir;

//ignore test for cc is oom killed by os.
#[ignore]
#[test]
fn test_that_installed_rust_code_compiles() {
    let dir = tempdir().unwrap();

    let status = Command::new("cargo")
        .current_dir("../..")
        .arg("run")
        .arg("-p")
        .arg("transaction-builder-generator")
        .arg("--")
        .arg("--language")
        .arg("rust")
        .arg("--module-name")
        .arg("starcoin-stdlib:0.1.1")
        .arg("--with-diem-types")
        .arg("etc/starcoin_types.yml")
        .arg("--target-source-dir")
        .arg(dir.path())
        .arg("vm/stdlib/compiled/latest/transaction_scripts/abi")
        .status()
        .unwrap();
    assert!(status.success());

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    // let target_dir = std::env::current_dir().unwrap().join("../../target");
    // println!("target dir is {:?}", target_dir.as_path());
    // let status = Command::new("cargo")
    //     .current_dir(dir.path().join("starcoin-stdlib"))
    //     .arg("build")
    //     .arg("--target-dir")
    //     .arg(target_dir)
    //     .status()
    //     .unwrap();
    // assert!(status.success());
}
