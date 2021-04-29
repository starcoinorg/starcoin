// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{encode_create_account_script_function, execute_readonly_function};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::TransactionPayload;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::gas_schedule::{GasAlgebra, InternalGasUnits};
use starcoin_vm_types::on_chain_config::{
    consensus_config_type_tag, vm_config_type_tag, ConsensusConfig, OnChainConfig, VMConfig,
    CONSENSUS_CONFIG_IDENTIFIER,
};
use starcoin_vm_types::value::{serialize_values, MoveValue};
use starcoin_vm_types::values::VMValueCast;
use test_helper::dao::{
    dao_vote_test, empty_txn_payload, execute_script_on_chain_config, on_chain_config_type_tag,
    reward_config_type_tag, transasction_timeout_type_tag, txn_publish_config_type_tag,
    vote_reward_scripts, vote_script_consensus, vote_txn_publish_option_script,
    vote_txn_timeout_script, vote_vm_config_script,
};
use test_helper::executor::{
    account_execute, account_execute_with_output, association_execute, blockmeta_execute,
    current_block_number, prepare_genesis,
};
use test_helper::Account;

#[stest::test]
fn test_modify_on_chain_consensus_config() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = consensus_config_type_tag();
    let strategy = 2u8;

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_consensus(&net, strategy),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    //get consensus config
    let module_id = ModuleId::new(genesis_address(), CONSENSUS_CONFIG_IDENTIFIER.clone());
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("get_config")?,
        vec![],
        vec![],
    )?;
    let annotator = MoveValueAnnotator::new(&chain_state);
    let (t, v) = read_config.pop().unwrap();
    let layout = annotator.type_tag_to_type_layout(&t)?;
    let r = v
        .simple_serialize(&layout)
        .ok_or_else(|| anyhow::format_err!("fail to serialize contract result"))?;
    let config = ConsensusConfig::deserialize_into_config(r.as_slice())?;
    assert_eq!(config.strategy, strategy);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_reward_config() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let action_type_tag = reward_config_type_tag();
    let reward_delay: u64 = 100;

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_reward_scripts(&net, reward_delay),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    //get RewardConfig
    let module_id = ModuleId::new(genesis_address(), Identifier::new("RewardConfig")?);
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("reward_delay")?,
        vec![],
        vec![],
    )?;
    let reward_delay_on_chain: u64 = read_config.pop().unwrap().1.cast().unwrap();

    assert_eq!(reward_delay_on_chain, reward_delay);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_config_txn_timeout() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let action_type_tag = transasction_timeout_type_tag();
    let duration_seconds: u64 = 3000;

    let chain_state = dao_vote_test(
        alice.clone(),
        chain_state,
        &net,
        vote_txn_timeout_script(&net, duration_seconds),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    //verify txn timeout
    {
        assert!(
            account_execute(&net, &alice, &chain_state, empty_txn_payload())?
                .status()
                .is_discarded()
        );
    }
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

    let chain_state = dao_vote_test(
        alice,
        chain_state,
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
    )?;
    let is_module_allowed_on_chain: bool = read_config.pop().unwrap().1.cast().unwrap();
    assert_eq!(is_module_allowed_on_chain, module_publishing_allowed);

    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("is_script_allowed")?,
        vec![],
        serialize_values(&vec![MoveValue::Address(genesis_address())]),
    )?;
    let is_script_allowed_on_chain: bool = read_config.pop().unwrap().1.cast().unwrap();
    assert_eq!(is_script_allowed_on_chain, script_allowed);
    Ok(())
}

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
    association_execute(
        &net,
        &chain_state,
        TransactionPayload::ScriptFunction(script_function),
    )?;

    //get gas_used
    let output = account_execute_with_output(&net, &bob, &chain_state, empty_txn_payload());
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

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
        0,
    )?;
    // get gas used of modified gas schedule
    let output = account_execute_with_output(&net, &bob, &chain_state, empty_txn_payload());
    assert!(output.gas_used() > old_gas_used);
    Ok(())
}
