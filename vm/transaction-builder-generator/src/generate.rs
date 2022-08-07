// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Code generator for Move script builders
//!
//! '''bash
//! cargo run -p transaction-builder-generator -- --help
//! '''

use clap::Parser;
use serde_generate as serdegen;
use serde_reflection::Registry;
use std::path::PathBuf;
use std::str::FromStr;
use transaction_builder_generator as buildgen;
use transaction_builder_generator::is_supported_abi;

#[derive(Debug, Parser)]
enum Language {
    Python3,
    Rust,
    Cpp,
    Java,
    Dart,
}
impl Language {
    fn variants() -> [&'static str; 5] {
        ["python3", "rust", "cpp", "java", "dart"]
    }
}
impl FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "python3" => Ok(Language::Python3),
            "rust" => Ok(Language::Rust),
            "cpp" => Ok(Language::Cpp),
            "java" => Ok(Language::Java),
            "dart" => Ok(Language::Dart),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

#[derive(Debug, Parser)]
#[clap(
    name = "Transaction builder generator",
    about = "Generate code for Move script builders"
)]
struct Options {
    /// Path to the directory containing ABI files in BCS encoding.
    abi_directory: PathBuf,

    /// Language for code generation.
    #[clap(long, possible_values = Language::variants(), default_value = "python3")]
    language: Language,

    /// Directory where to write generated modules (otherwise print code on stdout).
    #[clap(long)]
    target_source_dir: Option<PathBuf>,

    /// Also install the starcoin types described by the given YAML file, along with the BCS runtime.
    #[clap(long)]
    with_starcoin_types: Option<PathBuf>,

    /// Module name for the transaction builders installed in the `target_source_dir`.
    /// * Rust crates may contain a version number, e.g. "test:1.2.0".
    /// * In Java, this is expected to be a package name, e.g. "com.test" to create Java files in `com/test`.
    /// * In Go, this is expected to be of the format "go_module/path/go_package_name",
    /// and `starcoin_types` is assumed to be in "go_module/path/starcoin_types".
    #[clap(long)]
    module_name: Option<String>,

    /// Optional package name (Python) or module path (Go) of the Serde and BCS runtime dependencies.
    #[clap(long)]
    serde_package_name: Option<String>,

    /// Optional version number for the `starcoin_types` module (useful in Rust).
    /// If `--with-starcoin-types` is passed, this will be the version of the generated `starcoin_types` module.
    #[clap(long, default_value = "0.1.0")]
    starcoin_version_number: String,

    /// Optional package name (Python) or module path (Go) of the `starcoin_types` dependency.
    #[clap(long)]
    starcoin_package_name: Option<String>,

    /// Read custom code for starcoin containers from the given file paths. Containers will be matched with file stems.
    /// (e.g. `AddressAccount` <- `path/to/AddressAccount.py`)
    #[clap(long)]
    with_custom_starcoin_code: Vec<PathBuf>,
}

fn main() {
    let options = Options::parse();
    let abis =
        buildgen::read_abis(&options.abi_directory).expect("Failed to read ABI in directory");
    let abis = abis
        .into_iter()
        .filter(is_supported_abi)
        .collect::<Vec<_>>();
    let install_dir = match options.target_source_dir {
        None => {
            // Nothing to install. Just print to stdout.
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            match options.language {
                Language::Python3 => buildgen::python3::output(
                    &mut out,
                    options.serde_package_name.clone(),
                    options.starcoin_package_name.clone(),
                    &abis,
                )
                .unwrap(),
                Language::Rust => {
                    buildgen::rust::output(&mut out, &abis, /* local types */ false).unwrap()
                }
                Language::Cpp => {
                    buildgen::cpp::output(&mut out, &abis, options.module_name.as_deref()).unwrap()
                }
                Language::Java => {
                    panic!("Code generation in Java requires --target_source_dir");
                }
                Language::Dart => {
                    // let module_name = options.module_name.as_deref().unwrap_or("Helpers");
                    // let parts = module_name.rsplitn(2, '.').collect::<Vec<_>>();
                    // let (_, class_name) = if parts.len() > 1 {
                    //     (Some(parts[1]), parts[0])
                    // } else {
                    //     (None, parts[0])
                    // };
                    // buildgen::dart::output(&mut out, &abis, class_name).unwrap()
                    panic!("Code generation in dart requires --target_source_dir");
                }
            }
            return;
        }
        Some(dir) => dir,
    };

    // Starcoin types
    if let Some(registry_file) = options.with_starcoin_types {
        let installer: Box<dyn serdegen::SourceInstaller<Error = Box<dyn std::error::Error>>> =
            match options.language {
                Language::Python3 => Box::new(serdegen::python3::Installer::new(
                    install_dir.clone(),
                    options.serde_package_name.clone(),
                )),
                Language::Rust => Box::new(serdegen::rust::Installer::new(install_dir.clone())),
                Language::Cpp => Box::new(serdegen::cpp::Installer::new(install_dir.clone())),
                Language::Java => Box::new(serdegen::java::Installer::new(install_dir.clone())),
                Language::Dart => Box::new(serdegen::dart::Installer::new(install_dir.clone())),
            };

        match options.language {
            Language::Rust => (), // In Rust, runtimes are deployed as crates.
            _ => {
                installer.install_serde_runtime().unwrap();
                installer.install_bcs_runtime().unwrap();
            }
        }
        let content =
            std::fs::read_to_string(registry_file).expect("registry file must be readable");
        let registry = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();
        let (starcoin_package_name, starcoin_package_path) = match options.language {
            Language::Rust => (
                if options.starcoin_version_number == "0.1.0" {
                    "starcoin-types".to_string()
                } else {
                    format!("starcoin-types:{}", options.starcoin_version_number)
                },
                vec!["starcoin-types"],
            ),
            Language::Java => (
                "org.starcoin.types".to_string(),
                vec!["org", "starcoin", "types"],
            ),
            _ => ("starcoin_types".to_string(), vec!["starcoin_types"]),
        };
        let custom_starcoin_code = buildgen::read_custom_code_from_paths(
            &starcoin_package_path,
            options.with_custom_starcoin_code.into_iter(),
        );
        let config = serdegen::CodeGeneratorConfig::new(starcoin_package_name)
            .with_encodings(vec![serdegen::Encoding::Bcs])
            .with_custom_code(custom_starcoin_code);
        installer.install_module(&config, &registry).unwrap();
    }

    // Transaction builders
    let installer: Box<dyn buildgen::SourceInstaller<Error = Box<dyn std::error::Error>>> =
        match options.language {
            Language::Python3 => Box::new(buildgen::python3::Installer::new(
                install_dir,
                options.serde_package_name,
                options.starcoin_package_name,
            )),
            Language::Rust => Box::new(buildgen::rust::Installer::new(
                install_dir,
                options.starcoin_version_number,
            )),
            Language::Cpp => Box::new(buildgen::cpp::Installer::new(install_dir)),
            Language::Java => Box::new(buildgen::java::Installer::new(install_dir)),
            Language::Dart => Box::new(buildgen::dart::Installer::new(install_dir)),
        };

    if let Some(name) = options.module_name {
        installer
            .install_transaction_builders(&name, abis.as_slice())
            .unwrap();
    }
}
