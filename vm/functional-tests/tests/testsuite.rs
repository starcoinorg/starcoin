// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::{bail, Result};
use move_command_line_common::env::read_bool_env_var;
use move_lang::compiled_unit::{NamedCompiledModule, NamedCompiledScript};
use move_lang::diagnostics::report_warnings;
use starcoin_functional_tests::compiler::{Compiler, ScriptOrModule};
use starcoin_functional_tests::testsuite;
use starcoin_move_compiler::compiled_unit::CompiledUnit;
use starcoin_move_compiler::shared::Flags;
use starcoin_vm_types::account_address::AccountAddress;
use std::{fmt, io::Write, path::Path};
use tempfile::NamedTempFile;

pub const STD_LIB_DIR: &str = "../stdlib/modules";
pub const FUNCTIONAL_TEST_DIR: &str = "tests";

struct MoveSourceCompiler {
    deps: Vec<String>,
    temp_files: Vec<NamedTempFile>,
}

impl MoveSourceCompiler {
    fn new(stdlib_dir: String) -> Self {
        MoveSourceCompiler {
            deps: vec![stdlib_dir],
            temp_files: vec![],
        }
    }
}

#[derive(Debug)]
struct MoveSourceCompilerError(pub String);

impl fmt::Display for MoveSourceCompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\n\n{}", self.0)
    }
}

impl std::error::Error for MoveSourceCompilerError {}

impl Compiler for MoveSourceCompiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String)>(
        &mut self,
        _log: Logger,
        _address: AccountAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        let cur_file = NamedTempFile::new()?;
        // let sender_addr = Address::try_from(_address.as_ref()).unwrap();
        cur_file.reopen()?.write_all(input.as_bytes())?;
        let cur_path = cur_file.path().to_str().unwrap().to_owned();

        let targets = &vec![cur_path.clone()];
        // let sender = Some(sender_addr);

        let (files, units_or_errors) = move_lang::Compiler::new(targets, &self.deps)
            .set_flags(Flags::empty().set_sources_shadow_deps(true))
            .build()?;
        let unit = match units_or_errors {
            Err(errors) => {
                let error_buffer = if read_bool_env_var(testsuite::PRETTY) {
                    move_lang::diagnostics::report_diagnostics_to_color_buffer(&files, errors)
                } else {
                    move_lang::diagnostics::report_diagnostics_to_buffer(&files, errors)
                };
                return Err(
                    MoveSourceCompilerError(String::from_utf8(error_buffer).unwrap()).into(),
                );
            }
            Ok(units) => {
                let (mut units, warnings) = units;
                report_warnings(&files, warnings);
                let len = units.len();
                if len != 1 {
                    bail!("Invalid input. Expected 1 compiled unit but got {}", len)
                }
                units.pop().unwrap()
            }
        };

        Ok(match unit.into_compiled_unit() {
            CompiledUnit::Script(NamedCompiledScript { script, .. }) => {
                ScriptOrModule::Script(script)
            }
            CompiledUnit::Module(NamedCompiledModule { module, .. }) => {
                // let input = format!("address {} {{\n{}\n}}", sender_addr, input);
                // cur_file.reopen()?.write_all(input.as_bytes())?;
                self.temp_files.push(cur_file);
                self.deps.push(cur_path);
                ScriptOrModule::Module(module)
            }
        })
    }

    fn use_compiled_genesis(&self) -> bool {
        true
    }
}

fn functional_testsuite(path: &Path) -> datatest_stable::Result<()> {
    let _log = starcoin_logger::init_for_test();
    testsuite::functional_tests(MoveSourceCompiler::new(STD_LIB_DIR.to_string()), path)
}

datatest_stable::harness!(functional_testsuite, FUNCTIONAL_TEST_DIR, r".*\.move");
