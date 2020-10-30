// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::{account_execute, association_execute, blockmeta_execute, get_balance};
use crate::Account;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_executor::{encode_create_account_script, execute_readonly_function};
use starcoin_state_api::StateView;
use starcoin_statedb::ChainStateDB;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{association_address, genesis_address, stc_type_tag};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::genesis_config::ChainNetwork;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::{Script, TransactionArgument, TransactionPayload};
use starcoin_vm_types::values::{VMValueCast, Value};
use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};
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
        vec![token, action_ty],
        vec![Value::address(proposer_address), Value::u64(proposal_id)],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
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
    bob: &Account,
    pre_mint_amount: u128,
    block_number: u64,
    block_timestamp: u64,
) -> Result<()> {
    {
        blockmeta_execute(
            &chain_state,
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
        let script = encode_create_account_script(
            net.stdlib_version(),
            stc_type_tag(),
            alice.address(),
            alice.auth_key(),
            pre_mint_amount / 2,
        );
        association_execute(
            net.genesis_config(),
            &chain_state,
            TransactionPayload::Script(script),
        )?;

        let script = encode_create_account_script(
            net.stdlib_version(),
            stc_type_tag(),
            bob.address(),
            bob.auth_key(),
            pre_mint_amount / 4,
        );
        association_execute(
            net.genesis_config(),
            &chain_state,
            TransactionPayload::Script(script),
        )?;
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
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

pub fn voting_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_delay").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}
pub fn voting_period(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_period").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

pub fn min_action_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("min_action_delay").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

fn execute_cast_vote(
    net: &ChainNetwork,
    chain_state: &ChainStateDB,
    alice: &Account,
    dao_action_type_tag: &TypeTag,
    block_number: u64,
    block_timestamp: u64,
) -> Result<()> {
    blockmeta_execute(
        &chain_state,
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
    let cast_script =
        compiled_transaction_script(net.stdlib_version(), StdlibScript::CastVote).into_vec();
    let proposer_address = *alice.address();
    let proposer_id = 0;
    let voting_power = get_balance(*alice.address(), chain_state);
    println!("alice voting power: {}", voting_power);
    let script = Script::new(
        cast_script,
        vec![stc_type_tag(), dao_action_type_tag.clone()],
        vec![
            TransactionArgument::Address(proposer_address),
            TransactionArgument::U64(proposer_id),
            TransactionArgument::Bool(true),
            TransactionArgument::U128(voting_power / 2),
        ],
    );
    // vote first.
    account_execute(&alice, chain_state, TransactionPayload::Script(script))?;
    let quorum = quorum_vote(chain_state, stc_type_tag());
    println!("quorum: {}", quorum);

    let state = proposal_state(
        chain_state,
        stc_type_tag(),
        dao_action_type_tag.clone(),
        *alice.address(),
        0,
    );
    assert_eq!(state, ACTIVE);
    Ok(())
}

pub fn dao_vote_test(
    alice: Account,
    chain_state: ChainStateDB,
    net: ChainNetwork,
    vote_script: Script,
    action_type_tag: TypeTag,
    execute_script: Script,
) -> Result<ChainStateDB> {
    let bob = Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let one_day: u64 = 60 * 60 * 24 * 1000;
    // Block 1
    let block_number = 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    execute_create_account(
        &chain_state,
        &net,
        &alice,
        &bob,
        pre_mint_amount,
        block_number,
        block_timestamp,
    )?;

    // block 2
    let block_number = 2;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    {
        blockmeta_execute(
            &chain_state,
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
        account_execute(
            &alice,
            &chain_state,
            TransactionPayload::Script(vote_script),
        )?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, PENDING);
    }

    // block 3
    let block_number = 3;
    let block_timestamp =
        block_timestamp + voting_delay(&chain_state, stc_type_tag()) * 1000 + 10000;
    execute_cast_vote(
        &net,
        &chain_state,
        &alice,
        &action_type_tag,
        block_number,
        block_timestamp,
    )?;

    // block 4
    let block_number = 4;
    let block_timestamp =
        block_timestamp + voting_period(&chain_state, stc_type_tag()) * 1000 - 10 * 1000;
    {
        blockmeta_execute(
            &chain_state,
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
            &chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, ACTIVE);
    }

    // block 5
    let block_number = 5;
    let block_timestamp = block_timestamp + 20 * 1000;
    {
        blockmeta_execute(
            &chain_state,
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
            &chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, AGREED);

        let script =
            compiled_transaction_script(net.stdlib_version(), StdlibScript::QueueProposalAction)
                .into_vec();
        let script = Script::new(
            script,
            vec![stc_type_tag(), action_type_tag.clone()],
            vec![
                TransactionArgument::Address(*alice.address()),
                TransactionArgument::U64(0),
            ],
        );
        account_execute(&alice, &chain_state, TransactionPayload::Script(script))?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, QUEUED);
    }

    // block 6
    let block_number = 6;
    let block_timestamp = block_timestamp + min_action_delay(&chain_state, stc_type_tag()) * 1000;
    {
        blockmeta_execute(
            &chain_state,
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
            &chain_state,
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, EXECUTABLE);
        account_execute(
            &alice,
            &chain_state,
            TransactionPayload::Script(execute_script),
        )?;
    }

    // block 7
    let block_number = 7;
    let block_timestamp = block_timestamp + 1000;
    {
        blockmeta_execute(
            &chain_state,
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
            &chain_state,
            stc_type_tag(),
            action_type_tag,
            *alice.address(),
            0,
        );
        assert_eq!(state, EXTRACTED);
    }
    // return chain state for verify
    Ok(chain_state)
}
