// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{executor::Executor, TransactionExecutor};
use anyhow::Result;
use compiler::Compiler;
use logger::prelude::*;
use once_cell::sync::Lazy;
use starcoin_config::{ChainConfig, ChainNetwork};
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use starcoin_types::transaction::TransactionOutput;
use starcoin_types::{
    account_address::AccountAddress,
    account_config,
    block_metadata::BlockMetadata,
    transaction::Transaction,
    transaction::TransactionStatus,
    transaction::{Module, TransactionPayload},
    vm_error::{StatusCode, VMStatus},
};
use starcoin_vm_types::parser;
use state_tree::mock::MockStateNodeStore;
use statedb::ChainStateDB;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use vm_runtime::{
    account::Account,
    common_transactions::{create_account_txn_sent_as_association, peer_to_peer_txn, TXN_RESERVED},
};

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(
        VMStatus::new(StatusCode::ABORTED).with_sub_status(StatusCode::REJECTED_WRITE_SET.into()),
    )
});

fn prepare_genesis() -> ChainStateDB {
    prepare_genesis_with_chain_config(ChainNetwork::Dev.get_config())
}

fn prepare_genesis_with_chain_config(chain_config: &ChainConfig) -> ChainStateDB {
    let change_set = Executor::init_genesis(chain_config).unwrap();
    let (write_set, _event) = change_set.into_inner();

    let storage = MockStateNodeStore::new();
    let chain_state = ChainStateDB::new(Arc::new(storage), None);

    chain_state
        .apply_write_set(write_set)
        .unwrap_or_else(|e| panic!("Failure to apply state set: {}", e));
    chain_state
}

fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = Executor::execute_transactions(chain_state, vec![txn])
        .unwrap()
        .pop()
        .expect("Output must exist.");
    chain_state
        .apply_write_set(output.write_set().clone())
        .expect("apply write_set should success.");
    output
}

#[stest::test]
fn test_validate_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = Executor::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        0,
        1000,
        1,
        TXN_RESERVED,
    );
    let txn2 = account1.create_user_txn_from_raw_txn(raw_txn);
    let output = Executor::validate_transaction(&chain_state, txn2);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_execute_real_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1, // fix me
        50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let sequence_number2 = get_sequence_number(account_config::association_address(), &chain_state);
    let account2 = Account::new();
    let txn2 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account2,
        sequence_number2, // fix me
        1_000,
    ));
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(KEEP_STATUS.clone(), *output2.status());

    let sequence_number3 = get_sequence_number(*account1.address(), &chain_state);
    let txn3 = Transaction::UserTransaction(peer_to_peer_txn(
        &account1,
        &account2,
        sequence_number3, // fix me
        100,
    ));
    let output3 = execute_and_apply(&chain_state, txn3);
    assert_eq!(KEEP_STATUS.clone(), *output3.status());

    Ok(())
}

#[stest::test]
fn test_execute_mint_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let account = Account::new();
    let txn = Executor::build_mint_txn(*account.address(), account.auth_key_prefix(), 1, 1000);
    let output = Executor::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = Executor::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        0,
        1000,
        1,
        TXN_RESERVED,
    );

    let txn2 = Transaction::UserTransaction(account1.create_user_txn_from_raw_txn(raw_txn));
    let output = Executor::execute_transactions(&chain_state, vec![txn2]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_execute_multi_txn_with_same_account() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let txn2 = Transaction::UserTransaction(account1.create_user_txn_from_raw_txn(
        Executor::build_transfer_txn(
            *account1.address(),
            *account2.address(),
            account2.auth_key_prefix(),
            0,
            1000,
            1,
            TXN_RESERVED,
        ),
    ));

    let txn3 = Transaction::UserTransaction(account1.create_user_txn_from_raw_txn(
        Executor::build_transfer_txn(
            *account1.address(),
            *account2.address(),
            account2.auth_key_prefix(),
            1,
            1000,
            1,
            TXN_RESERVED,
        ),
    ));

    let output = Executor::execute_transactions(&chain_state, vec![txn2, txn3]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());
    assert_eq!(KEEP_STATUS.clone(), *output[1].status());

    Ok(())
}

#[stest::test]
fn test_sequence_number() -> Result<()> {
    let chain_state = prepare_genesis();
    let old_balance = get_balance(account_config::association_address(), &chain_state);
    info!("old balance: {:?}", old_balance);

    let old_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    let account = Account::new();
    let txn = Executor::build_mint_txn(*account.address(), account.auth_key_prefix(), 1, 1000);
    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KEEP_STATUS.clone(), *output.status());

    let new_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    assert_eq!(new_sequence_number, old_sequence_number + 1);

    Ok(())
}

#[stest::test]
fn test_gas_used() -> Result<()> {
    let chain_state = prepare_genesis();

    let account = Account::new();
    let txn = Executor::build_mint_txn(*account.address(), account.auth_key_prefix(), 1, 1000);
    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KEEP_STATUS.clone(), *output.status());
    assert!(output.gas_used() > 0);

    Ok(())
}

fn get_sequence_number(addr: AccountAddress, chain_state: &dyn ChainState) -> u64 {
    let account_reader = AccountStateReader::new(chain_state.as_super());
    account_reader
        .get_account_resource(&addr)
        .expect("read account state should ok")
        .map(|res| res.sequence_number())
        .unwrap_or_default()
}

fn get_balance(address: AccountAddress, chain_state: &dyn ChainState) -> u64 {
    let account_reader = AccountStateReader::new(chain_state.as_super());
    account_reader
        .get_balance(&address)
        .expect("read balance resource should ok")
        .unwrap_or_default()
}

pub fn compile_module_with_address(
    address: &AccountAddress,
    file_name: &str,
    code: &str,
) -> TransactionPayload {
    let compiler = Compiler {
        address: *address,
        ..Compiler::default()
    };
    TransactionPayload::Module(Module::new(
        compiler.into_module_blob(file_name, code).unwrap(),
    ))
}

#[stest::test]
fn test_publish_module() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, // fix me
        50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
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
        compiled_module,
        0,
        100_000,
        1,
        account_config::stc_type_tag(),
    ));

    let output = Executor::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_block_metadata() -> Result<()> {
    let chain_config = ChainNetwork::Dev.get_config();
    let chain_state = prepare_genesis_with_chain_config(chain_config);

    let account1 = Account::new();

    for i in 0..chain_config.reward_delay + 1 {
        debug!("execute block metadata: {}", i);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let txn = Transaction::BlockMetadata(BlockMetadata::new(
            crypto::HashValue::random(),
            timestamp,
            *account1.address(),
            Some(account1.auth_key_prefix()),
        ));
        let output = execute_and_apply(&chain_state, txn);
        assert_eq!(KEEP_STATUS.clone(), *output.status());
    }

    let balance = get_balance(*account1.address(), &chain_state);

    assert!(balance > 0);

    let token = String::from("0x0::STC::T");
    let token_balance = get_token_balance(*account1.address(), &chain_state, token)?.unwrap();
    assert_eq!(balance, token_balance);

    Ok(())
}

fn get_token_balance(
    address: AccountAddress,
    state_db: &dyn ChainStateReader,
    token: String,
) -> Result<Option<u64>> {
    let account_state_reader = AccountStateReader::new(state_db);
    let type_tag = parser::parse_type_tags(token.as_ref())?[0].clone();
    debug!("type_tag= {:?}", type_tag);
    account_state_reader.get_token_balance(&address, &type_tag)
}
