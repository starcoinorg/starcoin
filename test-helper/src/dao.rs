// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::{
    account_execute_should_success, association_execute_should_success, blockmeta_execute,
    current_block_number, get_balance,
};
use crate::Account;
use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_executor::{encode_create_account_script_function, execute_readonly_function};
use starcoin_logger::prelude::*;
use starcoin_state_api::{ChainStateReader, StateView};
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::build_empty_script;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{association_address, genesis_address, stc_type_tag};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::{ScriptFunction, TransactionPayload};
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::gas_schedule::GasAlgebra;
use starcoin_vm_types::on_chain_config::VMConfig;
use starcoin_vm_types::value::{serialize_values, MoveValue};

//TODO transfer to enum
pub const PENDING: u8 = 1;
pub const ACTIVE: u8 = 2;
#[allow(unused)]
pub const DEFEATED: u8 = 3;
pub const AGREED: u8 = 4;
pub const QUEUED: u8 = 5;
pub const EXECUTABLE: u8 = 6;
pub const EXTRACTED: u8 = 7;

pub fn proposal_state(
    state_view: &dyn StateView,
    token: TypeTag,
    action_ty: TypeTag,
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("proposal_state").unwrap(),
        vec![token, action_ty.clone()],
        serialize_values(&vec![
            MoveValue::Address(proposer_address),
            MoveValue::U64(proposal_id),
        ]),
        None,
    )
    .unwrap_or_else(|e| {
        panic!(
            "read proposal_state failed, action_ty: {:?}, proposer_address:{}, proposal_id:{}, vm_status: {:?}", action_ty,
            proposer_address, proposal_id, e
        )
    });
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn proposal_exist(
    state_view: &dyn StateView,
    token: TypeTag,
    action_ty: TypeTag,
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> bool {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("proposal_exists").unwrap(),
        vec![token, action_ty],
        serialize_values(&vec![
            MoveValue::Address(proposer_address),
            MoveValue::U64(proposal_id),
        ]),
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

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
pub fn transasction_timeout_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("TransactionTimeoutConfig").unwrap(),
        name: Identifier::new("TransactionTimeoutConfig").unwrap(),
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

fn execute_create_account(
    chain_state: &ChainStateDB,
    net: &ChainNetwork,
    alice: &Account,
    pre_mint_amount: u128,
    block_number: u64,
    block_timestamp: u64,
) -> Result<()> {
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                association_address(),
                None,
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        if !chain_state.exist_account(alice.address())? {
            let init_balance = pre_mint_amount / 4;
            let script_function = encode_create_account_script_function(
                net.stdlib_version(),
                stc_type_tag(),
                alice.address(),
                alice.auth_key(),
                init_balance,
            );
            debug!(
                "execute create account script: addr:{}, init_balance:{}",
                alice.address(),
                init_balance
            );
            association_execute_should_success(
                net,
                chain_state,
                TransactionPayload::ScriptFunction(script_function),
            )?;
        }

        Ok(())
    }
}
pub fn quorum_vote(state_view: &dyn StateView, token: TypeTag) -> u128 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("quorum_votes").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn voting_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_delay").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}
pub fn voting_period(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_period").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn min_action_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("min_action_delay").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

fn execute_cast_vote(
    net: &ChainNetwork,
    chain_state: &ChainStateDB,
    alice: &Account,
    dao_action_type_tag: &TypeTag,
    block_number: u64,
    block_timestamp: u64,
    proposal_id: u64,
) -> Result<()> {
    blockmeta_execute(
        chain_state,
        BlockMetadata::new(
            HashValue::zero(),
            block_timestamp,
            *alice.address(),
            Some(alice.auth_key()),
            0,
            block_number,
            net.chain_id(),
            0,
        ),
    )?;
    let proposer_address = *alice.address();
    let proposer_id = proposal_id;
    let voting_power = get_balance(*alice.address(), chain_state);
    debug!("{} voting power: {}", alice.address(), voting_power);
    let script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("DaoVoteScripts").unwrap(),
        ),
        Identifier::new("cast_vote").unwrap(),
        vec![stc_type_tag(), dao_action_type_tag.clone()],
        vec![
            bcs_ext::to_bytes(&proposer_address).unwrap(),
            bcs_ext::to_bytes(&proposer_id).unwrap(),
            bcs_ext::to_bytes(&true).unwrap(),
            bcs_ext::to_bytes(&(voting_power / 2)).unwrap(),
        ],
    );
    // vote first.
    account_execute_should_success(
        alice,
        chain_state,
        TransactionPayload::ScriptFunction(script_function),
    )?;
    let quorum = quorum_vote(chain_state, stc_type_tag());
    debug!(
        "proposer_id:{}, action: {}, quorum: {}",
        dao_action_type_tag, proposer_id, quorum
    );

    let state = proposal_state(
        chain_state,
        stc_type_tag(),
        dao_action_type_tag.clone(),
        *alice.address(),
        proposal_id,
    );
    assert_eq!(
        state, ACTIVE,
        "expect proposer_id {}'s state ACTIVE, but got: {}",
        proposer_id, state
    );
    Ok(())
}

///vote script consensus
pub fn vote_script_consensus(_net: &ChainNetwork, strategy: u8) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_consensus_config").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&80u64).unwrap(),
            bcs_ext::to_bytes(&10000u64).unwrap(),
            bcs_ext::to_bytes(&64000000000u128).unwrap(),
            bcs_ext::to_bytes(&10u64).unwrap(),
            bcs_ext::to_bytes(&48u64).unwrap(),
            bcs_ext::to_bytes(&24u64).unwrap(),
            bcs_ext::to_bytes(&1000u64).unwrap(),
            bcs_ext::to_bytes(&60000u64).unwrap(),
            bcs_ext::to_bytes(&2u64).unwrap(),
            bcs_ext::to_bytes(&1000000u64).unwrap(),
            bcs_ext::to_bytes(&strategy).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}

///reward on chain config script
pub fn vote_reward_scripts(_net: &ChainNetwork, reward_delay: u64) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_reward_config").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&reward_delay).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}

/// vote txn publish option scripts
pub fn vote_txn_timeout_script(_net: &ChainNetwork, duration_seconds: u64) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_txn_timeout_config").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&duration_seconds).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}
/// vote txn publish option scripts
pub fn vote_txn_publish_option_script(
    _net: &ChainNetwork,
    script_allowed: bool,
    module_publishing_allowed: bool,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_txn_publish_option").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&script_allowed).unwrap(),
            bcs_ext::to_bytes(&module_publishing_allowed).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}

/// vote vm config scripts
pub fn vote_vm_config_script(_net: &ChainNetwork, vm_config: VMConfig) -> ScriptFunction {
    let gas_constants = &vm_config.gas_schedule.gas_constants;
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_vm_config").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(
                &bcs_ext::to_bytes(&vm_config.gas_schedule.instruction_table).unwrap(),
            )
            .unwrap(),
            bcs_ext::to_bytes(&bcs_ext::to_bytes(&vm_config.gas_schedule.native_table).unwrap())
                .unwrap(),
            bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_cost.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_write_cost.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.min_transaction_gas_units.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.large_transaction_cutoff.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.intrinsic_gas_per_byte.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.maximum_number_of_gas_units.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.min_price_per_gas_unit.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.max_price_per_gas_unit.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.max_transaction_size_in_bytes).unwrap(),
            bcs_ext::to_bytes(&gas_constants.gas_unit_scaling_factor).unwrap(),
            bcs_ext::to_bytes(&gas_constants.default_account_size.get()).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}

pub fn vote_language_version(_net: &ChainNetwork, lang_version: u64) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_move_language_version").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&lang_version).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    )
}

/// execute on chain config scripts
pub fn execute_script_on_chain_config(
    _net: &ChainNetwork,
    type_tag: TypeTag,
    proposal_id: u64,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("execute_on_chain_config_proposal").unwrap(),
        vec![type_tag],
        vec![bcs_ext::to_bytes(&proposal_id).unwrap()],
    )
}

pub fn empty_txn_payload() -> TransactionPayload {
    TransactionPayload::ScriptFunction(build_empty_script())
}

pub fn dao_vote_test(
    alice: &Account,
    chain_state: &ChainStateDB,
    net: &ChainNetwork,
    vote_script: ScriptFunction,
    action_type_tag: TypeTag,
    execute_script: ScriptFunction,
    proposal_id: u64,
) -> Result<()> {
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let one_day: u64 = 60 * 60 * 24 * 1000;
    // Block 1
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    execute_create_account(
        chain_state,
        net,
        alice,
        pre_mint_amount,
        block_number,
        block_timestamp,
    )?;
    // block 2
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(vote_script),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, PENDING);
    }

    // block 3
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + voting_delay(chain_state, stc_type_tag()) + 10000;
    execute_cast_vote(
        net,
        chain_state,
        alice,
        &action_type_tag,
        block_number,
        block_timestamp,
        proposal_id,
    )?;

    // block 4
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + voting_period(chain_state, stc_type_tag()) - 10 * 1000;
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, ACTIVE);
    }

    // block 5
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + 20 * 1000;
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, AGREED);

        let script_function = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Dao").unwrap()),
            Identifier::new("queue_proposal_action").unwrap(),
            vec![stc_type_tag(), action_type_tag.clone()],
            vec![
                bcs_ext::to_bytes(alice.address()).unwrap(),
                bcs_ext::to_bytes(&proposal_id).unwrap(),
            ],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, QUEUED);
    }

    // block 6
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + min_action_delay(chain_state, stc_type_tag());
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, EXECUTABLE);
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(execute_script),
        )?;
    }

    // block 7
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + 1000;
    {
        blockmeta_execute(
            chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, EXTRACTED);
    }
    {
        //Unstack
        let script_function = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("DaoVoteScripts").unwrap(),
            ),
            Identifier::new("unstake_vote").unwrap(),
            vec![stc_type_tag(), action_type_tag.clone()],
            vec![
                bcs_ext::to_bytes(alice.address()).unwrap(),
                bcs_ext::to_bytes(&proposal_id).unwrap(),
            ],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }
    {
        //Unstack
        let script_function = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Dao").unwrap()),
            Identifier::new("destroy_terminated_proposal").unwrap(),
            vec![stc_type_tag(), action_type_tag],
            vec![
                bcs_ext::to_bytes(alice.address()).unwrap(),
                bcs_ext::to_bytes(&proposal_id).unwrap(),
            ],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }
    Ok(())
}
