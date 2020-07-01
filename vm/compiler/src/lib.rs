// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// A wrap to move-lang compiler
use crate::shared::Address;
use anyhow::{bail, ensure, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use starcoin_vm_types::account_address::AccountAddress;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod contract;

use crate::contract::{Contract, ModuleContract};
use move_lang::compiled_unit::CompiledUnit;
pub use move_lang::{
    move_check, move_check_no_report, move_compile, move_compile_no_report,
    move_compile_to_expansion_no_report, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use starcoin_vm_types::bytecode_verifier::VerifiedModule;
use starcoin_vm_types::file_format::CompiledModule;

pub mod errors {
    pub use move_lang::errors::*;
}

//TODO directly use AccountAddress
pub mod command_line {
    use crate::shared::Address;

    pub fn parse_address(s: &str) -> Result<Address, String> {
        let s = if !s.starts_with("0x") {
            format!("0x{}", s)
        } else {
            s.to_owned()
        };
        move_lang::command_line::parse_address(s.as_str())
    }
}

pub mod compiled_unit {
    pub use move_lang::compiled_unit::*;
}

pub mod shared {
    pub use move_lang::shared::*;
}

pub mod test_utils {
    pub use move_lang::test_utils::*;
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
            .unwrap_or_else(|| panic!("Can not find variable by name: {}", name))
    })
    .to_string()
}

//TODO find a gracefull method to do source file pre process and replace placeholders.
/// Replace {{variable}} placeholders in source file, default variable is `sender`.
pub fn process_source_tpl<S: ::std::hash::BuildHasher>(
    source: &str,
    sender: Address,
    ext_vars: HashMap<&str, String, S>,
) -> String {
    let mut vars = ext_vars;
    vars.insert("sender", format!("{}", sender));
    substitute_variable(source, vars)
}

pub fn process_source_tpl_file<P>(temp_dir: P, source_file: P, sender: Address) -> Result<PathBuf>
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

pub fn compile_source_string(
    source: &str,
    deps: &[String],
    sender: AccountAddress,
) -> Result<CompiledUnit> {
    let temp_dir = tempfile::tempdir()?;
    let temp_file = temp_dir.path().join("temp.move");
    let sender = Address::new(sender.into());
    let processed_source = process_source_tpl(source, sender, HashMap::new());
    std::fs::write(temp_file.as_path(), processed_source.as_bytes())?;
    let targets = vec![temp_file
        .to_str()
        .expect("temp file path must is str.")
        .to_string()];
    let (file_texts, compile_units) = move_compile_no_report(&targets, deps, Some(sender))?;
    let mut compiled_units = match compile_units {
        Err(e) => {
            let err = crate::errors::report_errors_to_color_buffer(file_texts, e);
            bail!(String::from_utf8(err).unwrap())
        }
        Ok(r) => r,
    };
    let compiled_unit = compiled_units.pop().expect("At least one compiled_unit");
    Ok(compiled_unit)
}

pub fn check_module_compat(pre_code: Vec<u8>, new_code: Vec<u8>) -> Result<()> {
    if pre_code == new_code {
        return Ok(());
    }
    let mut pre_version = CompiledModule::deserialize(pre_code.as_slice())?;
    let mut new_version = CompiledModule::deserialize(new_code.as_slice())?;
    pre_version = VerifiedModule::new(pre_version)
        .map_err(|e| e.1)?
        .into_inner();
    new_version = VerifiedModule::new(new_version)
        .map_err(|e| e.1)?
        .into_inner();
    let pre_contract = ModuleContract::new(&pre_version);
    let new_contract = ModuleContract::new(&new_version);
    new_contract.compat_with(&pre_contract)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::parse_address;
    use starcoin_logger::prelude::*;
    use starcoin_vm_types::language_storage::CORE_CODE_ADDRESS;

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
        println!("{}", source);
        assert!(!source.contains("sender"))
    }

    #[stest::test]
    fn test_compat() {
        let test_cases = vec![
            (
                r#"
            module M {
                struct M{
                    value: u64,
                }

                public fun hello(){
                }
            }
        "#,
                r#"
            module M {
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
            module M {
                struct M{
                    value: u64,
                }
            }
        "#,
                r#"
            module M {
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
        let pre_code = compile_source_string(pre_source_code, &[], CORE_CODE_ADDRESS)
            .unwrap()
            .serialize();
        let new_code = compile_source_string(new_source_code, &[], CORE_CODE_ADDRESS)
            .unwrap()
            .serialize();
        match check_module_compat(pre_code, new_code) {
            Err(e) => {
                if expect {
                    panic!(e)
                } else {
                    debug!("expected checked compat error: {:?}", e);
                }
            }
            Ok(_) => {
                if !expect {
                    panic!("expect not compat, but compat.")
                }
            }
        }
    }
}
