// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::KeptVMStatus;
use starcoin_language_e2e_tests::account::{AccountData, AccountRoleSpecifier};
use starcoin_language_e2e_tests::{
    account::Account, common_transactions::create_account_txn, current_function_name,
    executor::FakeExecutor,
};
use starcoin_vm_types::transaction::TransactionStatus;

#[test]
fn create_account() {
    let mut executor = FakeExecutor::from_test_genesis();
    //executor.set_golden_file(current_function_name!());

    // create and publish a sender with 1_000_000 coins
    let sender = Account::new_blessed_tc();
    let new_account = executor.create_raw_account();

    // define the arguments to the create account transaction
    let initial_amount = 0;
    let txn = create_account_txn(&sender, &new_account, 0);

    executor.add_account_data(&AccountData::with_account(
        sender.clone(),
        initial_amount,
        0,
        AccountRoleSpecifier::Root,
    ));

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let updated_sender = executor
        .read_account_resource(&sender)
        .expect("sender must exist");

    let updated_receiver_balance = executor
        .read_balance_resource(&new_account)
        .expect("receiver balance must exist");
    assert_eq!(initial_amount, updated_receiver_balance.token());
    assert_eq!(0, updated_sender.sequence_number());
}
