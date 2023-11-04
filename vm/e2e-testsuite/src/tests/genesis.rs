// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::KeptVMStatus;
use starcoin_language_e2e_tests::{common_transactions::peer_to_peer_txn, executor::FakeExecutor};

use starcoin_vm_types::transaction::{Transaction, TransactionStatus};

#[test]
fn no_deletion_in_genesis() {
    // let genesis = GENESIS_CHANGE_SET.clone();
    // assert!(!genesis.write_set().iter().any(|(_, op)| op.is_deletion()))
}

#[test]
fn execute_genesis_write_set() {
    // let executor = FakeExecutor::no_genesis();
    // let txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(GENESIS_CHANGE_SET.clone()));
    // let mut output = executor.execute_transaction_block(vec![txn]).unwrap();
    //
    // // Executing the genesis transaction should succeed
    // assert_eq!(output.len(), 1);
    // assert!(!output.pop().unwrap().status().is_discarded())
    let (_, (_, transaction_info)) = FakeExecutor::from_test_genesis_with_info();
    assert_eq!(transaction_info.status().clone(), KeptVMStatus::Executed);
}

#[test]
fn execute_genesis_and_drop_other_transaction() {
    let mut executor = FakeExecutor::no_genesis();
    //let txn = Transaction::GenesisTransaction(WriteSetPayload::Direct(GENESIS_CHANGE_SET.clone()));

    let sender = executor.create_raw_account_data(1_000_000, 10);
    let receiver = executor.create_raw_account_data(100_000, 10);
    let txn2 = peer_to_peer_txn(sender.account(), receiver.account(), 11, 1000);

    let mut output = executor
        .execute_transaction_block(vec![Transaction::UserTransaction(txn2)])
        .unwrap();

    // Transaction that comes after genesis should be dropped.
    assert_eq!(output.len(), 1);
    assert_eq!(output.pop().unwrap().status(), &TransactionStatus::Retry)
}
