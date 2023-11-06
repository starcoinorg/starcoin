// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crate::executor::{
    account_execute_should_success, association_execute_should_success, blockmeta_execute,
    current_block_number, get_balance,
};
use crate::Account;
use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_executor::execute_readonly_function;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::BlockBody;
use starcoin_state_api::{
    ChainStateReader, ChainStateWriter, StateReaderExt, StateView, StateWithProof,
};
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::encode_create_account_script_function;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::{association_address, genesis_address, stc_type_tag};
use starcoin_types::block::{Block, BlockHeader, BlockHeaderExtra};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::{ScriptFunction, TransactionPayload};
use starcoin_types::U256;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::value::{serialize_values, MoveValue};

//TODO transfer to enum
pub const PENDING: u8 = 1;
pub const ACTIVE: u8 = 2;
pub const REJECTED: u8 = 3;
#[allow(unused)]
pub const DEFEATED: u8 = 4;
pub const AGREED: u8 = 5;
pub const QUEUED: u8 = 6;
pub const EXECUTABLE: u8 = 7;
pub const EXTRACTED: u8 = 8;

fn snapshot_access_path<S: StateView>(state_view: &S, user_address: &AccountAddress) -> Vec<u8> {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("SnapshotUtil").unwrap()),
        &Identifier::new("get_access_path").unwrap(),
        vec![starcoin_dao_type_tag()],
        serialize_values(&vec![MoveValue::Address(*user_address)]),
        None,
    )
    .unwrap_or_else(|e| {
        panic!(
            "read snapshot_access_path failed, user_address:{}, vm_status: {:?}",
            user_address, e
        )
    });
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

fn get_with_proof_by_root(
    state_db: &ChainStateDB,
    access_path: AccessPath,
    state_root: HashValue,
) -> Result<StateWithProof> {
    let reader = state_db.fork_at(state_root);
    reader.get_with_proof(&access_path)
}

fn proposal_state<S: StateView>(state_view: &S, proposal_id: u64) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("proposal_state").unwrap(),
        vec![starcoin_dao_type_tag()],
        serialize_values(&vec![MoveValue::U64(proposal_id)]),
        None,
    )
    .unwrap_or_else(|e| {
        panic!(
            "read proposal_state failed, proposal_id:{}, vm_status: {:?}",
            proposal_id, e
        )
    });
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

// pub fn on_chain_config_type_tag(params_type_tag: TypeTag) -> TypeTag {
//     TypeTag::Struct(StructTag {
//         address: genesis_address(),
//         module: Identifier::new("OnChainConfigDao").unwrap(),
//         name: Identifier::new("OnChainConfigUpdate").unwrap(),
//         type_params: vec![params_type_tag],
//     })
// }
// pub fn reward_config_type_tag() -> TypeTag {
//     TypeTag::Struct(StructTag {
//         address: genesis_address(),
//         module: Identifier::new("RewardConfig").unwrap(),
//         name: Identifier::new("RewardConfig").unwrap(),
//         type_params: vec![],
//     })
// }
// pub fn transaction_timeout_type_tag() -> TypeTag {
//     TypeTag::Struct(StructTag {
//         address: genesis_address(),
//         module: Identifier::new("TransactionTimeoutConfig").unwrap(),
//         name: Identifier::new("TransactionTimeoutConfig").unwrap(),
//         type_params: vec![],
//     })
// }
// pub fn txn_publish_config_type_tag() -> TypeTag {
//     TypeTag::Struct(StructTag {
//         address: genesis_address(),
//         module: Identifier::new("TransactionPublishOption").unwrap(),
//         name: Identifier::new("TransactionPublishOption").unwrap(),
//         type_params: vec![],
//     })
// }

pub fn quorum_vote<S: StateView>(state_view: &S, dao_type_tag: TypeTag) -> u128 {
    let scale_factor: Option<u8> = None;
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("quorum_votes").unwrap(),
        vec![dao_type_tag],
        vec![bcs_ext::to_bytes(&scale_factor).unwrap()],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn min_proposal_deposit<S: StateView>(state_view: &S, dao_type_tag: TypeTag) -> u128 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("min_proposal_deposit").unwrap(),
        vec![dao_type_tag],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn get_parent_hash<S: StateView>(state_view: &S) -> Vec<u8> {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Block").unwrap()),
        &Identifier::new("get_parent_hash").unwrap(),
        vec![],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn voting_delay<S: StateView>(state_view: &S, dao: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("voting_delay").unwrap(),
        vec![dao],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn voting_period<S: StateView>(state_view: &S, dao: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("voting_period").unwrap(),
        vec![dao],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn min_action_delay<S: StateView>(state_view: &S, dao: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("DAOSpace").unwrap()),
        &Identifier::new("min_action_delay").unwrap(),
        vec![dao],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

fn execute_cast_vote(
    chain_state: &ChainStateDB,
    alice: &Account,
    proposal_id: u64,
    snapshot_proofs: StateWithProof,
    dao_type_tag: TypeTag,
    choice: u8,
) -> Result<()> {
    let voting_power = get_balance(*alice.address(), chain_state);
    debug!("{} voting power: {}", alice.address(), voting_power);
    let proof_bytes = bcs_ext::to_bytes(&snapshot_proofs).unwrap();
    let script_function = ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("DAOSpace").unwrap()),
        Identifier::new("cast_vote_entry").unwrap(),
        vec![dao_type_tag.clone()],
        vec![
            bcs_ext::to_bytes(&proposal_id).unwrap(),
            bcs_ext::to_bytes(&proof_bytes).unwrap(),
            bcs_ext::to_bytes(&choice).unwrap(),
        ],
    );
    // vote first.
    account_execute_should_success(
        alice,
        chain_state,
        TransactionPayload::ScriptFunction(script_function),
    )?;
    let quorum = quorum_vote(chain_state, dao_type_tag);
    debug!("proposer_id:{}, quorum: {}", proposal_id, quorum);

    let state = proposal_state(chain_state, proposal_id);
    assert_eq!(
        state, ACTIVE,
        "expect proposer_id {}'s state ACTIVE, but got: {}",
        proposal_id, state
    );
    Ok(())
}

// ///vote script consensus
// pub fn vote_script_consensus(_net: &ChainNetwork, strategy: u8) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_consensus_config").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(&80u64).unwrap(),
//             bcs_ext::to_bytes(&10000u64).unwrap(),
//             bcs_ext::to_bytes(&64000000000u128).unwrap(),
//             bcs_ext::to_bytes(&10u64).unwrap(),
//             bcs_ext::to_bytes(&48u64).unwrap(),
//             bcs_ext::to_bytes(&24u64).unwrap(),
//             bcs_ext::to_bytes(&1000u64).unwrap(),
//             bcs_ext::to_bytes(&60000u64).unwrap(),
//             bcs_ext::to_bytes(&2u64).unwrap(),
//             bcs_ext::to_bytes(&1000000u64).unwrap(),
//             bcs_ext::to_bytes(&strategy).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }

// ///reward on chain config script
// pub fn vote_reward_scripts(_net: &ChainNetwork, reward_delay: u64) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_reward_config").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(&reward_delay).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }

// /// vote txn publish option scripts
// pub fn vote_txn_timeout_script(_net: &ChainNetwork, duration_seconds: u64) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_txn_timeout_config").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(&duration_seconds).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }
// /// vote txn publish option scripts
// pub fn vote_txn_publish_option_script(
//     _net: &ChainNetwork,
//     script_allowed: bool,
//     module_publishing_allowed: bool,
// ) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_txn_publish_option").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(&script_allowed).unwrap(),
//             bcs_ext::to_bytes(&module_publishing_allowed).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }

// /// vote vm config scripts
// pub fn vote_vm_config_script(_net: &ChainNetwork, vm_config: VMConfig) -> ScriptFunction {
//     let gas_constants = &vm_config.gas_schedule.gas_constants;
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_vm_config").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(
//                 &bcs_ext::to_bytes(&vm_config.gas_schedule.instruction_table).unwrap(),
//             )
//             .unwrap(),
//             bcs_ext::to_bytes(&bcs_ext::to_bytes(&vm_config.gas_schedule.native_table).unwrap())
//                 .unwrap(),
//             bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_cost.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_write_cost.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.min_transaction_gas_units.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.large_transaction_cutoff.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.intrinsic_gas_per_byte.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.maximum_number_of_gas_units.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.min_price_per_gas_unit.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.max_price_per_gas_unit.get()).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.max_transaction_size_in_bytes).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.gas_unit_scaling_factor).unwrap(),
//             bcs_ext::to_bytes(&gas_constants.default_account_size.get()).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }

// pub fn vote_language_version(_net: &ChainNetwork, lang_version: u64) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("propose_update_move_language_version").unwrap(),
//         vec![],
//         vec![
//             bcs_ext::to_bytes(&lang_version).unwrap(),
//             bcs_ext::to_bytes(&0u64).unwrap(),
//         ],
//     )
// }

// /// execute on chain config scripts
// pub fn execute_script_on_chain_config(
//     _net: &ChainNetwork,
//     type_tag: TypeTag,
//     proposal_id: u64,
// ) -> ScriptFunction {
//     ScriptFunction::new(
//         ModuleId::new(
//             core_code_address(),
//             Identifier::new("OnChainConfigScripts").unwrap(),
//         ),
//         Identifier::new("execute_on_chain_config_proposal").unwrap(),
//         vec![type_tag],
//         vec![bcs_ext::to_bytes(&proposal_id).unwrap()],
//     )
// }

// pub fn empty_txn_payload() -> TransactionPayload {
//     TransactionPayload::ScriptFunction(build_empty_script())
// }

fn stake_to_be_member_function(
    dao_type: TypeTag,
    token_type: TypeTag,
    amount: u128,
    lock_time: u64,
) -> ScriptFunction {
    let args = vec![
        bcs_ext::to_bytes(&amount).unwrap(),
        bcs_ext::to_bytes(&lock_time).unwrap(),
    ];
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("StakeToSBTPlugin").unwrap(),
        ),
        Identifier::new("stake_entry").unwrap(),
        vec![dao_type, token_type],
        args,
    )
}

fn block_from_metadata(block_meta: BlockMetadata, chain_state: &ChainStateDB) -> Result<Block> {
    let (parent_hash, timestamp, author, _author_auth_key, _, number, _, _) =
        block_meta.into_inner();
    let block_body = BlockBody::new(vec![], None);
    let block_header = BlockHeader::new(
        parent_hash,
        timestamp,
        number,
        author,
        HashValue::random(),
        HashValue::random(),
        chain_state.state_root(),
        0u64,
        U256::zero(),
        block_body.hash(),
        chain_state.get_chain_id()?,
        0,
        BlockHeaderExtra::new([0u8; 4]),
        None,
    );
    Ok(Block::new(block_header, block_body))
}

pub fn starcoin_dao_type_tag() -> TypeTag {
    TypeTag::Struct(Box::new(StructTag {
        address: genesis_address(),
        module: Identifier::new("StarcoinDAO").unwrap(),
        name: Identifier::new("StarcoinDAO").unwrap(),
        type_params: vec![],
    }))
}

pub fn execute_create_account(
    chain_state: &ChainStateDB,
    net: &ChainNetwork,
    alice: &Account,
    pre_mint_amount: u128,
) -> Result<()> {
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

fn execute_block(
    net: &ChainNetwork,
    chain_state: &ChainStateDB,
    account: &Account,
    parent_hash: HashValue,
    block_number: u64,
    block_timestamp: u64,
) -> Result<Block> {
    let block_meta = BlockMetadata::new(
        parent_hash,
        block_timestamp,
        *account.address(),
        Some(account.auth_key()),
        0,
        block_number,
        net.chain_id(),
        0,
    );
    blockmeta_execute(chain_state, block_meta.clone())?;
    let _ = chain_state.commit();
    chain_state.flush()?;
    block_from_metadata(block_meta, chain_state)
}

// Vote methods use in daospace-v12, master not use it
// The proposal process is based on:
// https://github.com/starcoinorg/starcoin-framework/blob/daospace-v12/integration-tests/starcoin_dao/starcoin_upgrade_module.move
pub fn dao_vote_test(
    alice: &Account,
    chain_state: &ChainStateDB,
    net: &ChainNetwork,
    vote_script: ScriptFunction,
    execute_script: ScriptFunction,
    proposal_id: u64,
) -> Result<()> {
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let one_day: u64 = 60 * 60 * 24 * 1000;
    let alice_balance: u128 = pre_mint_amount / 4;
    let proposal_deposit_amount: u128 = min_proposal_deposit(chain_state, starcoin_dao_type_tag());
    let stake_amount = alice_balance - proposal_deposit_amount - 10_000_000_000;
    // Block 1
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block_meta = BlockMetadata::new(
        HashValue::zero(),
        block_timestamp,
        association_address(),
        None,
        0,
        block_number,
        net.chain_id(),
        0,
    );
    blockmeta_execute(chain_state, block_meta.clone())?;
    let block = block_from_metadata(block_meta, chain_state)?;
    execute_create_account(chain_state, net, alice, pre_mint_amount)?;

    // Block 2, stake STC to be a member of StarcoinDAO
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let script_fun = stake_to_be_member_function(
            starcoin_dao_type_tag(),
            stc_type_tag(),
            stake_amount,
            60000u64,
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_fun),
        )?;
    }
    // block 3
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    let snapshot = block.clone();

    // block 5: Block::checkpoint
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let script_fun = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Block").unwrap()),
            Identifier::new("checkpoint_entry").unwrap(),
            vec![],
            vec![],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_fun),
        )?;
    }

    // block 6
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;

    // block 7: Block::update_state_root, UpgradeModulePlugin::create_proposal
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let raw_header = bcs_ext::to_bytes(&snapshot.header())?;
        let script_fun = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Block").unwrap()),
            Identifier::new("update_state_root_entry").unwrap(),
            vec![],
            vec![bcs_ext::to_bytes(&raw_header)?],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_fun),
        )?;

        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(vote_script),
        )?;
        let state = proposal_state(chain_state, proposal_id);
        assert_eq!(state, PENDING);
    }

    // block: get snapshot proof and DAOSpace::cast_vote_entry
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp =
        block_timestamp + voting_delay(chain_state, starcoin_dao_type_tag()) + 10000;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    let access_path_bytes = snapshot_access_path(chain_state, alice.address());
    let access_path_str = std::str::from_utf8(&access_path_bytes)?;
    let access_path = AccessPath::from_str(access_path_str)?;
    let proof = get_with_proof_by_root(chain_state, access_path, snapshot.header.state_root())?;
    execute_cast_vote(
        chain_state,
        alice,
        proposal_id,
        proof,
        starcoin_dao_type_tag(),
        1u8,
    )?;

    // block: check proposal state.
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp =
        block_timestamp + voting_period(chain_state, starcoin_dao_type_tag()) - 10 * 1000;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    let state = proposal_state(chain_state, proposal_id);
    assert_eq!(state, ACTIVE);

    // block: DAOSpace::queue_proposal_action
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + 20 * 1000;
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let state = proposal_state(chain_state, proposal_id);
        assert_eq!(state, AGREED);

        let script_function = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("DAOSpace").unwrap()),
            Identifier::new("queue_proposal_action_entry").unwrap(),
            vec![starcoin_dao_type_tag()],
            vec![bcs_ext::to_bytes(&proposal_id).unwrap()],
        );
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
        let state = proposal_state(chain_state, proposal_id);
        assert_eq!(state, QUEUED);
    }

    // block: UpgradeModulePlugin::execute_proposal
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + min_action_delay(chain_state, starcoin_dao_type_tag());
    let block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let state = proposal_state(chain_state, proposal_id);
        assert_eq!(state, EXECUTABLE);
        account_execute_should_success(
            alice,
            chain_state,
            TransactionPayload::ScriptFunction(execute_script),
        )?;
    }

    // block: EXTRACTED
    let block_number = current_block_number(chain_state) + 1;
    let block_timestamp = block_timestamp + 1000;
    let _block = execute_block(
        net,
        chain_state,
        alice,
        block.id(),
        block_number,
        block_timestamp,
    )?;
    {
        let state = proposal_state(chain_state, proposal_id);
        assert_eq!(state, EXTRACTED);
    }
    Ok(())
}
