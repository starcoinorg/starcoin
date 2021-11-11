// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_executor::{encode_create_account_script_function, execute_readonly_function};
use starcoin_state_api::{AccountStateReader, StateReaderExt};
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::{TransactionArgument, TransactionPayload};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::gas_schedule::{GasAlgebra, InternalGasUnits};
use starcoin_vm_types::on_chain_config::{
    consensus_config_type_tag, vm_config_type_tag, ConsensusConfig, OnChainConfig, VMConfig,
    CONSENSUS_CONFIG_IDENTIFIER,
};
use starcoin_vm_types::transaction::Transaction;
use starcoin_vm_types::value::{serialize_values, MoveValue};
use test_helper::dao::{
    dao_vote_test, empty_txn_payload, execute_script_on_chain_config, on_chain_config_type_tag,
    reward_config_type_tag, transasction_timeout_type_tag, txn_publish_config_type_tag,
    vote_reward_scripts, vote_script_consensus, vote_txn_publish_option_script,
    vote_txn_timeout_script, vote_vm_config_script,
};
use test_helper::executor::{
    account_execute_with_output, association_execute_should_success, blockmeta_execute,
    build_raw_txn, current_block_number, execute_and_apply, prepare_genesis,
};
use test_helper::Account;

#[stest::test]
fn test_modify_on_chain_consensus_config() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = consensus_config_type_tag();
    let strategy = 2u8;

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script_consensus(&net, strategy),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    //get consensus config

    let config = {
        let module_id = ModuleId::new(genesis_address(), CONSENSUS_CONFIG_IDENTIFIER.clone());
        let mut rets = starcoin_dev::playground::call_contract(
            &chain_state,
            module_id,
            "get_config",
            vec![],
            vec![],
            None,
        )?;

        let r = rets.pop().unwrap().1;
        ConsensusConfig::deserialize_into_config(r.as_slice())?
    };
    assert_eq!(config.strategy, strategy);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_reward_config() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let action_type_tag = reward_config_type_tag();
    let reward_delay: u64 = 100;

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_reward_scripts(&net, reward_delay),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    //get RewardConfig
    let module_id = ModuleId::new(genesis_address(), Identifier::new("RewardConfig")?);
    let reward_delay_on_chain: u64 = {
        let mut rets = starcoin_dev::playground::call_contract(
            &chain_state,
            module_id,
            "reward_delay",
            vec![],
            vec![],
            None,
        )?;

        let r = rets.pop().unwrap().1;
        bcs_ext::from_bytes(r.as_slice())?
    };
    assert_eq!(reward_delay_on_chain, reward_delay);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_config_txn_timeout() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let action_type_tag = transasction_timeout_type_tag();
    let new_timeout_config_seconds: u64 = 40000;

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_txn_timeout_script(&net, new_timeout_config_seconds),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    let now_seconds = chain_state.get_timestamp()?.milliseconds / 1000;
    let txn = build_raw_txn(
        *alice.address(),
        &chain_state,
        empty_txn_payload(),
        Some(now_seconds + new_timeout_config_seconds + 1),
    );
    let signed_txn = alice.sign_txn(txn);

    let output = execute_and_apply(&chain_state, Transaction::UserTransaction(signed_txn));
    //verify txn timeout
    assert!(output.status().is_discarded());
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_txn_publish_option() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = txn_publish_config_type_tag();
    let script_allowed = false;
    let module_publishing_allowed = false;
    let vote_script =
        vote_txn_publish_option_script(&net, script_allowed, module_publishing_allowed);

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    // get TransactionPublishOption
    let module_id = ModuleId::new(
        genesis_address(),
        Identifier::new("TransactionPublishOption")?,
    );

    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("is_module_allowed")?,
        vec![],
        serialize_values(&vec![MoveValue::Address(genesis_address())]),
        None,
    )?;
    let is_module_allowed_on_chain: bool =
        bcs_ext::from_bytes(&read_config.pop().unwrap()).unwrap();
    assert_eq!(is_module_allowed_on_chain, module_publishing_allowed);

    let is_script_allowed_on_chain: bool = {
        let mut rets = starcoin_dev::playground::call_contract(
            &chain_state,
            module_id,
            "is_script_allowed",
            vec![],
            vec![TransactionArgument::Address(genesis_address())],
            None,
        )?;

        let r = rets.pop().unwrap().1;
        bcs_ext::from_bytes(r.as_slice())?
    };
    assert_eq!(is_script_allowed_on_chain, script_allowed);
    Ok(())
}

//TODO fix this test
// vm config is static code config from stdlib v10, use code update to update VMConfig.
#[ignore]
#[stest::test]
fn test_modify_on_chain_vm_config_option() -> Result<()> {
    let alice = Account::new();
    let bob = Account::new();
    let (chain_state, net) = prepare_genesis();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let action_type_tag = vm_config_type_tag();

    let one_day: u64 = 60 * 60 * 24 * 1000;

    // blockmeta txn is needed to create reward info.
    // block 1
    {
        let block_number = current_block_number(&chain_state) + 1;
        let block_timestamp = net.time_service().now_millis() + one_day * block_number - 1;
        let miner = Account::new();
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *miner.address(),
                Some(miner.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
    }
    //create user for txn verifier
    let script_function = encode_create_account_script_function(
        net.stdlib_version(),
        stc_type_tag(),
        bob.address(),
        bob.auth_key(),
        pre_mint_amount / 8,
    );
    association_execute_should_success(
        &net,
        &chain_state,
        TransactionPayload::ScriptFunction(script_function),
    )?;

    //get gas_used
    let output = account_execute_with_output(&bob, &chain_state, empty_txn_payload());
    let old_gas_used = output.gas_used();
    let account_state_reader = AccountStateReader::new(&chain_state);
    let mut vm_config = account_state_reader
        .get_on_chain_config::<VMConfig>()?
        .unwrap();
    //set vm config parameter
    vm_config
        .gas_schedule
        .gas_constants
        .global_memory_per_byte_cost = InternalGasUnits::new(8);
    vm_config
        .gas_schedule
        .gas_constants
        .global_memory_per_byte_write_cost = InternalGasUnits::new(12);
    let vote_script = vote_vm_config_script(&net, vm_config);

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    // get gas used of modified gas schedule
    let output = account_execute_with_output(&bob, &chain_state, empty_txn_payload());
    assert!(output.gas_used() > old_gas_used);
    Ok(())
}
