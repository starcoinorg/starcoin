// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor::Executor,
    mock_executor::{get_signed_txn, MockExecutor},
    TransactionExecutor,
};
use anyhow::Result;
use crypto::keygen::KeyGen;
use logger::prelude::*;
use starcoin_config::ChainNetwork;
use starcoin_state_api::{ChainState, ChainStateWriter};
use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;
use std::sync::Arc;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::AccountResource,
    account_config::BalanceResource,
    transaction::Transaction,
    vm_error::{StatusCode, VMStatus},
};
use vm_runtime::mock_vm::{
    encode_mint_transaction, encode_transfer_program, encode_transfer_transaction, KEEP_STATUS,
};
use vm_runtime::{
    account::Account,
    common_transactions::{create_account_txn_sent_as_association, peer_to_peer_txn},
};

#[stest::test]
fn test_execute_mint_txn() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);
    let account = Account::new();

    let receiver_account_address = account.address().clone();
    chain_state.create_account(AccountAddress::default())?;
    chain_state.create_account(receiver_account_address)?;
    let txn = MockExecutor::build_mint_txn(
        account.address().clone(),
        account.auth_key_prefix(),
        1,
        1000,
    );

    let output = MockExecutor::execute_transaction(&chain_state, txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output.status());
    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);
    let sender_account_address = AccountAddress::random();
    let receiver_account_address = AccountAddress::random();
    chain_state.create_account(sender_account_address)?;
    chain_state.create_account(receiver_account_address)?;
    let mint_txn = encode_mint_transaction(sender_account_address, 10000);
    let transfer_txn =
        encode_transfer_transaction(sender_account_address, receiver_account_address, 100);

    let output1 = MockExecutor::execute_transaction(&chain_state, mint_txn).unwrap();
    let output2 = MockExecutor::execute_transaction(&chain_state, transfer_txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output1.status());
    assert_eq!(KEEP_STATUS.clone(), *output2.status());
    Ok(())
}

#[stest::test]
fn test_validate_txn() -> Result<()> {
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    let sender_account_address = AccountAddress::random();
    let receiver_account_address = AccountAddress::random();
    let (private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(sender_account_address, 0, &private_key, public_key, program);
    let output = MockExecutor::validate_transaction(&chain_state, txn);
    assert_eq!(
        output,
        Some(VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST))
    );

    // now we create the account
    chain_state.create_account(sender_account_address)?;
    chain_state.create_account(receiver_account_address)?;
    let (private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(sender_account_address, 0, &private_key, public_key, program);
    // validate again
    let output = MockExecutor::validate_transaction(&chain_state, txn);
    assert_eq!(output, None);

    // now we execute it
    let mint_txn = encode_mint_transaction(sender_account_address, 10000);
    let transfer_txn =
        encode_transfer_transaction(sender_account_address, receiver_account_address, 100);

    let output1 = MockExecutor::execute_transaction(&chain_state, mint_txn).unwrap();
    let output2 = MockExecutor::execute_transaction(&chain_state, transfer_txn).unwrap();

    assert_eq!(KEEP_STATUS.clone(), *output1.status());
    assert_eq!(KEEP_STATUS.clone(), *output2.status());

    // after execute, the seq numebr should be 2.
    // and then validate again
    let (private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let program = encode_transfer_program(receiver_account_address, 100);
    let txn = get_signed_txn(
        sender_account_address,
        0,
        &private_key,
        public_key.clone(),
        program.clone(),
    );
    // validate again
    let output = MockExecutor::validate_transaction(&chain_state, txn);
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
    let output = MockExecutor::validate_transaction(&chain_state, txn);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_validate_txn_with_starcoin_vm() -> Result<()> {
    let (_hash, state_set) = Executor::init_genesis(ChainNetwork::Dev.get_config()).unwrap();
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply(state_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = Executor::execute_transaction(&chain_state, txn1).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = Executor::build_transfer_txn(
        account1.address().clone(),
        account1.auth_key_prefix(),
        account2.address().clone(),
        account2.auth_key_prefix(),
        0,
        1000,
    );
    let txn2 = account1.create_user_txn_from_raw_txn(raw_txn);
    let output = Executor::validate_transaction(&chain_state, txn2);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_execute_real_txn_with_starcoin_vm() -> Result<()> {
    let (_hash, state_set) = Executor::init_genesis(ChainNetwork::Dev.get_config()).unwrap();
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply(state_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));

    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1, // fix me
        50_000_000,
    ));
    let output1 = Executor::execute_transaction(&chain_state, txn1).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let sequence_number2 = get_sequence_number(account_config::association_address(), &chain_state);
    let account2 = Account::new();
    let txn2 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account2,
        sequence_number2, // fix me
        1_000,
    ));
    let output2 = Executor::execute_transaction(&chain_state, txn2).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output2.status());

    let sequence_number3 = get_sequence_number(account1.address().clone(), &chain_state);
    let txn3 = Transaction::UserTransaction(peer_to_peer_txn(
        &account1,
        &account2,
        sequence_number3, // fix me
        100,
    ));
    let output3 = Executor::execute_transaction(&chain_state, txn3).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output3.status());

    Ok(())
}

#[stest::test]
fn test_execute_mint_txn_with_starcoin_vm() -> Result<()> {
    let (_hash, state_set) = Executor::init_genesis(ChainNetwork::Dev.get_config()).unwrap();
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply(state_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));

    let account = Account::new();

    let txn = Executor::build_mint_txn(
        account.address().clone(),
        account.auth_key_prefix(),
        1,
        1000,
    );
    let output = Executor::execute_transaction(&chain_state, txn).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output.status());

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn_with_starcoin_vm() -> Result<()> {
    let (_hash, state_set) = Executor::init_genesis(ChainNetwork::Dev.get_config()).unwrap();
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply(state_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = Executor::execute_transaction(&chain_state, txn1).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = Executor::build_transfer_txn(
        account1.address().clone(),
        account1.auth_key_prefix(),
        account2.address().clone(),
        account2.auth_key_prefix(),
        0,
        1000,
    );

    let txn2 = Transaction::UserTransaction(account1.create_user_txn_from_raw_txn(raw_txn));
    let output = Executor::execute_transaction(&chain_state, txn2).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output.status());

    Ok(())
}

#[stest::test]
fn test_sequence_number() -> Result<()> {
    let (_hash, state_set) = Executor::init_genesis(ChainNetwork::Dev.get_config()).unwrap();
    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply(state_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));

    let old_balance = get_balance(account_config::association_address(), &chain_state);
    info!("old balance: {:?}", old_balance);

    let old_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    let account = Account::new();
    let txn = Executor::build_mint_txn(
        account.address().clone(),
        account.auth_key_prefix(),
        1,
        1000,
    );
    let output = Executor::execute_transaction(&chain_state, txn).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output.status());

    let new_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    assert_eq!(new_sequence_number, old_sequence_number + 1);

    Ok(())
}

fn get_sequence_number(addr: AccountAddress, chain_state: &dyn ChainState) -> u64 {
    let access_path = AccessPath::new_for_account(addr);
    let state = chain_state
        .get(&access_path)
        .expect("read account state should ok");
    match state {
        None => 0u64,
        Some(s) => AccountResource::make_from(&s)
            .expect("account resource decode ok")
            .sequence_number(),
    }
}

fn get_balance(address: AccountAddress, state_db: &dyn ChainState) -> u64 {
    let ap = AccessPath::new_for_balance(address);
    let balance_resource = state_db.get(&ap).expect("read balance resource should ok");
    match balance_resource {
        None => 0u64,
        Some(b) => BalanceResource::make_from(b.as_slice())
            .expect("decode balance resource should ok")
            .coin(),
    }
}
