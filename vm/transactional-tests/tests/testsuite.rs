// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_transactional_test_harness::run_test;
use std::path::Path;

pub const FUNCTIONAL_TEST_DIR: &str = "tests";

fn transactional_testsuite(path: &Path) -> datatest_stable::Result<()> {
    let _log = starcoin_logger::init();

    run_test(path)
}

datatest_stable::harness!(
    transactional_testsuite,
    FUNCTIONAL_TEST_DIR,
    r".*\.(mvir|move)$"
);

//
// struct MoveSourceCompiler {
//     deps: Vec<String>,
//     temp_files: Vec<NamedTempFile>,
// }
//
// impl MoveSourceCompiler {
//     fn new(stdlib_dir: String) -> Self {
//         MoveSourceCompiler {
//             deps: vec![stdlib_dir],
//             temp_files: vec![],
//         }
//     }
// }
//
// #[derive(Debug)]
// struct MoveSourceCompilerError(pub String);
//
// impl fmt::Display for MoveSourceCompilerError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         writeln!(f, "\n\n{}", self.0)
//     }
// }
//
// impl std::error::Error for MoveSourceCompilerError {}
//
// impl Compiler for MoveSourceCompiler {
//     /// Compile a transaction script or module.
//     fn compile<Logger: FnMut(String)>(
//         &mut self,
//         _log: Logger,
//         _address: AccountAddress,
//         input: &str,
//     ) -> Result<ScriptOrModule> {
//         let cur_file = NamedTempFile::new()?;
//         // let sender_addr = Address::try_from(_address.as_ref()).unwrap();
//         cur_file.reopen()?.write_all(input.as_bytes())?;
//         let cur_path = cur_file.path().to_str().unwrap().to_owned();
//
//         let targets = &vec![cur_path.clone()];
//         // let sender = Some(sender_addr);
//
//         let (files, units_or_errors) = move_compiler::Compiler::new(targets, &self.deps)
//             .set_flags(Flags::empty().set_sources_shadow_deps(true))
//             .build()?;
//         let unit = match units_or_errors {
//             Err(errors) => {
//                 let error_buffer = if read_bool_env_var(testsuite::PRETTY) {
//                     move_compiler::diagnostics::report_diagnostics_to_color_buffer(&files, errors)
//                 } else {
//                     move_compiler::diagnostics::report_diagnostics_to_buffer(&files, errors)
//                 };
//                 return Err(
//                     MoveSourceCompilerError(String::from_utf8(error_buffer).unwrap()).into(),
//                 );
//             }
//             Ok(units) => {
//                 let (mut units, warnings) = units;
//                 report_warnings(&files, warnings);
//                 let len = units.len();
//                 if len != 1 {
//                     bail!("Invalid input. Expected 1 compiled unit but got {}", len)
//                 }
//                 units.pop().unwrap()
//             }
//         };
//
//         Ok(match unit.into_compiled_unit() {
//             CompiledUnit::Script(NamedCompiledScript { script, .. }) => {
//                 ScriptOrModule::Script(script)
//             }
//             CompiledUnit::Module(NamedCompiledModule { module, .. }) => {
//                 // let input = format!("address {} {{\n{}\n}}", sender_addr, input);
//                 // cur_file.reopen()?.write_all(input.as_bytes())?;
//                 self.temp_files.push(cur_file);
//                 self.deps.push(cur_path);
//                 ScriptOrModule::Module(module)
//             }
//         })
//     }
//
//     fn use_compiled_genesis(&self) -> bool {
//         true
//     }
// }
