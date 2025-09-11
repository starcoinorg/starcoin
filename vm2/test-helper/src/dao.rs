// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::{
    account_execute_should_success, association_execute_should_success, blockmeta_execute,
    current_block_number, get_balance,
};
use anyhow::Result;
use starcoin_cached_packages::starcoin_stdlib::{
    dao_vote_scripts_cast_vote, on_chain_config_scripts_execute_on_chain_config_proposal,
};
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_transaction_builder::vm2::encode_create_account_script_function;
use starcoin_vm2_executor::executor::execute_readonly_function;
use starcoin_vm2_state_api::ChainStateReader;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{
    account::Account,
    account_address::AccountAddress,
    account_config::{association_address, genesis_address, stc_type_tag},
    block_metadata::BlockMetadata,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction::{EntryFunction, TransactionPayload},
};
use starcoin_vm2_vm_types::{
    on_chain_config::OnChainConfig,
    on_chain_resource::ChainId,
    value::{serialize_values, MoveValue},
    StateView,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    Pending = 1,
    Active = 2,
    Defeated = 3,
    Agreed = 4,
    Queued = 5,
    Executable = 6,
    Extracted = 7,
}

impl ProposalState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(ProposalState::Pending),
            2 => Some(ProposalState::Active),
            3 => Some(ProposalState::Defeated),
            4 => Some(ProposalState::Agreed),
            5 => Some(ProposalState::Queued),
            6 => Some(ProposalState::Executable),
            7 => Some(ProposalState::Extracted),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub fn proposal_state<S: StateView>(
    state_view: &S,
    token: TypeTag,
    action_ty: TypeTag,
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> ProposalState {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
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
    let state_value: u8 = bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap();
    ProposalState::from_u8(state_value).expect("Invalid proposal state value")
}

pub fn proposal_exist<S: StateView>(
    state_view: &S,
    token: TypeTag,
    action_ty: TypeTag,
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> bool {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
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

pub fn execute_create_account(
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
                0,
            ),
        )?;
        if !chain_state.exist_account(alice.address())? {
            let init_balance = pre_mint_amount / 4;
            let script_function = encode_create_account_script_function(
                net.stdlib_version().version(),
                stc_type_tag(),
                alice.address(),
                alice.auth_key(),
                init_balance,
            );
            association_execute_should_success(
                net,
                chain_state,
                TransactionPayload::EntryFunction(script_function),
            )?;
        }

        Ok(())
    }
}

pub fn quorum_vote<S: StateView>(state_view: &S, token: TypeTag) -> u128 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
        &Identifier::new("quorum_votes").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn voting_delay<S: StateView>(state_view: &S, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
        &Identifier::new("voting_delay").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn voting_period<S: StateView>(state_view: &S, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
        &Identifier::new("voting_period").unwrap(),
        vec![token],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn min_action_delay<S: StateView>(state_view: &S, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("dao").unwrap()),
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
            0,
            block_number,
            ChainId::new(net.chain_id().id()),
            0,
            vec![],
            0,
        ),
    )?;
    let proposer_address = *alice.address();
    let proposer_id = proposal_id;
    let voting_power = get_balance(*alice.address(), chain_state);
    let cast_vote_payload = dao_vote_scripts_cast_vote(
        stc_type_tag(),
        *dao_action_type_tag,
        proposer_address,
        proposal_id,
        true,
        voting_power / 2,
    );
    // vote first.
    account_execute_should_success(alice, chain_state, cast_vote_payload)?;
    let _quorum = quorum_vote(chain_state, stc_type_tag());

    let state = proposal_state(
        chain_state,
        stc_type_tag(),
        dao_action_type_tag.clone(),
        *alice.address(),
        proposal_id,
    );
    assert_eq!(
        state,
        ProposalState::Active,
        "expect proposer_id {}'s state ACTIVE, but got: {:?}",
        proposer_id,
        state
    );
    Ok(())
}

pub fn execute_script_on_chain_config(type_tag: TypeTag, proposal_id: u64) -> TransactionPayload {
    on_chain_config_scripts_execute_on_chain_config_proposal(type_tag, proposal_id)
}

pub fn dao_vote_test(
    alice: &Account,
    chain_state: &ChainStateDB,
    net: &ChainNetwork,
    vote_script: EntryFunction,
    action_type_tag: TypeTag,
    execute_txn_payload: TransactionPayload,
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
                0,
            ),
        )?;
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::EntryFunction(vote_script),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, ProposalState::Pending);
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
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
        assert_eq!(state, ProposalState::Active);
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
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
        assert_eq!(state, ProposalState::Agreed);

        let script_function = EntryFunction::new(
            ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
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
            TransactionPayload::EntryFunction(script_function),
        )?;
        let state = proposal_state(
            chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            proposal_id,
        );
        assert_eq!(state, ProposalState::Queued);
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
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
        assert_eq!(state, ProposalState::Executable);
        account_execute_should_success(alice, chain_state, execute_txn_payload)?;
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
                0,
                block_number,
                ChainId::new(net.chain_id().id()),
                0,
                vec![],
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
        assert_eq!(state, ProposalState::Extracted);
    }
    {
        // Unstake
        let script_function = EntryFunction::new(
            ModuleId::new(
                genesis_address(),
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
            TransactionPayload::EntryFunction(script_function),
        )?;
    }
    {
        // Destroy terminated proposal
        let script_function = EntryFunction::new(
            ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
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
            TransactionPayload::EntryFunction(script_function),
        )?;
    }
    Ok(())
}
