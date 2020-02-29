// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    mock_executor::{
        encode_mint_transaction, encode_transfer_program, encode_transfer_transaction,
        get_signed_txn, MockChainState, MockExecutor, DISCARD_STATUS, KEEP_STATUS,
    },
    TransactionExecutor,
    executor::{Executor}
};
use config::VMConfig;
use crypto::ed25519::compat;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
};
use logger::prelude::*;

fn gen_address(index: u8) -> AccountAddress {
    AccountAddress::new([index; ADDRESS_LENGTH])
}

#[test]
fn test_execute_txn() {
    let chain_state = MockChainState::new();
    let txn = encode_mint_transaction(gen_address(0), 100);
    let config = VMConfig::default();

    let output = MockExecutor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

#[test]
fn test_validate_txn() {
    let chain_state = MockChainState::new();
    let txn = encode_mint_transaction(gen_address(0), 100);
    let config = VMConfig::default();

    let (private_key, public_key) = compat::generate_keypair(None);

    let receiver = gen_address(1);
    let program = encode_transfer_program(receiver, 100);
    let txn = get_signed_txn(receiver, 1, &private_key, public_key, program);

    let output = MockExecutor::validate_transaction(&config, &chain_state, txn);

    assert_eq!(None, output);
}

#[stest::test]
fn test_execute_txn_with_starcoin_vm() {
    let chain_state = MockChainState::new();
    let txn = encode_mint_transaction(gen_address(0), 100);
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output = Executor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}
