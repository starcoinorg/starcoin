// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Ok, Result};
use starcoin_vm2_move_compiler::move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use starcoin_vm2_types::transaction::{Module, Package};
use std::{fs::File, io::Read, path::Path};

pub fn load_package_from_file(mv_or_package_file: &Path) -> Result<Package> {
    ensure!(
        mv_or_package_file.exists(),
        "file {:?} not exist",
        mv_or_package_file
    );

    let mut bytes = vec![];
    File::open(mv_or_package_file)?.read_to_end(&mut bytes)?;

    let package = if mv_or_package_file.extension().unwrap_or_default() == MOVE_COMPILED_EXTENSION {
        Package::new_with_module(Module::new(bytes))?
    } else {
        bcs_ext::from_bytes(&bytes).map_err(|e| {
            format_err!(
                "Decode Package failed {:?}, please ensure the file is a Package binary file.",
                e
            )
        })?
    };
    Ok(package)
}

// pub fn load_package_from_dir(path: &Path) -> Result<Package> {
//     ensure!(path.is_dir(), "path need to be a dir");
//
//     let mut modules = vec![];
//     for entry in fs::read_dir(path)? {
//         let file = entry?.path();
//         if file.extension().unwrap_or_default() != MOVE_COMPILED_EXTENSION {
//             continue;
//         }
//         let mut bytes = vec![];
//         File::open(file)?.read_to_end(&mut bytes)?;
//         modules.push(Module::new(bytes));
//     }
//
//     ensure!(!modules.is_empty(), "Modules is empty under {:?}", path);
//     let package = Package::new_with_modules(modules)?;
//     Ok(package)
// }
