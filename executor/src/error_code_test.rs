// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{create_account_txn_sent_as_association, Account};
use anyhow::Result;
use once_cell::sync::Lazy;
use starcoin_transaction_builder::{DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_types::{
    block_metadata::BlockMetadata, transaction::Transaction, transaction::TransactionStatus,
};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::vm_status::StatusCode;
use std::str::FromStr;
use test_helper::executor::{execute_and_apply, prepare_genesis};

pub static WRONG_TOKEN_CODE_FOR_TEST: Lazy<TokenCode> = Lazy::new(|| {
    TokenCode::from_str("0x1::ABC::ABC").expect("Parse wrong token code should success.")
});

#[stest::test]
fn test_block_metadata_error_code() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let account1 = Account::new();

    net.time_service().sleep(1000);
    let txn_normal = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.time_service().now_millis(),
        *account1.address(),
        Some(account1.auth_key()),
        0,
        1,
        net.chain_id(),
        0,
    ));
    let output_normal = execute_and_apply(&chain_state, txn_normal);
    assert_eq!(
        TransactionStatus::Keep(KeptVMStatus::Executed),
        *output_normal.status()
    );

    net.time_service().sleep(1000);
    let txn1 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.time_service().now_millis(),
        *account1.address(),
        Some(account1.auth_key()),
        0,
        3, //EBLOCK_NUMBER_MISMATCH
        net.chain_id(),
        0,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION),
        *output1.status()
    );

    net.time_service().sleep(1000);
    let txn2 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        0, //EINVALID_TIMESTAMP
        *account1.address(),
        Some(account1.auth_key()),
        0,
        2,
        net.chain_id(),
        0,
    ));
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION),
        *output2.status()
    );

    net.time_service().sleep(1000);
    let txn3 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.time_service().now_millis(),
        *account1.address(),
        Some(account1.auth_key()),
        net.genesis_config()
            .consensus_config
            .base_max_uncles_per_block
            + 1, //MAX_UNCLES_PER_BLOCK_IS_WRONG
        2,
        net.chain_id(),
        0,
    ));
    let output3 = execute_and_apply(&chain_state, txn3);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION),
        *output3.status()
    );

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn_with_wrong_token_code() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn_by_token_type(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        WRONG_TOKEN_CODE_FOR_TEST.clone(),
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );

    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn));
    let output = crate::execute_transactions(&chain_state, vec![txn2]).unwrap();
    assert_eq!(
        KeptVMStatus::MiscellaneousError,
        output[0].status().status().unwrap()
    );

    Ok(())
}
