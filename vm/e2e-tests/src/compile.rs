// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for compiling scripts and modules in tests.


use starcoin_vm_types::transaction::{Module, Script};
use starcoin_vm_types::file_format::CompiledModule;

/// Compile the provided Move code into a blob which can be used as the code to be published
/// (a Module).
pub fn compile_module(code: &str) -> (CompiledModule, Module) {
    let compiled_module = Compiler {
        deps: cached_framework_packages::modules().iter().collect(),
    }
    .into_compiled_module(code)
    .expect("Module compilation failed");
    let module = Module::new(
        Compiler {
            deps: cached_framework_packages::modules().iter().collect(),
        }
        .into_module_blob(code)
        .expect("Module compilation failed"),
    );
    (compiled_module, module)
}

/// Compile the provided Move code into a blob which can be used as the code to be executed
/// (a Script).
pub fn compile_script(code: &str, extra_deps: Vec<CompiledModule>) -> Script {
    let compiler = Compiler {
        deps: cached_framework_packages::modules()
            .iter()
            .chain(extra_deps.iter())
            .collect(),
    };
    Script::new(
        compiler
            .into_script_blob(code)
            .expect("Script compilation failed"),
        vec![],
        vec![],
    )
}
