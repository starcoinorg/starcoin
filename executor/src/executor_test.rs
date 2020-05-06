// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{executor::Executor, TransactionExecutor};
use anyhow::Result;
use compiler::Compiler;
use logger::prelude::*;
use once_cell::sync::Lazy;
use starcoin_config::ChainNetwork;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::AccountResource,
    account_config::BalanceResource,
    block_metadata::BlockMetadata,
    transaction::Transaction,
    transaction::TransactionStatus,
    transaction::{Module, TransactionPayload},
    vm_error::{StatusCode, VMStatus},
};
use vm_runtime::type_tag_parser::parse_type_tags;
use vm_runtime::{
    account::Account,
    common_transactions::{create_account_txn_sent_as_association, peer_to_peer_txn},
};

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(
        VMStatus::new(StatusCode::ABORTED).with_sub_status(StatusCode::REJECTED_WRITE_SET.into()),
    )
});

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

pub fn compile_module_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
) -> TransactionPayload {
    let addr = address.clone().into();
    let compiler = Compiler {
        address: addr,
        ..Compiler::default()
    };
    TransactionPayload::Module(Module::new(
        compiler.into_module_blob(file_name, code).unwrap(),
    ))
}

#[stest::test]
fn test_publish_module() -> Result<()> {
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

    let program = String::from(
        "
        module M {

        }
        ",
    );
    // compile with account 1's address
    let compiled_module = compile_module_with_address(account1.address(), "file_name", &program);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        compiled_module.into(),
        0,
        100_000,
        1,
        account_config::starcoin_type_tag().into(),
    ));

    let output = Executor::execute_transaction(&chain_state, txn).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output.status());

    for _i in 0..10 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let txn2 = Transaction::BlockMetadata(BlockMetadata::new(
            crypto::HashValue::zero(),
            timestamp,
            account1.address().clone(),
            Some(account1.auth_key_prefix()),
        ));

        let output2 = Executor::execute_transaction(&chain_state, txn2).unwrap();
        assert_eq!(KEEP_STATUS.clone(), *output2.status());

        let balance = get_balance(account1.address().clone(), &chain_state);
        debug!("balance= {:?}", balance);

        let token = String::from("0x0::Starcoin::T");
        let token_balance =
            get_token_balance(account1.address().clone(), &chain_state, token)?.unwrap();
        assert_eq!(balance, token_balance);
    }

    Ok(())
}

fn get_token_balance(
    address: AccountAddress,
    state_db: &dyn ChainStateReader,
    token: String,
) -> Result<Option<u64>> {
    let account_state_reader = AccountStateReader::new(state_db);
    let type_tag = parse_type_tags(token.as_ref())?[0].clone().into();
    debug!("type_tag= {:?}", type_tag);
    account_state_reader.get_token_balance(&address, &type_tag)
}
