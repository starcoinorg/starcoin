// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::KeptVMStatus;
use starcoin_language_e2e_tests::account::Account;
use starcoin_language_e2e_tests::common_transactions::create_account_txn;
use starcoin_language_e2e_tests::current_function_name;
use starcoin_language_e2e_tests::executor::FakeExecutor;
use starcoin_types::account_config;
use starcoin_vm_types::transaction::TransactionStatus;

#[test]
fn create_account() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    // create and publish a sender with 1_000_000 coins
    let sender = Account::new_blessed_tc();
    let new_account = executor.create_raw_account();

    // define the arguments to the create account transaction
    let initial_amount = 0;
    let txn = create_account_txn(&sender, &new_account, 0);

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
    assert_eq!(initial_amount, updated_receiver_balance.coin());
    assert_eq!(1, updated_sender.sequence_number());
}
