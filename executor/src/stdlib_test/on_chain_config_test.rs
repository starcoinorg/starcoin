// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{encode_create_account_script, execute_readonly_function};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::TransactionPayload;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::gas_schedule::{GasAlgebra, GasUnits};
use starcoin_vm_types::on_chain_config::{
    consensus_config_type_tag, version_config_type_tag, vm_config_type_tag, ConsensusConfig,
    OnChainConfig, VMConfig, CONSENSUS_CONFIG_IDENTIFIER, VERSION_CONFIG_IDENTIFIER,
};
use starcoin_vm_types::values::{VMValueCast, Value};
use test_helper::dao::{
    dao_vote_test, empty_txn_payload, execute_script_on_chain_config, on_chain_config_type_tag,
    reward_config_type_tag, transasction_timeout_type_tag, txn_publish_config_type_tag,
    vote_reward_scripts, vote_script_consensus, vote_txn_publish_option_script,
    vote_txn_timeout_script, vote_version_script, vote_vm_config_script,
};
use test_helper::executor::{
    account_execute, account_execute_with_output, association_execute, prepare_genesis,
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
    )?;
    //verify txn timeout
    {
        assert!(account_execute(&alice, &chain_state, empty_txn_payload(&net)).is_err());
    }
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_txn_publish_option() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = txn_publish_config_type_tag();
    let script_hash = HashValue::random();
    let module_publishing_allowed = true;
    let vote_script = vote_txn_publish_option_script(&net, script_hash, module_publishing_allowed);

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
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
        vec![Value::address(genesis_address())],
    )?;
    let is_script_allowed_on_chain: bool = read_config.pop().unwrap().1.cast().unwrap();
    assert_eq!(is_script_allowed_on_chain, module_publishing_allowed);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_vm_config_option() -> Result<()> {
    let alice = Account::new();
    let bob = Account::new();
    let (chain_state, net) = prepare_genesis();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let action_type_tag = vm_config_type_tag();

    //create user for txn verifier
    let script = encode_create_account_script(
        net.stdlib_version(),
        stc_type_tag(),
        bob.address(),
        bob.auth_key(),
        pre_mint_amount / 8,
    );
    association_execute(
        net.genesis_config(),
        &chain_state,
        TransactionPayload::Script(script),
    )?;
    //get gas_used
    let output = account_execute_with_output(&bob, &chain_state, empty_txn_payload(&net));
    let old_gas_used = output.gas_used();

    let account_state_reader = AccountStateReader::new(&chain_state);
    let mut vm_config = account_state_reader
        .get_on_chain_config::<VMConfig>()?
        .unwrap();
    //set vm config parameter
    vm_config
        .gas_schedule
        .gas_constants
        .global_memory_per_byte_cost = GasUnits::new(8);
    vm_config
        .gas_schedule
        .gas_constants
        .global_memory_per_byte_write_cost = GasUnits::new(12);
    let vote_script = vote_vm_config_script(&net, vm_config);

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
    )?;

    // get gas used of modified gas schedule
    let output = account_execute_with_output(&bob, &chain_state, empty_txn_payload(&net));
    assert!(output.gas_used() > old_gas_used);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_version() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = version_config_type_tag();

    let major: u64 = 8;
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_version_script(&net, major),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
    )?;
    // get version on chain
    let module_id = ModuleId::new(genesis_address(), VERSION_CONFIG_IDENTIFIER.clone());
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("get")?,
        vec![],
        vec![],
    )?;
    let on_chain_version: u64 = read_config.pop().unwrap().1.cast().unwrap();
    assert_eq!(on_chain_version, major);
    Ok(())
}
