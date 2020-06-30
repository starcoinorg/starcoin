// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
/// A wrap to move-lang compiler
use crate::shared::Address;
use anyhow::{bail, ensure, Result};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use move_lang::compiled_unit::CompiledUnit;
pub use move_lang::{
    move_check, move_check_no_report, move_compile, move_compile_no_report,
    move_compile_to_expansion_no_report, MOVE_COMPILED_EXTENSION, MOVE_EXTENSION,
};
use starcoin_vm_types::account_address::AccountAddress;

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
    let mut compile_units = match compile_units {
        Err(e) => {
            let err = crate::errors::report_errors_to_color_buffer(file_texts, e);
            bail!(String::from_utf8(err).unwrap())
        }
        Ok(r) => r,
    };
    let compile_result = compile_units.pop().unwrap();
    Ok(compile_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_line::parse_address;

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
}
