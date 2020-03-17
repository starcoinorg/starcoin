// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::Executor,
    mock_executor::{
        get_signed_txn, mock_mint_txn, mock_transfer_txn, mock_txn, MockChainState, MockExecutor,
    },
    TransactionExecutor,
};
use config::VMConfig;
use crypto::ed25519::compat;
use logger::prelude::*;
use state_tree::mock::MockStateNodeStore;
use state_tree::StateNodeStore;
use statedb::ChainStateDB;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::Duration;
use stdlib::transaction_scripts::EMPTY_TXN;
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    transaction::{SignedUserTransaction, Transaction},
    vm_error::{StatusCode, VMStatus},
};
use vm_runtime::mock_vm::{
    encode_mint_transaction, encode_transfer_program, encode_transfer_transaction, DISCARD_STATUS,
    KEEP_STATUS,
};
use vm_runtime::{
    account::Account, common_transactions::create_account_txn_send_with_genesis_account,
};

#[stest::test]
fn test_execute_mint_txn() {
    let repo = Arc::new(storage::memory_storage::MemoryStorage::new());
    let chain_state =
        ChainStateDB::new(Arc::new(storage::StarcoinStorage::new(repo).unwrap()), None);
    let receiver_account_address = AccountAddress::random();
    chain_state
        .create_account(receiver_account_address)
        .unwrap_or_else(|e| panic!("Failure"));
    let txn = encode_mint_transaction(receiver_account_address, 100);
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output = MockExecutor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

#[stest::test]
fn test_execute_transfer_txn() {
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);
    let sender_account_address = AccountAddress::random();
    let receiver_account_address = AccountAddress::random();
    chain_state.create_account(sender_account_address);
    chain_state.create_account(receiver_account_address);
    info!(
        "create account: sender: {:?}, receiver: {:?}",
        sender_account_address, receiver_account_address
    );
    let mint_txn = encode_mint_transaction(sender_account_address, 10000);
    let transfer_txn =
        encode_transfer_transaction(sender_account_address, receiver_account_address, 100);
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output1 = MockExecutor::execute_transaction(&config, &chain_state, mint_txn).unwrap();
    let output2 = MockExecutor::execute_transaction(&config, &chain_state, transfer_txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output1.status());
    assert_eq!(KEEP_STATUS.clone(), *output2.status());
}

#[stest::test]
fn test_validate_txn() {
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);
    let config = VMConfig::default();
    let sender_account_address = AccountAddress::random();
    let receiver_account_address = AccountAddress::random();
    let (private_key, public_key) = compat::generate_keypair(None);
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(sender_account_address, 0, &private_key, public_key, program);
    let output = MockExecutor::validate_transaction(&config, &chain_state, txn);
    assert_eq!(
        output,
        Some(VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST))
    );

    // now we create the account
    chain_state.create_account(sender_account_address);
    chain_state.create_account(receiver_account_address);
    info!(
        "create account: sender: {:?}, receiver: {:?}",
        sender_account_address, receiver_account_address
    );
    let (private_key, public_key) = compat::generate_keypair(None);
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(sender_account_address, 0, &private_key, public_key, program);
    // validate again
    let output = MockExecutor::validate_transaction(&config, &chain_state, txn);
    assert_eq!(output, None);

    // now we execute it
    let mint_txn = encode_mint_transaction(sender_account_address, 10000);
    let transfer_txn =
        encode_transfer_transaction(sender_account_address, receiver_account_address, 100);
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output1 = MockExecutor::execute_transaction(&config, &chain_state, mint_txn).unwrap();
    let output2 = MockExecutor::execute_transaction(&config, &chain_state, transfer_txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output1.status());
    assert_eq!(KEEP_STATUS.clone(), *output2.status());

    // after execute, the seq numebr should be 2.
    // and then validate again
    let (private_key, public_key) = compat::generate_keypair(None);
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(
        sender_account_address,
        0,
        &private_key,
        public_key.clone(),
        program.clone(),
    );
    // validate again
    let output = MockExecutor::validate_transaction(&config, &chain_state, txn);
    assert_eq!(
        output,
        Some(VMStatus::new(StatusCode::SEQUENCE_NUMBER_TOO_OLD))
    );

    // use right seq number
    let txn = get_signed_txn(
        sender_account_address,
        2,
        &private_key,
        public_key.clone(),
        program.clone(),
    );
    let output = MockExecutor::validate_transaction(&config, &chain_state, txn);
    assert_eq!(output, None);
}

#[stest::test]
fn test_execute_txn_with_starcoin_vm() {
    let repo = Arc::new(storage::memory_storage::MemoryStorage::new());
    let chain_state =
        ChainStateDB::new(Arc::new(storage::StarcoinStorage::new(repo).unwrap()), None);

    let txn = mock_txn();
    let config = VMConfig::default();
    info!("invoke Executor::execute_transaction");
    let output = Executor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

fn test_generate_genesis_state_set() {
    let config = VMConfig::default();
    let (_hash, state_set) = Executor::init_genesis(&config).unwrap();
    let repo = Arc::new(storage::memory_storage::MemoryStorage::new());
    let chain_state =
        ChainStateDB::new(Arc::new(storage::StarcoinStorage::new(repo).unwrap()), None);

    chain_state.apply(state_set);
    let txn = mock_txn();
    let output = Executor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}

fn test_execute_real_txn_with_starcoin_vm() {
    let config = VMConfig::default();
    let (_hash, state_set) = Executor::init_genesis(&config).unwrap();
    let repo = Arc::new(storage::memory_storage::MemoryStorage::new());
    let chain_state =
        ChainStateDB::new(Arc::new(storage::StarcoinStorage::new(repo).unwrap()), None);

    chain_state.apply(state_set).unwrap();
    let new_account = Account::new();
    let initial_amount = 1_000;
    let txn = Transaction::UserTransaction(create_account_txn_send_with_genesis_account(
        &new_account,
        10,
        initial_amount,
    ));

    let output = Executor::execute_transaction(&config, &chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
}
