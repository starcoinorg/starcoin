// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_generate as serdegen;
use serde_generate::SourceInstaller as _;
use serde_reflection::Registry;
use starcoin_vm_types::transaction::ScriptABI;
use std::{io::Write, path::Path, process::Command};
use tempfile::tempdir;
use transaction_builder_generator as buildgen;
use transaction_builder_generator::{is_supported_abi, SourceInstaller};

fn get_starcoin_registry() -> Registry {
    let path = "../../etc/starcoin_types.yml";
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str::<Registry>(content.as_str()).unwrap()
}

fn get_stdlib_script_abis() -> Vec<ScriptABI> {
    let path = Path::new("../stdlib/compiled/latest/transaction_scripts/abi");
    buildgen::read_abis(path)
        .expect("reading ABI files should not fail")
        .into_iter()
        .filter(is_supported_abi)
        .collect()
}

const EXPECTED_OUTPUT : &str = "2 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 15 84 114 97 110 115 102 101 114 83 99 114 105 112 116 115 26 112 101 101 114 95 116 111 95 112 101 101 114 95 119 105 116 104 95 109 101 116 97 100 97 116 97 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 83 84 67 3 83 84 67 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 4 32 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 2 135 214 18 0 0 0 0 0 0 0 0 0 0 0 0 0 4 0 \n";
// const OUTPUT : &str = "181 1 161 28 235 11 1 0 0 0 6 1 0 2 3 2 17 4 19 4 5 23 28 7 51 56 8 107 16 0 0 0 1 0 1 1 1 0 2 2 3 0 0 3 4 1 1 1 0 6 2 6 2 5 10 2 0 1 5 1 1 4 6 12 5 4 10 2 5 6 12 5 10 2 4 10 2 1 9 0 7 65 99 99 111 117 110 116 14 99 114 101 97 116 101 95 97 99 99 111 117 110 116 9 101 120 105 115 116 115 95 97 116 22 112 97 121 95 102 114 111 109 95 119 105 116 104 95 109 101 116 97 100 97 116 97 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 5 1 14 10 1 17 1 32 3 5 5 8 10 1 11 2 56 0 11 0 10 1 10 3 11 4 56 1 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 76 66 82 3 76 66 82 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 4 32 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 2 135 214 18 0 0 0 0 0 0 0 0 0 0 0 0 0 4 0 \n";
const OUTPUT: &str = "255 1 161 28 235 11 2 0 0 0 7 1 0 4 3 4 22 4 26 4 5 30 33 7 63 80 8 143 1 16 6 159 1 10 0 0 0 1 0 2 3 4 0 0 3 6 3 1 4 1 4 7 7 0 0 5 8 2 1 4 1 5 3 5 5 6 12 5 10 2 4 10 2 3 5 1 3 0 1 5 1 1 1 9 0 1 10 2 1 3 4 6 12 5 4 10 2 7 65 99 99 111 117 110 116 6 69 114 114 111 114 115 9 101 120 105 115 116 115 95 97 116 14 99 114 101 97 116 101 95 97 99 99 111 117 110 116 16 105 110 118 97 108 105 100 95 97 114 103 117 109 101 110 116 22 112 97 121 95 102 114 111 109 95 119 105 116 104 95 109 101 116 97 100 97 116 97 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 8 101 0 0 0 0 0 0 0 1 4 0 1 27 10 1 17 0 32 3 5 5 21 11 2 56 0 12 5 10 1 10 5 33 7 0 17 2 12 7 12 6 11 6 3 21 11 0 1 11 7 39 11 0 10 1 10 3 11 4 56 1 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 83 84 67 3 83 84 67 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 4 32 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 2 135 214 18 0 0 0 0 0 0 0 0 0 0 0 0 0 4 0 \n";

#[ignore]
#[test]
fn test_that_python_code_parses_and_passes_pyre_check() {
    let registry = get_starcoin_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let src_dir_path = dir.path().join("src");
    let installer =
        serdegen::python3::Installer::new(src_dir_path.clone(), /* package */ None);

    let config = serdegen::CodeGeneratorConfig::new("starcoin_types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs, serdegen::Encoding::Bincode]);
    installer.install_module(&config, &registry).unwrap();

    // installer
    //     .install_module("starcoin_types", &registry)
    //     .unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    let stdlib_dir_path = src_dir_path.join("starcoin_stdlib");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();
    let source_path = stdlib_dir_path.join("__init__.py");

    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::python3::output(&mut source, None, None, &abis).unwrap();

    std::fs::copy(
        "examples/python3/stdlib_demo.py",
        dir.path().join("src/stdlib_demo.py"),
    )
    .unwrap();

    let python_path = format!(
        "{}:{}",
        std::env::var("PYTHONPATH").unwrap_or_default(),
        src_dir_path.to_string_lossy(),
    );
    let output = Command::new("python3")
        .env("PYTHONPATH", python_path)
        .arg(dir.path().join("src/stdlib_demo.py"))
        .output()
        .unwrap();
    eprintln!(
        "stderr: \n {}",
        std::str::from_utf8(&output.stderr).unwrap()
    );
    assert!(output.status.success());
    assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), OUTPUT);

    // This temporarily requires a checkout of serde-reflection.git next to diem.git
    // Hopefully, numpy's next release will include typeshed (.pyi) files and we will only
    // require a local install of numpy (on top of python3 and pyre).
    //     let status = Command::new("cp")
    //         .arg("-r")
    //         .arg("../../../serde-reflection/serde-generate/runtime/python/typeshed")
    //         .arg(dir.path())
    //         .status()
    //         .unwrap();
    //     assert!(status.success());
    //
    //     let mut pyre_config = std::fs::File::create(dir.path().join(".pyre_configuration")).unwrap();
    //     writeln!(
    //         &mut pyre_config,
    //         r#"{{
    //   "source_directories": [
    //     "src"
    //   ],
    //   "search_path": [
    //     "typeshed"
    //   ]
    // }}"#,
    //     )
    //     .unwrap();
    //
    //     let status = Command::new("pyre")
    //         .current_dir(dir.path())
    //         .arg("check")
    //         .status()
    //         .unwrap();
    //     assert!(status.success());
}

//ignore test for cc is oom killed by os.
#[ignore]
#[test]
fn test_that_rust_code_compiles() {
    let registry = get_starcoin_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::rust::Installer::new(dir.path().to_path_buf());
    let config = serdegen::CodeGeneratorConfig::new("starcoin-types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs, serdegen::Encoding::Bincode]);
    installer.install_module(&config, &registry).unwrap();

    let stdlib_dir_path = dir.path().join("starcoin-stdlib");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();

    let mut cargo = std::fs::File::create(&stdlib_dir_path.join("Cargo.toml")).unwrap();
    write!(
        cargo,
        r#"[package]
name = "starcoin-stdlib"
version = "0.1.0"
edition = "2018"

[dependencies]
serde_bytes = "0.11"
serde = {{ version = "1.0.114", features = ["derive"] }}
bcs_ext = {{ package="bcs-ext", git = "https://github.com/starcoinorg/starcoin"}}
starcoin-types = {{ path = "../starcoin-types", version = "0.1.0" }}
once_cell = "1.4.0"

[[bin]]
name = "stdlib_demo"
path = "src/stdlib_demo.rs"
test = false
"#
    )
    .unwrap();
    std::fs::create_dir(stdlib_dir_path.join("src")).unwrap();
    let source_path = stdlib_dir_path.join("src/lib.rs");
    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::rust::output(&mut source, &abis, /* local types */ false).unwrap();

    std::fs::copy(
        "examples/rust/stdlib_demo.rs",
        stdlib_dir_path.join("src/stdlib_demo.rs"),
    )
    .unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("starcoin-stdlib"))
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir.clone())
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(target_dir.join("debug/stdlib_demo"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
}

#[test]
#[ignore]
fn test_that_cpp_code_compiles_and_demo_runs() {
    let registry = get_starcoin_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::cpp::Installer::new(dir.path().to_path_buf());
    // lcs_installer
    //     .install_module("starcoin_types", &registry)
    //     .unwrap();
    // lcs_installer.install_serde_runtime().unwrap();
    // lcs_installer.install_bcs_runtime().unwrap();

    let config = serdegen::CodeGeneratorConfig::new("starcoin_types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs, serdegen::Encoding::Bincode]);
    installer.install_module(&config, &registry).unwrap();

    let abi_installer = buildgen::cpp::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("starcoin_stdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/cpp/stdlib_demo.cpp",
        dir.path().join("stdlib_demo.cpp"),
    )
    .unwrap();

    let status = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-g")
        .arg(dir.path().join("starcoin_stdlib.cpp"))
        .arg(dir.path().join("stdlib_demo.cpp"))
        .arg("-o")
        .arg(dir.path().join("stdlib_demo"))
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(dir.path().join("stdlib_demo"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), OUTPUT);
}

#[ignore]
#[test]
fn test_that_java_code_compiles_and_demo_runs() {
    let registry = get_starcoin_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::java::Installer::new(dir.path().to_path_buf());
    // lcs_installer
    //     .install_module("org.starcoin.types", &registry)
    //     .unwrap();
    // lcs_installer.install_serde_runtime().unwrap();
    // lcs_installer.install_bcs_runtime().unwrap();

    let config = serdegen::CodeGeneratorConfig::new("org.starcoin.types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs, serdegen::Encoding::Bincode]);
    installer.install_module(&config, &registry).unwrap();

    let abi_installer = buildgen::java::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("org.starcoin.stdlib.Stdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/java/StdlibDemo.java",
        dir.path().join("StdlibDemo.java"),
    )
    .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir(dir.path().join("com/facebook/serde")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("com/facebook/lcs")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("org/starcoin/types")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("org/starcoin/stdlib")).unwrap())
        .map(|e| e.unwrap().path())
        .chain(std::iter::once(dir.path().join("StdlibDemo.java")));

    let status = Command::new("javac")
        .arg("-cp")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new("java")
        .arg("-enableassertions")
        .arg("-cp")
        .arg(dir.path())
        .arg("StdlibDemo")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), OUTPUT);
}
