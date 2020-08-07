// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_test::{execute_and_apply, prepare_genesis};
use anyhow::Result;
use starcoin_consensus::Consensus;
use starcoin_functional_tests::account::Account;
use starcoin_types::{
    block_metadata::BlockMetadata, transaction::Transaction, transaction::TransactionStatus,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::vm_status::StatusCode;

#[stest::test]
fn test_block_metadata_error_code() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let account1 = Account::new();

    net.consensus().time().sleep(1);
    let txn_normal = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.consensus().now(),
        *account1.address(),
        Some(account1.auth_key_prefix()),
        0,
        1,
    ));
    let output_normal = execute_and_apply(&chain_state, txn_normal);
    assert_eq!(
        TransactionStatus::Keep(KeptVMStatus::Executed),
        *output_normal.status()
    );

    net.consensus().time().sleep(1);
    let txn1 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.consensus().now(),
        *account1.address(),
        Some(account1.auth_key_prefix()),
        0,
        3, //EBLOCK_NUMBER_MISMATCH
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNKNOWN_STATUS),
        *output1.status()
    );

    net.consensus().time().sleep(1);
    let txn2 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        0, //EINVALID_TIMESTAMP
        *account1.address(),
        Some(account1.auth_key_prefix()),
        0,
        2,
    ));
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNKNOWN_STATUS),
        *output2.status()
    );

    net.consensus().time().sleep(1);
    let txn3 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.consensus().now(),
        *account1.address(),
        Some(account1.auth_key_prefix()),
        net.get_config().max_uncles_per_block + 1, //MAX_UNCLES_PER_BLOCK_IS_WRONG
        2,
    ));
    let output3 = execute_and_apply(&chain_state, txn3);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNKNOWN_STATUS),
        *output3.status()
    );

    Ok(())
}
