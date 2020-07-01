// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use logger::prelude::*;
use once_cell::sync::Lazy;
use starcoin_config::ChainNetwork;
use starcoin_functional_tests::account::{
    create_account_txn_sent_as_association, peer_to_peer_txn, Account,
};
use starcoin_genesis::Genesis;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use starcoin_transaction_builder::{
    build_upgrade_package, create_signed_txn_with_association_account,
};
use starcoin_types::language_storage::CORE_CODE_ADDRESS;
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
use statedb::ChainStateDB;
use std::time::{SystemTime, UNIX_EPOCH};
use stdlib::StdLibOptions;

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(
        VMStatus::new(StatusCode::ABORTED).with_sub_status(StatusCode::REJECTED_WRITE_SET.into()),
    )
});

fn prepare_genesis() -> ChainStateDB {
    prepare_genesis_with_chain_net(ChainNetwork::Dev)
}

fn prepare_genesis_with_chain_net(net: ChainNetwork) -> ChainStateDB {
    let chain_state = ChainStateDB::mock();
    let genesis_txn = Genesis::build_genesis_transaction(net).unwrap();
    Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    chain_state
}

fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = crate::execute_transactions(chain_state, vec![txn])
        .unwrap()
        .pop()
        .expect("Output must exist.");
    if let TransactionStatus::Keep(_) = output.status() {
        chain_state
            .apply_write_set(output.write_set().clone())
            .expect("apply write_set should success.");
    }

    output
}

#[stest::test]
fn test_block_execute_gas_limit() -> Result<()> {
    let chain_state = prepare_genesis();
    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    // create account uses about 26w gas.
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1, // fix me
        50_000_000,
    ));
    let output = execute_and_apply(&chain_state, txn1);
    info!("output: {:?}", output.gas_used());

    let block_meta = BlockMetadata::new(
        crypto::HashValue::random(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        *account1.address(),
        Some(account1.auth_key_prefix()),
    );

    // pre-run a txn to get gas_used
    // transferring to an non-exists account uses about 30w gas.
    let transfer_txn_gas = {
        let txn =
            Transaction::UserTransaction(peer_to_peer_txn(&account1, &Account::new(), 0, 10_000));
        crate::execute_transactions(&chain_state, vec![txn])
            .unwrap()
            .pop()
            .expect("Output must exist.")
            .gas_used()
    };

    let block_gas_limit = 1_000_000;
    let max_include_txn_num: u64 = block_gas_limit / transfer_txn_gas;
    {
        let mut txns = (0u64..max_include_txn_num)
            .map(|seq_number| {
                Transaction::UserTransaction(peer_to_peer_txn(
                    &account1,
                    &Account::new(),
                    seq_number,
                    10_000,
                ))
            })
            .collect::<Vec<_>>();

        assert_eq!(max_include_txn_num, txns.len() as u64);

        txns.insert(0, Transaction::BlockMetadata(block_meta.clone()));
        let (_, txn_infos) = crate::block_execute(&chain_state, txns, block_gas_limit)?;

        // all user txns can be included
        assert_eq!(txn_infos.len() as u64, max_include_txn_num + 1);
        let block_gas_used = txn_infos.iter().fold(0u64, |acc, i| acc + i.gas_used());
        assert!(
            block_gas_used <= block_gas_limit,
            "block_gas_used is bigger than block_gas_limit"
        );
    }

    let latest_seq_number = max_include_txn_num;

    {
        let mut txns: Vec<Transaction> = (0..max_include_txn_num * 2)
            .map(|i| {
                let seq_number = i + latest_seq_number;
                Transaction::UserTransaction(peer_to_peer_txn(
                    &account1,
                    &Account::new(),
                    seq_number,
                    10_000,
                ))
            })
            .collect();
        txns.insert(0, Transaction::BlockMetadata(block_meta));
        let (_, txn_infos) = crate::block_execute(&chain_state, txns, block_gas_limit)?;

        // not all user txns can be included
        assert_eq!(txn_infos.len() as u64, max_include_txn_num + 1);
        let block_gas_used = txn_infos.iter().fold(0u64, |acc, i| acc + i.gas_used());
        assert!(
            block_gas_used <= block_gas_limit,
            "block_gas_used is bigger than block_gas_limit"
        );
    }

    Ok(())
}

#[stest::test]
fn test_validate_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        0,
        1000,
        1,
        crate::TXN_RESERVED,
    );
    let txn2 = account1.sign_txn(raw_txn);
    let output = crate::validate_transaction(&chain_state, txn2);
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
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.auth_key_prefix(),
        0,
        1000,
    );
    let output = crate::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn_with_starcoin_vm() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        0,
        1000,
        1,
        crate::TXN_RESERVED,
    );

    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn));
    let output = crate::execute_transactions(&chain_state, vec![txn2]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_execute_multi_txn_with_same_account() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let account2 = Account::new();

    let txn2 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        0,
        1000,
        1,
        crate::TXN_RESERVED,
    )));

    let txn3 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.auth_key_prefix(),
        1,
        1000,
        1,
        crate::TXN_RESERVED,
    )));

    let output = crate::execute_transactions(&chain_state, vec![txn2, txn3]).unwrap();
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
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.auth_key_prefix(),
        old_sequence_number,
        1000,
    );
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
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.auth_key_prefix(),
        0,
        1000,
    );
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

fn compile_module_with_address(address: AccountAddress, code: &str) -> Module {
    let compiled_result =
        starcoin_move_compiler::compile_source_string(code, &[], address).expect("compile fail");
    Module::new(compiled_result.serialize())
}

#[stest::test]
fn test_publish_module() -> Result<()> {
    let chain_state = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KEEP_STATUS.clone(), *output1.status());

    let program = r#"
        module M {

        }
        "#;
    // compile with account 1's address
    let compiled_module = compile_module_with_address(*account1.address(), program);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Module(compiled_module),
        0,
        100_000,
        1,
    ));

    let output = crate::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KEEP_STATUS.clone(), *output[0].status());

    Ok(())
}

#[stest::test]
fn test_block_metadata() -> Result<()> {
    let chain_config = ChainNetwork::Dev.get_config();
    let chain_state = prepare_genesis_with_chain_net(ChainNetwork::Dev);

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

    let token = String::from("0x1::STC::STC");
    let token_balance = get_token_balance(*account1.address(), &chain_state, token)?.unwrap();
    assert_eq!(balance, token_balance);

    Ok(())
}

#[stest::test]
fn test_stdlib_upgrade() -> Result<()> {
    let chain_net = ChainNetwork::Dev;
    let chain_state = prepare_genesis_with_chain_net(chain_net);

    let mut upgrade_package = build_upgrade_package(chain_net, StdLibOptions::Fresh, false)?;

    let program = r#"
        module M {
            public fun hello(){
            }
        }
        "#;
    let module = compile_module_with_address(CORE_CODE_ADDRESS, program);
    upgrade_package.add_module(module);

    let txn = create_signed_txn_with_association_account(
        TransactionPayload::Package(upgrade_package),
        0,
        50_000_000,
        1,
    );
    let output = execute_and_apply(&chain_state, Transaction::UserTransaction(txn));
    assert_eq!(KEEP_STATUS.clone(), *output.status());

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
    account_state_reader.get_balance_by_type(&address, &type_tag)
}
