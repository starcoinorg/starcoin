// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::{mock_txn, Executor},
    mock_executor::{
        get_signed_txn, MockChainState, MockExecutor,
    },
    TransactionExecutor,
};
use config::VMConfig;
use crypto::ed25519::compat;
use logger::prelude::*;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    transaction::{SignedUserTransaction, Transaction},
};
use vm_runtime::{
    account::AccountData,
    mock_vm::{encode_mint_transaction, encode_transfer_program, encode_transfer_transaction,
              DISCARD_STATUS, KEEP_STATUS,
    }

};
use std::time::Duration;

fn gen_address(index: u8) -> AccountAddress {
    AccountAddress::new([index; ADDRESS_LENGTH])
}



#[test]
fn test_execute_mint_txn() {
    let chain_state = MockChainState::new();
    let mut executor = MockExecutor::new();
    let sender = AccountData::new(1_000_000, 10);
    executor.add_account_data(&sender, &chain_state);
    info!("create account: {:?}", sender.account().address());
    let txn = encode_mint_transaction(sender.account().address().clone(), 100);
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output = MockExecutor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

#[test]
fn test_execute_multiple_mint_txns() {
    for i in 0..10 {
        let chain_state = MockChainState::new();
        let mut executor = MockExecutor::new();
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender, &chain_state);
        info!("create account: {:?}", sender.account().address());
        let txn = encode_mint_transaction(sender.account().address().clone(), 100);
        let config = VMConfig::default();
        info!("invoke Executor::execute_transaction");
        let output = MockExecutor::execute_transaction(&config, &chain_state, txn).unwrap();

        assert_eq!(KEEP_STATUS.clone(), *output.status());
    }
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
    //let txn = encode_mint_transaction(gen_address(0), 100);
    let txn = mock_txn();
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output = Executor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

