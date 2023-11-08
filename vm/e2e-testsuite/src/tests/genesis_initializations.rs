// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::account_config;
use move_core_types::{
    account_address::AccountAddress,
    value::{serialize_values, MoveValue},
};
use starcoin_language_e2e_tests::executor::FakeExecutor;

#[test]
fn test_diem_initialize() {
    let mut executor = FakeExecutor::stdlib_only_genesis();

    // DR doesn't have role yet, so role check will fail
    let output = executor.try_exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );
    assert_eq!(output.unwrap_err().move_abort_code(), Some(5));

    // Grant the DR role
    executor.exec(
        "Roles",
        "grant_diem_root_role",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    // Now initialize, it should all succeed.
    executor.exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    // Second time you try though you'll get an already published error with EMODIFY_CAPABILITY
    // reason.
    let output = executor.try_exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    assert_eq!(output.unwrap_err().move_abort_code(), Some(262));
}

#[test]
fn test_diem_initialize_tc_account() {
    let mut executor = FakeExecutor::stdlib_only_genesis();

    // DR doesn't have role yet, so role check will fail
    let output = executor.try_exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );
    assert_eq!(output.unwrap_err().move_abort_code(), Some(5));

    // Grant the DR role
    executor.exec(
        "Roles",
        "grant_diem_root_role",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    // Grant the TC role
    executor.exec(
        "Roles",
        "grant_treasury_compliance_role",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::treasury_compliance_account_address()),
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    // Try to initialize, invalid sender so role check will fail
    let output = executor.try_exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![MoveValue::Signer(
            account_config::treasury_compliance_account_address(),
        )]),
    );

    assert_eq!(output.unwrap_err().move_abort_code(), Some(2));

    // Now initialize, it should all succeed.
    executor.exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    // Second time you try though you'll get an already published error with EMODIFY_CAPABILITY
    // reason.
    let output = executor.try_exec(
        "Diem",
        "initialize",
        vec![],
        serialize_values(&vec![MoveValue::Signer(
            account_config::treasury_compliance_account_address(),
        )]),
    );

    assert_eq!(output.unwrap_err().move_abort_code(), Some(2));
}

#[test]
fn test_diem_timestamp_time_has_started() {
    let mut executor = FakeExecutor::stdlib_only_genesis();
    let account_address = AccountAddress::random();

    // Invalid address used to call `DiemTimestamp::set_time_has_started`
    let output = executor.try_exec(
        "DiemTimestamp",
        "set_time_has_started",
        vec![],
        serialize_values(&vec![MoveValue::Signer(account_address)]),
    );
    assert_eq!(output.unwrap_err().move_abort_code(), Some(2));

    executor.exec(
        "DiemTimestamp",
        "set_time_has_started",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    let output = executor.try_exec(
        "DiemTimestamp",
        "set_time_has_started",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    assert_eq!(output.unwrap_err().move_abort_code(), Some(1));
}

#[test]
fn test_diem_block_double_init() {
    let mut executor = FakeExecutor::stdlib_only_genesis();

    executor.exec(
        "DiemBlock",
        "initialize_block_metadata",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    let output = executor.try_exec(
        "DiemBlock",
        "initialize_block_metadata",
        vec![],
        serialize_values(&vec![
            MoveValue::Signer(account_config::diem_root_address()),
        ]),
    );

    assert_eq!(output.unwrap_err().move_abort_code(), Some(6));
}
