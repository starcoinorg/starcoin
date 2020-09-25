// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::test_helper::{
    compile_module_with_address, execute_and_apply, get_balance, get_sequence_number,
    prepare_genesis,
};
use anyhow::Result;
use logger::prelude::*;
use starcoin_config::ChainNetwork;
use starcoin_consensus::Consensus;
use starcoin_functional_tests::account::{
    create_account_txn_sent_as_association, peer_to_peer_txn, Account,
};
use starcoin_transaction_builder::{StdlibScript, DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_types::{
    account_config, block_metadata::BlockMetadata, transaction::Transaction,
    transaction::TransactionPayload, transaction::TransactionStatus,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::{transaction::Package, vm_status::StatusCode};
use stdlib::transaction_scripts::compiled_transaction_script;

#[stest::test(timeout = 200)]
fn test_block_execute_gas_limit() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1,
        50_000_000,
        net.consensus().now() + DEFAULT_EXPIRATION_TIME,
        net,
    ));
    let output = execute_and_apply(&chain_state, txn1);
    info!("output: {:?}", output.gas_used());
    net.consensus().time().sleep(1);

    // pre-run a txn to get gas_used
    // transferring to an non-exists account uses about 700 gas.
    let transfer_txn_gas = {
        let txn = Transaction::UserTransaction(peer_to_peer_txn(
            &account1,
            &Account::new(),
            0,
            10_000,
            net.consensus().now() + DEFAULT_EXPIRATION_TIME,
            net.chain_id(),
        ));
        crate::execute_transactions(&chain_state, vec![txn])
            .unwrap()
            .pop()
            .expect("Output must exist.")
            .gas_used()
    };
    assert!(
        transfer_txn_gas > 0,
        "transfer_txn_gas used must not be zero."
    );

    let block_meta = BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.consensus().now(),
        *account1.address(),
        Some(account1.clone().pubkey),
        0,
        1,
        net.chain_id(),
    );
    let block_gas_limit = 10_000;
    let max_include_txn_num: u64 = block_gas_limit / transfer_txn_gas;
    {
        let mut txns = (0u64..max_include_txn_num)
            .map(|seq_number| {
                Transaction::UserTransaction(peer_to_peer_txn(
                    &account1,
                    &Account::new(),
                    seq_number,
                    10_000,
                    net.consensus().now() + DEFAULT_EXPIRATION_TIME,
                    net.chain_id(),
                ))
            })
            .collect::<Vec<_>>();

        assert_eq!(max_include_txn_num, txns.len() as u64);

        txns.insert(0, Transaction::BlockMetadata(block_meta));
        let executed_data = crate::block_execute(&chain_state, txns, block_gas_limit)?;
        let txn_infos = executed_data.txn_infos;

        // all user txns can be included
        assert_eq!(txn_infos.len() as u64, max_include_txn_num + 1);
        let block_gas_used = txn_infos.iter().fold(0u64, |acc, i| acc + i.gas_used());
        assert!(
            block_gas_used <= block_gas_limit,
            "block_gas_used is bigger than block_gas_limit"
        );
    }

    let latest_seq_number = max_include_txn_num;

    net.consensus().time().sleep(1);

    let block_meta2 = BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.consensus().now(),
        *account1.address(),
        Some(account1.clone().pubkey),
        0,
        2,
        net.chain_id(),
    );

    let max_block_gas_limit = 1_000_000;
    let max_txn_num: u64 = max_block_gas_limit / transfer_txn_gas;
    let wrong_block_gas_limit = 2_000_000; //large than maxium_block_gas_limit
    let wrong_include_txn_num: u64 = wrong_block_gas_limit / transfer_txn_gas;
    {
        let mut txns: Vec<Transaction> = (0..wrong_include_txn_num)
            .map(|i| {
                let seq_number = i + latest_seq_number;
                Transaction::UserTransaction(peer_to_peer_txn(
                    &account1,
                    &Account::new(),
                    seq_number,
                    10_000,
                    net.consensus().now() + DEFAULT_EXPIRATION_TIME,
                    net.chain_id(),
                ))
            })
            .collect();
        txns.insert(0, Transaction::BlockMetadata(block_meta2));
        let txn_infos = crate::block_execute(&chain_state, txns, max_block_gas_limit)?.txn_infos;

        // not all user txns can be included
        assert_eq!(txn_infos.len() as u64, max_txn_num + 1);
        let block_gas_used = txn_infos.iter().fold(0u64, |acc, i| acc + i.gas_used());
        assert!(
            block_gas_used <= max_block_gas_limit,
            "block_gas_used is bigger than block_gas_limit"
        );
    }

    Ok(())
}

#[stest::test]
fn test_validate_sequence_number_too_new() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let account1 = Account::new();
    let txn = create_account_txn_sent_as_association(&account1, 10000, 50_000_000, 1, net);
    let output = crate::validate_transaction(&chain_state, txn);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_validate_sequence_number_too_old() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let account1 = Account::new();
    let txn1 = create_account_txn_sent_as_association(&account1, 0, 50_000_000, 1, net);
    let output1 = execute_and_apply(&chain_state, Transaction::UserTransaction(txn1));
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    let txn2 = create_account_txn_sent_as_association(&account1, 0, 50_000_000, 1, net);
    let output = crate::validate_transaction(&chain_state, txn2);
    assert!(
        output.is_some(),
        "expect validate transaction return VMStatus, but get None "
    );
    let status_code = output.unwrap().status_code();
    assert_eq!(
        status_code,
        StatusCode::SEQUENCE_NUMBER_TOO_OLD,
        "expect StatusCode SEQUENCE_NUMBER_TOO_OLD, but get: {:?}",
        status_code
    );
    Ok(())
}

#[stest::test]
fn test_validate_txn() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.pubkey.to_bytes().to_vec(),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.consensus().now() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );
    let txn2 = account1.sign_txn(raw_txn);
    let output = crate::validate_transaction(&chain_state, txn2);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_validate_txn_chain_id() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        0,
        50_000_000,
        1,
        &ChainNetwork::DEV, //wrong chain id
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::BAD_CHAIN_ID),
        *output1.status()
    );

    Ok(())
}

#[stest::test]
fn test_gas_charge_for_invalid_script_argument_txn() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1,
        50_000_000,
        1,
        net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let sequence_number2 = get_sequence_number(*account1.address(), &chain_state);
    let txn2 = Transaction::UserTransaction(account1.create_signed_txn_with_args(
        compiled_transaction_script(net.stdlib_version(), StdlibScript::PeerToPeer).into_vec(),
        vec![],
        //Do not pass any argument.
        vec![],
        sequence_number2,
        DEFAULT_MAX_GAS_AMOUNT, // this is a default for gas
        1,                      // this is a default for gas
        1,
        net.chain_id(),
    ));
    let output2 = execute_and_apply(&chain_state, txn2);
    //assert!(output3.status().vm_status().status_type());
    assert!(output2.gas_used() > 0, "gas used must not be zero.");
    Ok(())
}

#[stest::test]
fn test_execute_real_txn_with_starcoin_vm() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        sequence_number1, // fix me
        50_000_000,
        1,
        net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let sequence_number2 = get_sequence_number(account_config::association_address(), &chain_state);
    let account2 = Account::new();
    let txn2 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account2,
        sequence_number2, // fix me
        1_000,
        1,
        net,
    ));
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(KeptVMStatus::Executed, output2.status().status().unwrap());

    let sequence_number3 = get_sequence_number(*account1.address(), &chain_state);
    let txn3 = Transaction::UserTransaction(peer_to_peer_txn(
        &account1,
        &account2,
        sequence_number3, // fix me
        100,
        1,
        net.chain_id(),
    ));
    let output3 = execute_and_apply(&chain_state, txn3);
    assert_eq!(KeptVMStatus::Executed, output3.status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_execute_mint_txn_with_starcoin_vm() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account = Account::new();
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.pubkey.to_bytes().to_vec(),
        0,
        1000,
        1,
        net,
    );
    let output = crate::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn_with_starcoin_vm() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.pubkey.to_bytes().to_vec(),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.consensus().now() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );

    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn));
    let output = crate::execute_transactions(&chain_state, vec![txn2]).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_execute_multi_txn_with_same_account() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let txn2 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.pubkey.to_bytes().to_vec(),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.consensus().now() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    )));

    let txn3 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        account2.pubkey.to_bytes().to_vec(),
        1,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.consensus().now() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    )));

    let output = crate::execute_transactions(&chain_state, vec![txn2, txn3]).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());
    assert_eq!(KeptVMStatus::Executed, output[1].status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_sequence_number() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let old_balance = get_balance(account_config::association_address(), &chain_state);
    info!("old balance: {:?}", old_balance);

    let old_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    let account = Account::new();
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.pubkey.to_bytes().to_vec(),
        old_sequence_number,
        1000,
        1,
        net,
    );
    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

    let new_sequence_number =
        get_sequence_number(account_config::association_address(), &chain_state);

    assert_eq!(new_sequence_number, old_sequence_number + 1);

    Ok(())
}

#[stest::test]
fn test_gas_used() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account = Account::new();
    let txn = crate::build_transfer_from_association(
        *account.address(),
        account.pubkey.to_bytes().to_vec(),
        0,
        1000,
        1,
        net,
    );
    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    assert!(output.gas_used() > 0);

    Ok(())
}

#[stest::test]
fn test_publish_module_and_upgrade() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let module_source = r#"
        module M {
            public fun hello(){
            }
        }
        "#;
    // compile with account 1's address
    let compiled_module = compile_module_with_address(*account1.address(), module_source);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Package(Package::new_with_module(compiled_module).unwrap()),
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = crate::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    //upgrade, add new method.
    let module_source = r#"
        module M {
            public fun hello(){
            }
            public fun hello2(){
            }
        }
        "#;
    // compile with account 1's address
    let compiled_module = compile_module_with_address(*account1.address(), module_source);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Package(Package::new_with_module(compiled_module).unwrap()),
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = crate::execute_transactions(&chain_state, vec![txn]).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_block_metadata() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let genesis_config = net.genesis_config();

    let account1 = Account::new();

    for i in 0..genesis_config.reward_delay + 1 {
        debug!("execute block metadata: {}", i);
        net.consensus().time().sleep(1);
        let timestamp = net.consensus().now();
        let txn = Transaction::BlockMetadata(BlockMetadata::new(
            starcoin_crypto::HashValue::random(),
            timestamp,
            *account1.address(),
            Some(account1.clone().pubkey),
            0,
            i + 1,
            net.chain_id(),
        ));
        let output = execute_and_apply(&chain_state, txn);
        assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    }

    let balance = get_balance(*account1.address(), &chain_state);

    assert!(balance > 0);

    Ok(())
}
