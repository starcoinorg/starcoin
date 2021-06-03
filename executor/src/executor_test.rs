// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{create_account_txn_sent_as_association, peer_to_peer_txn};
use crate::{encode_create_account_script_function, Account};
use anyhow::anyhow;
use anyhow::Result;
use logger::prelude::*;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_transaction_builder::{DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::{RawUserTransaction, ScriptFunction};
use starcoin_types::{
    account_config, block_metadata::BlockMetadata, transaction::Transaction,
    transaction::TransactionPayload, transaction::TransactionStatus,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::on_chain_config::{ConsensusConfig, OnChainConfig};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::token::stc::stc_type_tag;
use starcoin_vm_types::value::{serialize_values, MoveValue};
use starcoin_vm_types::values::VMValueCast;
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::{transaction::Package, vm_status::StatusCode};
use test_helper::executor::{
    account_execute, account_execute_should_success, association_execute_should_success,
    blockmeta_execute, build_raw_txn, current_block_number, TEST_MODULE, TEST_MODULE_1,
    TEST_MODULE_2,
};

use test_helper::executor::{
    compile_modules_with_address, execute_and_apply, get_balance, get_sequence_number,
    prepare_genesis,
};
// use test_helper::Account;
use starcoin_vm_types::account_config::core_code_address;
use vm_runtime::starcoin_vm::StarcoinVM;

#[derive(Default)]
pub struct NullStateView;

impl StateView for NullStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Err(anyhow!("No data"))
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        Err(anyhow!("No data"))
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

#[stest::test]
fn test_vm_version() {
    let (chain_state, _net) = prepare_genesis();

    let mut vm = StarcoinVM::new();
    let version_module_id = ModuleId::new(genesis_address(), Identifier::new("Version").unwrap());
    let mut read_version = vm
        .execute_readonly_function(
            &chain_state,
            &version_module_id,
            &Identifier::new("get").unwrap(),
            vec![],
            serialize_values(&vec![MoveValue::Address(genesis_address())]),
        )
        .unwrap();
    let readed_version: u64 = read_version.pop().unwrap().1.cast().unwrap();
    let version = vm.get_version().unwrap().major;
    assert_eq!(readed_version, version);
}

#[stest::test]
fn test_consensus_config_get() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let mut vm = StarcoinVM::new();
    let module_id = ModuleId::new(
        genesis_address(),
        Identifier::new("ConsensusConfig").unwrap(),
    );
    let mut read_config = vm.execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("get_config").unwrap(),
        vec![],
        serialize_values(&vec![]),
    )?;
    let annotator = MoveValueAnnotator::new(&chain_state);
    let (t, v) = read_config.pop().unwrap();
    let layout = annotator.type_tag_to_type_layout(&t)?;
    let r = v
        .simple_serialize(&layout)
        .ok_or_else(|| anyhow::format_err!("fail to serialize contract result"))?;
    let config = ConsensusConfig::deserialize_into_config(r.as_slice())?;
    assert_eq!(config.strategy, 0);
    Ok(())
}

#[stest::test]
fn test_batch_transfer() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let bob = Account::new();
    let mut addresses = vec![];
    let mut auth_keys = vec![];
    addresses.push(*alice.address());
    addresses.push(*bob.address());
    auth_keys.push(alice.auth_key().to_vec());
    auth_keys.push(bob.auth_key().to_vec());

    (1..30).for_each(|_| {
        let account = Account::new();
        addresses.push(*account.address());
        auth_keys.push(account.auth_key().to_vec());
    });

    let len = addresses.len();
    let addresses = MoveValue::Vector(addresses.into_iter().map(MoveValue::Address).collect());
    let auth_keys = MoveValue::Vector(auth_keys.into_iter().map(MoveValue::vector_u8).collect());
    let amounts = MoveValue::Vector((0..len).map(|_| MoveValue::U128(1)).collect());

    let script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("batch_peer_to_peer").unwrap(),
        vec![stc_type_tag()],
        vec![
            addresses.simple_serialize().unwrap(),
            auth_keys.simple_serialize().unwrap(),
            amounts.simple_serialize().unwrap(),
        ],
    );
    association_execute_should_success(
        &net,
        &chain_state,
        TransactionPayload::ScriptFunction(script_function),
    )?;
    Ok(())
}

#[stest::test]
fn test_txn_verify_err_case() -> Result<()> {
    let (_chain_state, net) = prepare_genesis();
    let mut vm = StarcoinVM::new();
    let alice = Account::new();
    let bob = Account::new();
    let script_function = encode_create_account_script_function(
        net.stdlib_version(),
        stc_type_tag(),
        alice.address(),
        alice.auth_key(),
        100000,
    );
    let txn = RawUserTransaction::new_with_default_gas_token(
        *alice.address(),
        0,
        TransactionPayload::ScriptFunction(script_function),
        10000000,
        1,
        1000 + 60 * 60,
        ChainId::test(),
    );

    let signed_by_bob = bob.sign_txn(txn);
    let verify_result = vm.verify_transaction(&NullStateView, signed_by_bob);
    assert!(verify_result.is_some());
    assert_eq!(
        verify_result.unwrap().status_code(),
        StatusCode::VM_STARTUP_FAILURE
    );
    Ok(())
}

#[stest::test(timeout = 360)]
fn test_package_txn() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = test_helper::Account::new();
    let bob = Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;

    // create alice, bob accounts
    {
        let script_function = encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            alice.address(),
            alice.auth_key(),
            pre_mint_amount / 4,
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;

        let script_function = encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            bob.address(),
            bob.auth_key(),
            pre_mint_amount / 4,
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }

    // verify package txn
    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;
        // let package_hash = package.crypto_hash();

        let mut vm = StarcoinVM::new();
        let txn = alice.sign_txn(build_raw_txn(
            *alice.address(),
            &chain_state,
            TransactionPayload::Package(package.clone()),
            net.chain_id(),
        ));
        let verify_result = vm.verify_transaction(&chain_state, txn);
        assert!(verify_result.is_none());
        // execute the package txn
        account_execute_should_success(
            &net,
            &alice,
            &chain_state,
            TransactionPayload::Package(package),
        )
        .unwrap();
    }

    // now, upgrade to test module_1
    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE_1)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;
        let mut vm = StarcoinVM::new();
        let txn = alice.sign_txn(build_raw_txn(
            *alice.address(),
            &chain_state,
            TransactionPayload::Package(package),
            net.chain_id(),
        ));
        let verify_result = vm.verify_transaction(&chain_state, txn);
        assert!(verify_result.is_some());
        assert_eq!(
            verify_result.unwrap().status_code(),
            StatusCode::BACKWARD_INCOMPATIBLE_MODULE_UPDATE
        );
    }

    // now, upgrade the test module
    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE_2)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;
        let mut vm = StarcoinVM::new();
        let txn = alice.sign_txn(build_raw_txn(
            *alice.address(),
            &chain_state,
            TransactionPayload::Package(package.clone()),
            net.chain_id(),
        ));
        let verify_result = vm.verify_transaction(&chain_state, txn);
        assert!(verify_result.is_none());
        // execute the package txn
        account_execute_should_success(
            &net,
            &alice,
            &chain_state,
            TransactionPayload::Package(package),
        )
        .unwrap();
    }

    Ok(())
}

#[stest::test(timeout = 360)]
fn test_wrong_package_address() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = test_helper::Account::new();
    let bob = test_helper::Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;

    // create alice, bob accounts
    {
        let script_function = encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            alice.address(),
            alice.auth_key(),
            pre_mint_amount / 4,
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;

        let script_function = encode_create_account_script_function(
            net.stdlib_version(),
            stc_type_tag(),
            bob.address(),
            bob.auth_key(),
            pre_mint_amount / 4,
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }

    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;

        // execute the package txn
        let output = account_execute(
            &net,
            &bob, // sender is bob, not package address alice
            &chain_state,
            TransactionPayload::Package(package),
        )?;
        // MODULE_ADDRESS_DOES_NOT_MATCH_SENDER is converted to UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
        assert_eq!(
            &TransactionStatus::Discard(StatusCode::UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION),
            output.status()
        );
    }

    Ok(())
}

#[stest::test(timeout = 200)]
fn test_block_execute_gas_limit() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let sequence_number1 = get_sequence_number(account_config::association_address(), &chain_state);
    let account1 = Account::new();
    {
        let miner = Account::new();
        let block_meta = BlockMetadata::new(
            starcoin_crypto::HashValue::random(),
            net.time_service().now_millis(),
            *miner.address(),
            Some(miner.auth_key()),
            0,
            current_block_number(&chain_state) + 1,
            net.chain_id(),
            0,
        );
        blockmeta_execute(&chain_state, block_meta)?;

        let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
            &account1,
            sequence_number1,
            50_000_000,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            &net,
        ));
        let output = execute_and_apply(&chain_state, txn1);
        info!("output: {:?}", output.gas_used());
    }

    net.time_service().sleep(1000);

    // pre-run a txn to get gas_used
    // transferring to an non-exists account uses about 700 gas.
    let transfer_txn_gas = {
        let txn = Transaction::UserTransaction(peer_to_peer_txn(
            &account1,
            &Account::new(),
            0,
            10_000,
            net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
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
        net.time_service().now_millis(),
        *account1.address(),
        Some(account1.auth_key()),
        0,
        current_block_number(&chain_state) + 1,
        net.chain_id(),
        0,
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
                    net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
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

    net.time_service().sleep(1000);

    let block_meta2 = BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.time_service().now_millis(),
        *account1.address(),
        Some(account1.auth_key()),
        0,
        current_block_number(&chain_state) + 1,
        net.chain_id(),
        0,
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
                    net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
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
    let txn = create_account_txn_sent_as_association(&account1, 10000, 50_000_000, 1, &net);
    let output = crate::validate_transaction(&chain_state, txn);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_validate_sequence_number_too_old() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let account1 = Account::new();
    let txn1 = create_account_txn_sent_as_association(&account1, 0, 50_000_000, 1, &net);
    let output1 = execute_and_apply(&chain_state, Transaction::UserTransaction(txn1));
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    let txn2 = create_account_txn_sent_as_association(&account1, 0, 50_000_000, 1, &net);
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
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );
    let txn2 = account1.sign_txn(raw_txn);
    let output = crate::validate_transaction(&chain_state, txn2);
    assert_eq!(output, None);
    Ok(())
}

#[stest::test]
fn test_validate_txn_chain_id() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        ChainId::new(123), //wrong chain id
    );
    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn));
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::BAD_CHAIN_ID),
        *output2.status()
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
        &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let sequence_number2 = get_sequence_number(*account1.address(), &chain_state);
    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("peer_to_peer").unwrap(),
        vec![],
        vec![],
    ));
    let txn2 = Transaction::UserTransaction(account1.create_signed_txn_with_args(
        payload,
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
        &net,
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
        &net,
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
        Some(account.auth_key()),
        0,
        1000,
        1,
        &net,
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
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
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
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let txn2 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    )));

    let txn3 = Transaction::UserTransaction(account1.sign_txn(crate::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        Some(account2.auth_key()),
        1,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
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
        Some(account.auth_key()),
        old_sequence_number,
        1000,
        1,
        &net,
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
        Some(account.auth_key()),
        0,
        1000,
        1,
        &net,
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
        &account1, 0, 50_000_000, 1, &net,
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
    let compiled_module = compile_modules_with_address(*account1.address(), module_source)
        .pop()
        .unwrap();

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
    let compiled_module = compile_modules_with_address(*account1.address(), module_source)
        .pop()
        .unwrap();

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
        net.time_service().sleep(1000);
        let txn = Transaction::BlockMetadata(BlockMetadata::new(
            starcoin_crypto::HashValue::random(),
            net.time_service().now_millis(),
            *account1.address(),
            Some(account1.auth_key()),
            0,
            i + 1,
            net.chain_id(),
            0,
        ));
        let output = execute_and_apply(&chain_state, txn);
        assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    }

    let balance = get_balance(*account1.address(), &chain_state);

    assert!(balance > 0);

    Ok(())
}
