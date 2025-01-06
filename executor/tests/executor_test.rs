// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;
use anyhow::anyhow;
use anyhow::Result;
use forkable_jellyfish_merkle::node_type::SparseMerkleLeafNode;
use sha3::{Digest, Sha3_256};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;

use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_executor::validate_transaction;
use starcoin_logger::prelude::*;
use starcoin_state_api::{ChainStateReader, StateReaderExt};
use starcoin_transaction_builder::{
    build_batch_payload_same_amount, build_transfer_txn, encode_transfer_script_by_token_code,
    raw_peer_to_peer_txn, DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_types::account::peer_to_peer_txn;
use starcoin_types::account::Account;
use starcoin_types::account_config::G_STC_TOKEN_CODE;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, CORE_CODE_ADDRESS};
use starcoin_types::transaction::{EntryFunction, RawUserTransaction, TransactionArgument};
use starcoin_types::{
    account_config, block_metadata::BlockMetadata, transaction::Transaction,
    transaction::TransactionPayload, transaction::TransactionStatus,
};
use starcoin_vm_runtime::starcoin_vm::{chunk_block_transactions, StarcoinVM};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::account_config::AccountResource;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::TStateView;
use starcoin_vm_types::token::stc::{stc_type_tag, STCUnit};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::{
    on_chain_config::{ConsensusConfig, OnChainConfig},
    transaction::Package,
    vm_status::StatusCode,
};
use test_helper::executor::{
    account_execute, account_execute_should_success, association_execute_should_success,
    blockmeta_execute, build_raw_txn, current_block_number, prepare_customized_genesis,
    TEST_MODULE, TEST_MODULE_1, TEST_MODULE_2,
};
use test_helper::executor::{
    compile_modules_with_address, execute_and_apply, get_balance, get_sequence_number,
    prepare_genesis,
};
use test_helper::txn::create_account_txn_sent_as_association;

#[derive(Default)]
pub struct NullStateView;

impl TStateView for NullStateView {
    type Key = StateKey;
    fn get_state_value(
        &self,
        _state_key: &StateKey,
    ) -> starcoin_vm_types::state_store::Result<Option<StateValue>> {
        Err(anyhow!("No data").into())
    }

    fn get_usage(
        &self,
    ) -> starcoin_vm_types::state_store::Result<
        starcoin_vm_types::state_store::state_storage_usage::StateStorageUsage,
    > {
        unimplemented!("get_usage not implemented")
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

#[stest::test]
fn test_vm_version() {
    let (chain_state, _net) = prepare_genesis();

    let version_module_id =
        ModuleId::new(genesis_address(), Identifier::new("stc_version").unwrap());
    let mut value = starcoin_dev::playground::call_contract(
        &chain_state,
        version_module_id,
        "get",
        vec![],
        vec![TransactionArgument::Address(genesis_address())],
        None,
    )
    .unwrap();

    let readed_version: u64 = bcs_ext::from_bytes(&value.pop().unwrap().1).unwrap();
    let version = {
        let mut vm = StarcoinVM::new(None, &chain_state);
        vm.load_configs(&chain_state).unwrap();
        vm.get_version().unwrap().major
    };

    assert_eq!(readed_version, version);
}

#[stest::test]
fn test_flexidag_config_get() {
    let (chain_state, _net) = prepare_genesis();

    let version_module_id = ModuleId::new(
        genesis_address(),
        Identifier::new("flexi_dag_config").unwrap(),
    );
    let mut value = starcoin_dev::playground::call_contract(
        &chain_state,
        version_module_id,
        "effective_height",
        vec![],
        vec![TransactionArgument::Address(genesis_address())],
        None,
    )
    .unwrap();

    let read_version: u64 = bcs_ext::from_bytes(&value.pop().unwrap().1).unwrap();
    let version = {
        let mut vm = StarcoinVM::new(None, &chain_state);
        vm.load_configs(&chain_state).unwrap();
        vm.get_flexidag_config().unwrap().effective_height
    };

    assert_eq!(read_version, version);
}

#[stest::test]
fn test_flexidag_config_get_for_halley() {
    let chain_state =
        prepare_customized_genesis(&ChainNetwork::new_builtin(BuiltinNetworkID::Halley));

    let version = {
        let mut vm = StarcoinVM::new(None, &chain_state);
        vm.load_configs(&chain_state).unwrap();
        vm.get_flexidag_config().unwrap().effective_height
    };

    assert_eq!(version, 0);
}

#[stest::test]
fn test_flexidag_config_get_for_proxima() {
    let chain_state =
        prepare_customized_genesis(&ChainNetwork::new_builtin(BuiltinNetworkID::Proxima));

    let version = {
        let mut vm = StarcoinVM::new(None, &chain_state);
        vm.load_configs(&chain_state).unwrap();
        vm.get_flexidag_config().unwrap().effective_height
    };

    assert_eq!(version, 0);
}

#[stest::test]
fn test_consensus_config_get() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let module_id = ModuleId::new(
        genesis_address(),
        Identifier::new("consensus_config").unwrap(),
    );
    let mut rets = starcoin_dev::playground::call_contract(
        &chain_state,
        module_id,
        "get_config",
        vec![],
        vec![],
        None,
    )?;

    let r = rets.pop().unwrap().1;
    let config = ConsensusConfig::deserialize_into_config(r.as_slice())?;
    assert_eq!(config.strategy, 0);
    Ok(())
}

#[stest::test]
fn test_batch_transfer() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let bob = Account::new();
    let mut addresses = vec![*alice.address(), *bob.address()];

    // fixme: reduce account number to avoid OUT_OF_GAS exception
    (1..20).for_each(|_| {
        let account = Account::new();
        addresses.push(*account.address());
    });

    let payload = build_batch_payload_same_amount(addresses, 1);
    association_execute_should_success(&net, &chain_state, payload)?;
    Ok(())
}

#[stest::test]
fn test_txn_verify_err_case() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();
    let mut vm = StarcoinVM::new(None, &chain_state);
    let alice = Account::new();
    let bob = Account::new();
    let script_function =
        encode_transfer_script_by_token_code(*alice.address(), 100000, G_STC_TOKEN_CODE.clone());
    let txn = RawUserTransaction::new_with_default_gas_token(
        *alice.address(),
        0,
        script_function,
        10000000,
        1,
        1000 + 60 * 60,
        ChainId::test(),
    );

    let signed_by_bob = bob.sign_txn(txn).unwrap();
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
    let alice = Account::new();
    let bob = Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;

    // create alice, bob accounts
    {
        let script_function = encode_transfer_script_by_token_code(
            *alice.address(),
            pre_mint_amount / 4,
            G_STC_TOKEN_CODE.clone(),
        );
        association_execute_should_success(&net, &chain_state, script_function)?;

        let script_function = encode_transfer_script_by_token_code(
            *bob.address(),
            pre_mint_amount / 4,
            G_STC_TOKEN_CODE.clone(),
        );
        association_execute_should_success(&net, &chain_state, script_function)?;
    }

    // verify package txn
    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;
        // let package_hash = package.crypto_hash();

        let mut vm = StarcoinVM::new(None, &chain_state);
        let txn = alice
            .sign_txn(build_raw_txn(
                *alice.address(),
                &chain_state,
                TransactionPayload::Package(package.clone()),
                None,
            ))
            .unwrap();
        let verify_result = vm.verify_transaction(&chain_state, txn);
        assert!(verify_result.is_none());
        // execute the package txn
        account_execute_should_success(&alice, &chain_state, TransactionPayload::Package(package))
            .unwrap();
    }

    // now, upgrade to test module_1
    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE_1)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;
        let mut vm = StarcoinVM::new(None, &chain_state);
        let txn = alice
            .sign_txn(build_raw_txn(
                *alice.address(),
                &chain_state,
                TransactionPayload::Package(package),
                None,
            ))
            .unwrap();
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
        let mut vm = StarcoinVM::new(None, &chain_state);
        let txn = alice
            .sign_txn(build_raw_txn(
                *alice.address(),
                &chain_state,
                TransactionPayload::Package(package.clone()),
                None,
            ))
            .unwrap();
        let verify_result = vm.verify_transaction(&chain_state, txn);
        assert!(verify_result.is_none());
        // execute the package txn
        account_execute_should_success(&alice, &chain_state, TransactionPayload::Package(package))
            .unwrap();
    }

    Ok(())
}

#[stest::test(timeout = 360)]
fn test_wrong_package_address() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let bob = Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;

    // create alice, bob accounts
    {
        let script_function = encode_transfer_script_by_token_code(
            *alice.address(),
            pre_mint_amount / 4,
            G_STC_TOKEN_CODE.clone(),
        );
        association_execute_should_success(&net, &chain_state, script_function)?;

        let script_function = encode_transfer_script_by_token_code(
            *bob.address(),
            pre_mint_amount / 4,
            G_STC_TOKEN_CODE.clone(),
        );
        association_execute_should_success(&net, &chain_state, script_function)?;
    }

    {
        let module = compile_modules_with_address(*alice.address(), TEST_MODULE)
            .pop()
            .unwrap();
        let package = Package::new_with_module(module)?;

        // execute the package txn
        let output = account_execute(
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
        starcoin_executor::execute_transactions(&chain_state, vec![txn], None)
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
        let executed_data =
            starcoin_executor::block_execute(&chain_state, txns, block_gas_limit, None)?;
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
        let txn_infos =
            starcoin_executor::block_execute(&chain_state, txns, max_block_gas_limit, None)?
                .txn_infos;

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
    let output = validate_transaction(&chain_state, txn, None);
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
    let output = validate_transaction(&chain_state, txn2, None);
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
fn test_validate_txn_args() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let account1 = Account::new();

    let txn = {
        let action = EntryFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("transfer_scripts").unwrap(),
            ),
            Identifier::new("peer_to_peer").unwrap(),
            vec![stc_type_tag()],
            vec![],
        );
        let txn = build_raw_txn(
            *account1.address(),
            &chain_state,
            TransactionPayload::EntryFunction(action),
            None,
        );
        account1.sign_txn(txn)
    }
    .unwrap();
    assert!(validate_transaction(&chain_state, txn, None).is_some());

    let txn = {
        let action = EntryFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("transfer_scripts").unwrap(),
            ),
            Identifier::new("peer_to_peer_v2").unwrap(),
            vec![stc_type_tag()],
            vec![],
        );
        let txn = build_raw_txn(
            *account1.address(),
            &chain_state,
            TransactionPayload::EntryFunction(action),
            None,
        );
        account1.sign_txn(txn)
    }
    .unwrap();
    assert!(validate_transaction(&chain_state, txn, None).is_some());

    let txn = {
        let action = EntryFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("TransferScripts").unwrap(),
            ),
            Identifier::new("peer_to_peer_v2").unwrap(),
            vec![stc_type_tag()],
            vec![vec![0u8, 1u8]],
        );
        let txn = build_raw_txn(
            *account1.address(),
            &chain_state,
            TransactionPayload::EntryFunction(action),
            None,
        );
        account1.sign_txn(txn)
    }
    .unwrap();
    assert!(validate_transaction(&chain_state, txn, None).is_some());
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

    let raw_txn = starcoin_transaction_builder::build_transfer_txn(
        *account1.address(),
        *account2.address(),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );
    let txn2 = account1.sign_txn(raw_txn).unwrap();
    let output = validate_transaction(&chain_state, txn2, None);
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

    let raw_txn = build_transfer_txn(
        *account1.address(),
        *account2.address(),
        0,
        1000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        ChainId::new(123), //wrong chain id
    );
    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn).unwrap());
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
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("peer_to_peer_v2").unwrap(),
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
    let txn = starcoin_transaction_builder::build_transfer_from_association(
        *account.address(),
        0,
        1000,
        1,
        &net,
    );
    let output = starcoin_executor::execute_transactions(&chain_state, vec![txn], None).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    Ok(())
}

#[stest::test]
fn test_execute_transfer_txn() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1,
        0,
        STCUnit::STC.value_of(100).scaling(),
        1,
        &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let account2 = Account::new();

    let raw_txn = raw_peer_to_peer_txn(
        *account1.address(),
        *account2.address(),
        STCUnit::STC.value_of(1).scaling(),
        0,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        G_STC_TOKEN_CODE.clone(),
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );

    let txn2 = Transaction::UserTransaction(account1.sign_txn(raw_txn).unwrap());
    let output = execute_and_apply(&chain_state, txn2);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    let account_resource = chain_state.get_account_resource(*account2.address())?;

    // auth_key is empty when account create.
    assert_eq!(
        account_resource.authentication_key(),
        &AccountResource::DUMMY_AUTH_KEY
    );

    let raw_txn = raw_peer_to_peer_txn(
        *account2.address(),
        *account1.address(),
        1000,
        0,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        G_STC_TOKEN_CODE.clone(),
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );

    // account1 try to transfer stc from account2, will discard.
    let txn3 = Transaction::UserTransaction(account1.sign_txn(raw_txn.clone()).unwrap());
    let output = execute_and_apply(&chain_state, txn3);
    assert_eq!(
        StatusCode::INVALID_AUTH_KEY,
        output.status().status().err().unwrap()
    );

    let txn4 = Transaction::UserTransaction(account2.sign_txn(raw_txn).unwrap());
    let output = execute_and_apply(&chain_state, txn4);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

    let account_resource = chain_state
        .get_account_resource(*account2.address())
        .expect("account resource should exist.");

    // account2's auth_key will set in txn epilogue_v2 when execute first transaction.
    assert_eq!(
        account_resource.authentication_key(),
        account2.auth_key().to_vec().as_slice()
    );

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

    let txn2 = Transaction::UserTransaction(
        account1
            .sign_txn(starcoin_transaction_builder::build_transfer_txn(
                *account1.address(),
                *account2.address(),
                0,
                1000,
                1,
                DEFAULT_MAX_GAS_AMOUNT,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ))
            .unwrap(),
    );

    let txn3 = Transaction::UserTransaction(
        account1
            .sign_txn(starcoin_transaction_builder::build_transfer_txn(
                *account1.address(),
                *account2.address(),
                1,
                1000,
                1,
                DEFAULT_MAX_GAS_AMOUNT,
                net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            ))
            .unwrap(),
    );

    let output =
        starcoin_executor::execute_transactions(&chain_state, vec![txn2, txn3], None).unwrap();
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
    let txn = starcoin_transaction_builder::build_transfer_from_association(
        *account.address(),
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
    let txn = starcoin_transaction_builder::build_transfer_from_association(
        *account.address(),
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
        module {{sender}}::M {
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

    let output = starcoin_executor::execute_transactions(&chain_state, vec![txn], None).unwrap();
    assert_eq!(KeptVMStatus::Executed, output[0].status().status().unwrap());

    //upgrade, add new method.
    let module_source = r#"
        module {{sender}}::M {
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

    let output = starcoin_executor::execute_transactions(&chain_state, vec![txn], None).unwrap();
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

#[stest::test]
fn test_insufficient_balance_for_transaction_fee() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let alice = Account::new();
    let txn1 = starcoin_transaction_builder::build_transfer_from_association(
        *alice.address(),
        0,
        20000000,
        1,
        &net,
    );
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    assert!(output1.gas_used() > 0);

    let bob = Account::new();
    let raw_txn1 = starcoin_transaction_builder::build_transfer_txn(
        *alice.address(),
        *bob.address(),
        0,
        10000000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );
    let txn2 = Transaction::UserTransaction(alice.sign_txn(raw_txn1).unwrap());
    let output2 = execute_and_apply(&chain_state, txn2);
    assert_eq!(
        TransactionStatus::Discard(StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE),
        *output2.status()
    );

    let tom = Account::new();
    let raw_txn2 = starcoin_transaction_builder::build_transfer_txn(
        *alice.address(),
        *tom.address(),
        0,
        10000000,
        1,
        DEFAULT_MAX_GAS_AMOUNT / 4,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net.chain_id(),
    );
    let txn3 = Transaction::UserTransaction(alice.sign_txn(raw_txn2).unwrap());
    let output3 = execute_and_apply(&chain_state, txn3);
    assert_eq!(KeptVMStatus::Executed, output3.status().status().unwrap());
    assert!(output3.gas_used() > 0);

    Ok(())
}

#[test]
fn test_chunk_block_transactions() -> Result<()> {
    let (_chain_state, net) = prepare_genesis();
    let account1 = Account::new();
    let mut txns1 = vec![];
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));

    let txn2 = Transaction::BlockMetadata(BlockMetadata::new(
        starcoin_crypto::HashValue::random(),
        net.time_service().now_millis(),
        *account1.address(),
        0,
        1,
        net.chain_id(),
        0,
    ));

    let txn3 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));

    txns1.push(txn1.clone());
    let result1 = chunk_block_transactions(txns1);
    assert_eq!(result1.len(), 1);

    let txns2 = vec![txn1.clone(), txn2.clone()];
    let result2 = chunk_block_transactions(txns2);
    assert_eq!(result2.len(), 2);

    let txns3 = vec![txn1, txn2, txn3];
    let result3 = chunk_block_transactions(txns3);
    assert_eq!(result3.len(), 3);

    Ok(())
}

#[test]
fn test_get_chain_id_after_genesis_with_proof_verify() -> Result<()> {
    let (chain_state, _net) = prepare_genesis();
    let chain_id_struct_tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("chain_id").unwrap(),
        name: Identifier::new("ChainId").unwrap(),
        type_args: vec![],
    };

    let path_statekey = StateKey::resource(&CORE_CODE_ADDRESS, &chain_id_struct_tag)?;

    // Print 0x1 version resource
    let resource_value = bcs_ext::from_bytes::<ChainId>(
        &chain_state.get_resource(CORE_CODE_ADDRESS, &chain_id_struct_tag)?,
    )?;
    println!(
        "test_get_chain_id_after_genesis_with_proof_verify | path: {:?}, state_value : {:?}",
        chain_id_struct_tag, resource_value
    );
    assert_eq!(resource_value.id(), 0xff, "not expect chain id");

    // Get proof and verify proof
    let mut state_proof = chain_state.get_with_proof(&path_statekey)?;
    let proof_path = AccessPath::resource_access_path(genesis_address(), chain_id_struct_tag);
    state_proof.verify(chain_state.state_root(), proof_path.clone())?;

    state_proof.state.as_mut().unwrap()[0] = 0xFE;
    assert!(state_proof
        .verify(chain_state.state_root(), proof_path)
        .is_err());
    Ok(())
}

#[test]
fn test_sha3_256_diffrent_with_crypto_macro() -> Result<()> {
    let hash_1 = HashValue::from_hex_literal(
        "0x4cc8bd9df94b37c233555d9a3bba0a712c3c709f047486d1e624b2bcd3b83266",
    )?;
    let hash_2 = HashValue::from_hex_literal(
        "0x4f2b59b9af93b435e0a33b6ab7a8a90e471dba936be2bc2937629b7782b8ebd0",
    )?;

    let smt_hash = SparseMerkleLeafNode::new(hash_1, hash_2).crypto_hash();
    println!(
        "test_sha3_256_diffrent_with_crypto_macro | SparseMerkleLeafNode crypto hash: {:?}",
        SparseMerkleLeafNode::new(hash_1, hash_2).crypto_hash()
    );

    let mut hash_vec = Vec::new();
    hash_vec.append(&mut hash_1.to_vec());
    hash_vec.append(&mut hash_2.to_vec());
    let move_hash = HashValue::sha3_256_of(hash_vec.as_slice());
    println!(
        "test_sha3_256_diffrent_with_crypto_macro | sha3 crypto {:?}",
        HashValue::sha3_256_of(hash_vec.as_slice()),
    );

    assert_eq!(move_hash, smt_hash, "Failed to get the same hash");

    Ok(())
}
