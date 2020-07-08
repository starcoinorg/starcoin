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
pub use move_lang::compiled_unit::{verify_units, CompiledUnit};
pub use move_lang::{
    errors::*, move_check, move_check_no_report, move_compile, move_compile_no_report,
    move_compile_to_expansion_no_report, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use starcoin_vm_types::bytecode_verifier::VerifiedModule;
use starcoin_vm_types::file_format::CompiledModule;
use std::fs::OpenOptions;
use std::io::Read;

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

pub fn check_module_compat(pre_code: &[u8], new_code: &[u8]) -> Result<()> {
    if pre_code == new_code {
        return Ok(());
    }
    let mut pre_version = CompiledModule::deserialize(pre_code)?;
    let mut new_version = CompiledModule::deserialize(new_code)?;
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

/// Compile move source file, or load move bytecode file, based on file path extension.
pub fn compile_or_load_move_file<P: AsRef<Path>>(
    file_path: P,
    extra_deps: &[String],
    sender: AccountAddress,
) -> Result<(Vec<u8>, bool)> {
    let ext = file_path
        .as_ref()
        .extension()
        .map(|os_str| os_str.to_str().expect("file extension should is utf8 str"))
        .unwrap_or_else(|| "");
    if ext == MOVE_EXTENSION {
        let compile_units = compile_source_string(
            std::fs::read_to_string(file_path)?.as_str(),
            extra_deps,
            sender,
        )?;
        let is_script = match compile_units {
            CompiledUnit::Module { .. } => false,
            CompiledUnit::Script { .. } => true,
        };
        Ok((compile_units.serialize(), is_script))
    } else if ext == MOVE_COMPILED_EXTENSION {
        let mut file = OpenOptions::new().read(true).write(false).open(file_path)?;
        let mut bytecode = vec![];
        file.read_to_end(&mut bytecode)?;
        let is_script = match starcoin_vm_types::file_format::CompiledScript::deserialize(
            bytecode.as_slice(),
        ) {
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
    } else {
        bail!("Only support *.move or *.mv file");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::parse_address;
    use starcoin_logger::prelude::*;
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
        match check_module_compat(pre_code.as_slice(), new_code.as_slice()) {
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
