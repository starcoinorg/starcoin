// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// A wrap to move-lang compiler
use anyhow::{bail, ensure, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::compatibility::Compatibility;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::normalized::Module;
use starcoin_vm_types::{errors::Location, errors::VMResult};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::shared::AddressBytes;
use move_lang::shared::Flags;
pub use move_lang::{
    compiled_unit::{verify_units, CompiledUnit},
    errors::*,
    move_compile, move_compile_and_report, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};

pub mod errors {
    pub use move_lang::errors::*;
}

//TODO directly use AccountAddress
pub mod command_line {
    use crate::shared::AddressBytes;

    pub fn parse_address(s: &str) -> Result<AddressBytes, String> {
        let s = if !s.starts_with("0x") {
            format!("0x{}", s)
        } else {
            s.to_owned()
        };
        AddressBytes::parse_str(s.as_str())
    }
}

pub mod compiled_unit {
    pub use move_lang::compiled_unit::*;
}

pub mod shared {
    pub use move_lang::shared::*;
}

pub mod test_utils {
    pub use move_lang_test_utils::*;
}

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

//TODO find a graceful method to do source file pre process and replace placeholders.
/// Replace {{variable}} placeholders in source file, default variable is `sender`.
pub fn process_source_tpl<S: ::std::hash::BuildHasher>(
    source: &str,
    sender: AddressBytes,
    ext_vars: HashMap<&str, String, S>,
) -> String {
    let mut vars = ext_vars;
    vars.insert("sender", format!("{}", sender));
    substitute_variable(source, vars)
}

pub fn process_source_tpl_file<P>(
    temp_dir: P,
    source_file: P,
    sender: AddressBytes,
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
        .with_extension(MOVE_EXTENSION);
    std::fs::write(temp_file.as_path(), processed_source)?;
    Ok(temp_file)
}

/// Compile source, and report error.
pub fn compile_sorce_string(
    source: &str,
    deps: &[String],
    sender: AccountAddress,
) -> anyhow::Result<(FilesSourceText, Vec<CompiledUnit>)> {
    let (source_text, compiled_result) = compile_source_string_no_report(source, deps, sender)?;
    match compiled_result {
        Ok(c) => Ok((source_text, c)),
        Err(e) => errors::report_errors(source_text, e),
    }
}

/// Compile source, and return compile error.
pub fn compile_source_string_no_report(
    source: &str,
    deps: &[String],
    sender: AccountAddress,
) -> Result<(FilesSourceText, Result<Vec<CompiledUnit>, Errors>)> {
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("temp.move");
    let sender = AddressBytes::new(sender.into());
    let processed_source = process_source_tpl(source, sender, HashMap::new());
    std::fs::write(temp_file.as_path(), processed_source.as_bytes())?;
    let targets = vec![temp_file
        .to_str()
        .expect("temp file path must is str.")
        .to_string()];
    move_compile(
        &targets,
        deps,
        None,
        Flags::empty().set_sources_shadow_deps(true),
    )
    .map(|(f, u)| {
        // let compiled_result = u.map(|mut us| us.pop().expect("At least one compiled_unit"));
        (f, u)
    })
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
        let source = process_source_tpl(source_tpl, sender, vars);
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
        let source = process_source_tpl(source_tpl, sender, vars);
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
            .pop()
            .unwrap()
            .serialize();
        let new_code = compile_source_string_no_report(new_source_code, &[], CORE_CODE_ADDRESS)
            .unwrap()
            .1
            .unwrap()
            .pop()
            .unwrap()
            .serialize();
        let compatible = check_module_compat(pre_code.as_slice(), new_code.as_slice()).unwrap();
        assert_eq!(compatible, expect);
    }
}
