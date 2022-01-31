// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use move_command_line_common;
pub use move_compiler::compiled_unit::{verify_units, CompiledUnit};
pub use move_compiler::diagnostics;
pub use move_compiler::Compiler;

use crate::diagnostics::report_diagnostics_to_color_buffer;
/// A wrap to move-lang compiler
use anyhow::{bail, ensure, Result};
use move_compiler::compiled_unit::AnnotatedCompiledUnit;
use move_compiler::diagnostics::{unwrap_or_report_diagnostics, Diagnostics, FilesSourceText};
use move_compiler::shared::{Flags, NumericalAddress};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::compatibility::Compatibility;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::normalized::Module;
use starcoin_vm_types::{errors::Location, errors::VMResult};
use std::collections::{BTreeMap, HashMap};
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};

pub mod bytecode_transpose;

pub mod utils;
pub mod command_line {
    use crate::shared::NumericalAddress;
    pub fn parse_address(s: &str) -> Result<NumericalAddress, String> {
        let s = if !s.starts_with("0x") {
            format!("0x{}", s)
        } else {
            s.to_owned()
        };
        NumericalAddress::parse_str(&s)
    }
}

pub mod compiled_unit {
    pub use move_compiler::compiled_unit::*;
}

pub mod shared {
    pub use move_compiler::shared::*;
}

pub fn starcoin_framework_named_addresses() -> BTreeMap<String, NumericalAddress> {
    let mapping = [
        ("VMReserved", "0x0"),
        ("Genesis", "0x1"),
        ("StarcoinFramework", "0x1"),
        ("StarcoinAssociation", "0xA550C18"),
    ];
    mapping
        .iter()
        .map(|(name, addr)| (name.to_string(), NumericalAddress::parse_str(addr).unwrap()))
        .collect()
}

// pub mod test_utils {
//     pub use move_lang_test_utils::*;
// }

pub mod dependency_order;

/// Substitutes the placeholders variables.
fn substitute_variable<S: ::std::hash::BuildHasher>(
    text: &str,
    vars: HashMap<&str, String, S>,
) -> String {
    static PAT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([A-Za-z][A-Za-z0-9]*)\}\}").unwrap());
    PAT.replace_all(text, |caps: &Captures| {
        let name = &caps[1];
        vars.get(name)
            .map(|s| s.to_string())
            //if name not found, replace with origin place holder {{name}}, '{{' in format represent '{'
            .unwrap_or_else(|| format!("{{{{{}}}}}", name))
    })
    .to_string()
}

/// perform Windows style line ending (CRLF) to Unix stype (LF) conversion in given file
fn windows_line_ending_to_unix_in_file(file_path: &str) -> Result<&str> {
    let content = std::fs::read_to_string(file_path)?;
    let converted = content.replace("\r\n", "\n");
    // only write back when conversion actually takes place
    if converted != content {
        std::fs::write(file_path, converted)?;
    }
    Ok(file_path)
}

//TODO find a graceful method to do source file pre process and replace placeholders.
/// Replace {{variable}} placeholders in source file, default variable is `sender`.
pub fn process_source_tpl<S: ::std::hash::BuildHasher>(
    source: &str,
    sender: AccountAddress,
    ext_vars: HashMap<&str, String, S>,
) -> String {
    let mut vars = ext_vars;
    vars.insert("sender", format!("{}", sender));
    substitute_variable(source, vars)
}

pub fn process_source_tpl_file<P>(
    temp_dir: P,
    source_file: P,
    sender: AccountAddress,
) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let temp_dir = temp_dir.as_ref();
    ensure!(temp_dir.is_dir(), "temp_dir must be dir.");
    let source_file = source_file.as_ref();
    ensure!(source_file.exists(), "{:?} not exist.", source_file);
    ensure!(source_file.is_file(), "{:?} not a file.", source_file);
    let source = std::fs::read_to_string(source_file)?;
    let processed_source = process_source_tpl(source.as_str(), sender, HashMap::new());
    let temp_file = temp_dir
        .join(
            source_file
                .file_name()
                .expect("source_file must contains file_name."),
        )
        .with_extension(move_command_line_common::files::MOVE_EXTENSION);
    std::fs::write(temp_file.as_path(), processed_source)?;
    Ok(temp_file)
}

/// Compile source, and report error.
pub fn compile_source_string(
    source: &str,
    deps: &[String],
    sender: AccountAddress,
) -> anyhow::Result<(FilesSourceText, Vec<CompiledUnit>)> {
    let (source_text, compiled_result) = compile_source_string_no_report(source, deps, sender)?;

    let compiled_units = unwrap_or_report_diagnostics(&source_text, compiled_result);

    println!(
        "{}",
        String::from_utf8_lossy(&report_diagnostics_to_color_buffer(
            &source_text,
            compiled_units.1
        ))
    );

    // report_warnings(&source_text, compiled_units.1);

    Ok((
        source_text,
        compiled_units
            .0
            .into_iter()
            .map(|c| c.into_compiled_unit())
            .collect(),
    ))
}

/// Compile source, and return compile error.
pub fn compile_source_string_no_report(
    source: &str,
    deps: &[String],
    sender: AccountAddress,
) -> Result<(
    FilesSourceText,
    Result<(Vec<AnnotatedCompiledUnit>, Diagnostics), Diagnostics>,
)> {
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("temp.move");
    let processed_source = process_source_tpl(
        source.replace("\r\n", "\n").as_str(),
        sender,
        HashMap::new(),
    );
    std::fs::write(temp_file.as_path(), processed_source.as_bytes())?;
    let targets = vec![temp_file
        .to_str()
        .expect("temp file path must is str.")
        .to_string()];
    for dep in deps {
        windows_line_ending_to_unix_in_file(dep)?;
    }
    let compiler = move_compiler::Compiler::new(&targets, deps)
        .set_named_address_values(starcoin_framework_named_addresses())
        .set_flags(Flags::empty().set_sources_shadow_deps(true));
    compiler.build()
}

/// check module compatibility
pub fn check_module_compat(pre_code: &[u8], new_code: &[u8]) -> VMResult<bool> {
    let pre_module =
        CompiledModule::deserialize(pre_code).map_err(|e| e.finish(Location::Undefined))?;
    let new_module =
        CompiledModule::deserialize(new_code).map_err(|e| e.finish(Location::Undefined))?;

    let old = Module::new(&pre_module);
    let new = Module::new(&new_module);

    Ok(Compatibility::check(&old, &new).is_fully_compatible())
}

/// check module compatibility
pub fn check_compiled_module_compat(pre: &CompiledModule, new: &CompiledModule) -> bool {
    let old = Module::new(pre);
    let new = Module::new(new);

    Compatibility::check(&old, &new).is_fully_compatible()
}

/// Load bytecode file, return the bytecode bytes, and whether it's script.
pub fn load_bytecode_file<P: AsRef<Path>>(file_path: P) -> Result<(Vec<u8>, bool)> {
    let mut file = OpenOptions::new().read(true).write(false).open(file_path)?;
    let mut bytecode = vec![];
    file.read_to_end(&mut bytecode)?;
    let is_script =
        match starcoin_vm_types::file_format::CompiledScript::deserialize(bytecode.as_slice()) {
            Err(_) => {
                match starcoin_vm_types::file_format::CompiledModule::deserialize(
                    bytecode.as_slice(),
                ) {
                    Ok(_) => false,
                    Err(e) => {
                        bail!(
                            "invalid bytecode file, cannot deserialize as script or module, {}",
                            e
                        );
                    }
                }
            }
            Ok(_) => true,
        };
    Ok((bytecode, is_script))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::parse_address;
    use starcoin_vm_types::language_storage::CORE_CODE_ADDRESS;

    #[test]
    fn test_unknown_place_holder() {
        let source_tpl = r#"
        script{
        use {{alice}}.MyModule;
        fun main() {
        }
        }
        "#;
        let sender = parse_address("0x1dcd9f05cc902e4f342a404ade878efa").unwrap();
        let mut vars = HashMap::new();
        vars.insert("counter", format!("{}", 1));
        let source = process_source_tpl(source_tpl, sender.into_inner(), vars);
        //replace fail.
        assert!(source.contains("{{alice}}"));
        assert_eq!(
            source_tpl,
            source.as_str(),
            "source tpl should equals source after replace fail."
        );
    }

    #[test]
    fn test_process_source_tpl() {
        let source_tpl = r#"
        script{
        use {{sender}}.MyModule;
        fun main() {
            let counter = {{counter}};
            MyModule::init();
            assert({{counter}} > 0, 1000)
            assert({{sender}}!=0x0, 1000);
        }
        }
        "#;
        let sender = parse_address("0x1dcd9f05cc902e4f342a404ade878efa").unwrap();
        let mut vars = HashMap::new();
        vars.insert("counter", format!("{}", 1));
        let source = process_source_tpl(source_tpl, sender.into_inner(), vars);
        assert!(!source.contains("sender"))
    }

    #[stest::test]
    fn test_compat() {
        let test_cases = vec![
            (
                r#"
            module 0x1::M {
                struct M{
                    value: u64,
                }

                public fun hello(){
                }
            }
        "#,
                r#"
            module 0x1::M {
                struct M{
                    value: u64,
                }
                
                struct M2{
                    value: u128,
                }

                public fun hello(){
                }
                
                public fun hello2(){
                }
            }
        "#,
                true,
            ),
            (
                r#"
            module 0x1::M {
                struct M{
                    value: u64,
                }
            }
        "#,
                r#"
            module 0x1::M {
                struct M{
                    value: u64,
                    new_field: address,
                }
            }
        "#,
                false,
            ),
        ];
        for (pre_version, new_version, expect) in test_cases {
            do_test_compat(pre_version, new_version, expect);
        }
    }

    fn do_test_compat(pre_source_code: &str, new_source_code: &str, expect: bool) {
        let pre_code = compile_source_string_no_report(pre_source_code, &[], CORE_CODE_ADDRESS)
            .unwrap()
            .1
            .unwrap()
            .0
            .pop()
            .unwrap()
            .into_compiled_unit()
            .serialize();
        let new_code = compile_source_string_no_report(new_source_code, &[], CORE_CODE_ADDRESS)
            .unwrap()
            .1
            .unwrap()
            .0
            .pop()
            .unwrap()
            .into_compiled_unit()
            .serialize();
        let compatible = check_module_compat(pre_code.as_slice(), new_code.as_slice()).unwrap();
        assert_eq!(compatible, expect);
    }
}
