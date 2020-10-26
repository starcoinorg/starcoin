// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execute_readonly_function;
use crate::stdlib_test::dao_vote_test;
use crate::test_helper::prepare_genesis;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_functional_tests::account::Account;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::Script;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::on_chain_config::{
    consensus_config_type_tag, ConsensusConfig, OnChainConfig,
};
use starcoin_vm_types::transaction_argument::TransactionArgument;
use starcoin_vm_types::values::{VMValueCast, Value};
use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};

pub fn on_chain_config_type_tag(params_type_tag: TypeTag) -> TypeTag {
    TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("OnChainConfigDao").unwrap(),
        name: Identifier::new("OnChainConfigUpdate").unwrap(),
        type_params: vec![params_type_tag],
    })
}
pub fn reward_config_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("RewardConfig").unwrap(),
        name: Identifier::new("RewardConfig").unwrap(),
        type_params: vec![],
    })
}
pub fn txn_publish_config_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("TransactionPublishOption").unwrap(),
        name: Identifier::new("TransactionPublishOption").unwrap(),
        type_params: vec![],
    })
}

#[stest::test]
fn test_modify_on_chain_consensus_config() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let script1 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ProposeUpdateConsensusConfig,
    )
    .into_vec();

    let strategy = 2u8;
    let vote_script = Script::new(
        script1,
        vec![],
        vec![
            TransactionArgument::U64(80),
            TransactionArgument::U64(10),
            TransactionArgument::U128(64000000000),
            TransactionArgument::U64(10),
            TransactionArgument::U64(48),
            TransactionArgument::U64(24),
            TransactionArgument::U64(1),
            TransactionArgument::U64(60),
            TransactionArgument::U64(2),
            TransactionArgument::U64(1000000),
            TransactionArgument::U8(strategy),
            TransactionArgument::U64(0),
        ],
    );

    let script2 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ExecuteOnChainConfigProposal,
    )
    .into_vec();

    let execute_script = Script::new(
        script2,
        vec![consensus_config_type_tag()],
        vec![TransactionArgument::U64(0)],
    );

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        net,
        vote_script,
        on_chain_config_type_tag(consensus_config_type_tag()),
        execute_script,
    )?;
    //get consensus config
    let module_id = ModuleId::new(
        genesis_address(),
        Identifier::new("ConsensusConfig").unwrap(),
    );
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("get_config").unwrap(),
        vec![],
        vec![],
    )
    .unwrap();
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

    let script1 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ProposeUpdateRewardConfig,
    )
    .into_vec();

    let reward_delay: u64 = 10;
    let vote_script = Script::new(
        script1,
        vec![],
        vec![
            TransactionArgument::U64(reward_delay),
            TransactionArgument::U64(0),
        ],
    );

    let script2 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ExecuteOnChainConfigProposal,
    )
    .into_vec();

    let execute_script = Script::new(
        script2,
        vec![reward_config_type_tag()],
        vec![TransactionArgument::U64(0)],
    );

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        net,
        vote_script,
        on_chain_config_type_tag(reward_config_type_tag()),
        execute_script,
    )?;
    //get RewardConfig
    let module_id = ModuleId::new(genesis_address(), Identifier::new("RewardConfig").unwrap());
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("reward_delay").unwrap(),
        vec![],
        vec![],
    )
    .unwrap();
    let reward_delay_on_chain: u64 = read_config.pop().unwrap().1.cast().unwrap();

    assert_eq!(reward_delay_on_chain, reward_delay);
    Ok(())
}

#[stest::test]
fn test_modify_on_chain_txn_publish_option() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let script1 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ProposeUpdateTxnPublishOption,
    )
    .into_vec();

    let script_hash = HashValue::random();
    let module_publishing_allowed: bool = false;
    let vote_script = Script::new(
        script1,
        vec![],
        vec![
            TransactionArgument::U8Vector(script_hash.to_vec()),
            TransactionArgument::Bool(module_publishing_allowed),
            TransactionArgument::U64(0),
        ],
    );

    let script2 = compiled_transaction_script(
        net.stdlib_version(),
        StdlibScript::ExecuteOnChainConfigProposal,
    )
    .into_vec();

    let execute_script = Script::new(
        script2,
        vec![txn_publish_config_type_tag()],
        vec![TransactionArgument::U64(0)],
    );

    let chain_state = dao_vote_test(
        alice,
        chain_state,
        net.clone(),
        vote_script,
        on_chain_config_type_tag(txn_publish_config_type_tag()),
        execute_script,
    )?;
    //get RewardConfig
    let address = genesis_address();
    let module_id = ModuleId::new(
        address,
        Identifier::new("TransactionPublishOption").unwrap(),
    );
    let mut read_config = execute_readonly_function(
        &chain_state,
        &module_id,
        &Identifier::new("is_module_allowed").unwrap(),
        vec![],
        vec![Value::address(address)],
    )?;
    let is_script_allowed_on_chain: bool = read_config.pop().unwrap().1.cast().unwrap();

    assert_eq!(is_script_allowed_on_chain, module_publishing_allowed);
    Ok(())
}
