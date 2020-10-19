// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::temp_path;
use starcoin_move_compiler::test_utils::error;
use starcoin_move_compiler::{
    compiled_unit::verify_units, errors::report_errors_to_buffer, move_compile_no_report,
    shared::Address, test_utils::*,
};
use std::{fs, path::Path};

pub const STD_LIB_DIR: &str = "../stdlib/modules";
pub const STD_LIB_COMPILED_DIR: &str = "../stdlib/compiled/latest/stdlib";
pub const STD_LIB_TRANSACTION_SCRIPTS_DIR: &str = "../stdlib/transaction_scripts";

const OUT_EXT: &str = "out";

const KEEP_TMP: &str = "KEEP";

// Runs all tests under the test/testsuite directory.
fn sanity_check_testsuite_impl(
    path: &Path,
    std_lib_is_target: bool,
    std_lib_dir: String,
) -> datatest_stable::Result<()> {
    let mut targets: Vec<String> = vec![path.to_str().unwrap().to_owned()];
    let mut deps: Vec<String> = vec![];
    if std_lib_is_target {
        targets.push(std_lib_dir)
    } else {
        deps.push(std_lib_dir)
    }
    let sender = Some(Address::LIBRA_CORE);

    let out_path = path.with_extension(OUT_EXT);
    let temp_dir = temp_path();
    let (files, units_or_errors) = move_compile_no_report(
        &targets,
        &deps,
        sender,
        Some(
            temp_dir
                .path()
                .to_str()
                .expect("path to string should success")
                .to_string(),
        ),
    )?;
    let errors = match units_or_errors {
        Err(errors) => errors,
        Ok(units) => verify_units(units).1,
    };
    let has_errors = !errors.is_empty();
    let error_buffer = if has_errors {
        report_errors_to_buffer(files, errors)
    } else {
        vec![]
    };

    let save_errors = read_bool_var(KEEP_TMP);

    fs::write(out_path.clone(), error_buffer)?;
    let rendered_errors = fs::read_to_string(out_path.clone())?;
    if !save_errors {
        fs::remove_file(out_path)?;
    }

    if has_errors {
        let msg = format!("Expected success. Unexpected errors:\n{}", rendered_errors);
        error(msg)
    } else {
        Ok(())
    }
}

fn sanity_check_testsuite(path: &Path) -> datatest_stable::Result<()> {
    sanity_check_testsuite_impl(path, true, STD_LIB_DIR.to_string())?;
    sanity_check_testsuite_impl(path, false, STD_LIB_COMPILED_DIR.to_string())
}

datatest_stable::harness!(
    sanity_check_testsuite,
    STD_LIB_TRANSACTION_SCRIPTS_DIR,
    r".*\.move"
);
