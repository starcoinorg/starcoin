// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::StatusCode;
use starcoin_config::ChainNetwork;
use starcoin_language_e2e_tests::{
    account::{Account, AccountData, AccountRoleSpecifier},
    current_function_name,
    executor::FakeExecutor,
};

use starcoin_transaction_builder::encode_create_account_script_function;
use starcoin_vm_types::{
    account_config::stc_type_tag, transaction::authenticator::AuthenticationKey,
};

// Make sure we can start the experimental genesis
#[test]
fn experimental_genesis_runs() {
    //FakeExecutor::from_experimental_genesis();
    FakeExecutor::from_test_genesis();
}

// Make sure that we can execute transactions with the experimental genesis
#[test]
fn experimental_genesis_execute_txn_successful() {
    let mut executor = FakeExecutor::from_test_genesis();
    //executor.set_golden_file(current_function_name!());
    let net = ChainNetwork::new_test();
    let new_account = executor.create_raw_account();
    executor.add_account_data(&AccountData::with_account(
        new_account.clone(),
        100,
        0,
        AccountRoleSpecifier::Root,
    ));

    //let new_new_account = executor.create_raw_account();
    //let dr_account = Account::new_starcoin_root();
    // let txn = dr_account
    //     .transaction()
    //     .script_function(encode_create_account_script_function(
    //         net.stdlib_version(),
    //         stc_type_tag(),
    //         &new_account.address(),
    //         AuthenticationKey::try_from(new_account.auth_key()).unwrap(),
    //         0,
    //     ))
    //     .sequence_number(0)
    //     .sign();
    // executor.execute_and_apply(txn);

    let auth_key = AuthenticationKey::try_from(new_account.auth_key()).expect("Failed to encode");
    // Other accounts can create accounts, no role checks
    let txn = new_account
        .transaction()
        .script_function(encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            &Account::new().address(),
            auth_key,
            0,
        ))
        .sequence_number(0)
        .sign();
    executor.execute_and_apply(txn);
}

// Make sure that we can handle prologue errors from the non-DPN account module
#[test]
fn experimental_genesis_execute_txn_non_existent_sender() {
    let mut executor = FakeExecutor::from_test_genesis();
    // executor.set_golden_file(current_function_name!());
    let net = ChainNetwork::new_test();
    let new_account = executor.create_raw_account();
    let txn = new_account
        .transaction()
        .script_function(encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            &Account::new().address(),
            AuthenticationKey::try_from(new_account.auth_key()).unwrap(),
            0,
        ))
        .sequence_number(0)
        .sign();

    let output = &executor.execute_transaction(txn);
    assert_eq!(
        output.status().status(),
        Err(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST),
    );
}
