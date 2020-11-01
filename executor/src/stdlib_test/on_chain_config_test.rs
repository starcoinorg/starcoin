// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execute_readonly_function;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::on_chain_config::{
    consensus_config_type_tag, ConsensusConfig, OnChainConfig,
};
use starcoin_vm_types::values::VMValueCast;
use test_helper::dao::{
    dao_vote_test, execute_script_on_chain_config, on_chain_config_type_tag,
    reward_config_type_tag, txn_publish_config_type_tag, vote_reward_scripts,
    vote_script_consensus, vote_txn_publish_option_script,
};
use test_helper::executor::prepare_genesis;
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
    let module_id = ModuleId::new(genesis_address(), Identifier::new("ConsensusConfig")?);
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
fn test_modify_on_chain_txn_publish_option() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = txn_publish_config_type_tag();
    let script_hash = HashValue::random();
    let module_publishing_allowed = true;
    let vote_script = vote_txn_publish_option_script(&net, script_hash, module_publishing_allowed);

    let _chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script,
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(&net, action_type_tag, 0u64),
    )?;
    //get TransactionPublishOption
    // let module_id = ModuleId::new(
    //     genesis_address(),
    //     Identifier::new("TransactionPublishOption")?,
    // );

    // Fixme: verify
    // let mut read_config = execute_readonly_function(
    //     &chain_state,
    //     &module_id,
    //     &Identifier::new("is_module_allowed")?,
    //     vec![],
    //     vec![Value::address(*alice.address())],
    // )?;
    // // dbg!(read_config);
    // let is_script_allowed_on_chain: bool = read_config.pop().unwrap().1.cast().unwrap();
    // assert_eq!(is_script_allowed_on_chain, module_publishing_allowed);
    Ok(())
}
