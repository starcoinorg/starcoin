// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus::Consensus;
use starcoin_chain_mock::BlockChain;
use starcoin_config::NodeConfig;
use starcoin_executor::{Account, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_transaction_builder::encode_create_account_script;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::association_address;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::block::Block;
use starcoin_types::genesis_config::{ChainId, ChainNetwork};
use starcoin_types::transaction::{
    Script, SignedUserTransaction, TransactionArgument, TransactionPayload,
};
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::on_chain_config::{consensus_config_type_tag, GlobalTimeOnChain};
use starcoin_vm_types::transaction::RawUserTransaction;
use std::sync::Arc;
use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};
use test_helper::dao::{
    execute_script_on_chain_config, min_action_delay, on_chain_config_type_tag, proposal_state,
    quorum_vote, reward_config_type_tag, vote_reward_scripts, vote_script_consensus, voting_delay,
    voting_period, ACTIVE, AGREED, EXECUTABLE, EXTRACTED, PENDING, QUEUED,
};
use test_helper::executor::{get_balance, get_sequence_number};

pub fn create_new_block(
    chain: &BlockChain,
    address: AccountAddress,
    txns: Vec<SignedUserTransaction>,
) -> Result<Block> {
    let (template, _) = chain.create_block_template(address, None, None, txns, vec![], None)?;
    chain
        .consensus()
        .create_block(chain, template, chain.time_service().as_ref())
}

pub fn build_transaction(
    user_address: AccountAddress,
    seq_number: u64,
    payload: TransactionPayload,
    expire_time: u64,
) -> RawUserTransaction {
    RawUserTransaction::new(
        user_address,
        seq_number,
        payload,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expire_time + 60 * 60,
        ChainId::test(),
    )
}

fn create_user_txn(
    address: AccountAddress,
    seq_number: u64,
    net: &ChainNetwork,
    alice: &Account,
    bob: &Account,
    pre_mint_amount: u128,
    expire_time: u64,
) -> Result<Vec<SignedUserTransaction>> {
    let script = encode_create_account_script(
        net.stdlib_version(),
        stc_type_tag(),
        alice.address(),
        alice.auth_key(),
        pre_mint_amount / 4,
    );
    let script2 = encode_create_account_script(
        net.stdlib_version(),
        stc_type_tag(),
        bob.address(),
        bob.auth_key(),
        pre_mint_amount / 4,
    );
    let txn = net
        .genesis_config()
        .sign_with_association(build_transaction(
            address,
            seq_number,
            TransactionPayload::Script(script),
            expire_time + 60 * 60,
        ))?;
    let txn2 = net
        .genesis_config()
        .sign_with_association(build_transaction(
            address,
            seq_number + 1,
            TransactionPayload::Script(script2),
            expire_time + 60 * 60,
        ))?;
    Ok(vec![txn, txn2])
}

fn create_vote_txn(
    alice: &Account,
    seq_number: u64,
    vote_script: Script,
    expire_time: u64,
) -> Result<SignedUserTransaction> {
    let txn = alice.sign_txn(build_transaction(
        *alice.address(),
        seq_number,
        TransactionPayload::Script(vote_script),
        expire_time,
    ));
    Ok(txn)
}

fn cast_vote_txn(
    seq_number: u64,
    alice: &Account,
    action_type_tag: TypeTag,
    voting_power: u128,
    net: &ChainNetwork,
    expire_time: u64,
) -> Result<SignedUserTransaction> {
    let script =
        compiled_transaction_script(net.stdlib_version(), StdlibScript::CastVote).into_vec();
    let proposer_id = 0;
    println!("bob voting power: {}", voting_power);
    let vote_script = Script::new(
        script,
        vec![stc_type_tag(), action_type_tag],
        vec![
            TransactionArgument::Address(*alice.address()),
            TransactionArgument::U64(proposer_id),
            TransactionArgument::Bool(true),
            TransactionArgument::U128(voting_power / 2),
        ],
    );
    let txn = alice.sign_txn(build_transaction(
        *alice.address(),
        seq_number,
        TransactionPayload::Script(vote_script),
        expire_time,
    ));
    Ok(txn)
}

fn queue_txn(
    seq_number: u64,
    alice: &Account,
    net: &ChainNetwork,
    action_type_tag: TypeTag,
    expire_time: u64,
) -> Result<SignedUserTransaction> {
    let script =
        compiled_transaction_script(net.stdlib_version(), StdlibScript::QueueProposalAction)
            .into_vec();
    let script = Script::new(
        script,
        vec![stc_type_tag(), action_type_tag],
        vec![
            TransactionArgument::Address(*alice.address()),
            TransactionArgument::U64(0),
        ],
    );
    let txn = alice.sign_txn(build_transaction(
        *alice.address(),
        seq_number,
        TransactionPayload::Script(script),
        expire_time,
    ));
    Ok(txn)
}

fn execute_txn(
    seq_number: u64,
    alice: &Account,
    execute_script: Script,
    expire_time: u64,
) -> Result<SignedUserTransaction> {
    let txn = alice.sign_txn(build_transaction(
        *alice.address(),
        seq_number,
        TransactionPayload::Script(execute_script),
        expire_time,
    ));
    Ok(txn)
}

pub fn modify_on_chain_config_by_dao_block(
    alice: Account,
    bob: Account,
    mut chain: BlockChain,
    net: &ChainNetwork,
    vote_script: Script,
    action_type_tag: TypeTag,
    execute_script: Script,
) -> Result<BlockChain> {
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let one_day: u64 = 60 * 60 * 24 * 1000;
    let address = association_address();

    // Block 1
    let block_number = 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let chain_state = chain.chain_state();
    let seq = get_sequence_number(address, chain_state);
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });

        let (template, _) = chain.create_block_template(
            address,
            None,
            None,
            create_user_txn(
                address,
                seq,
                net,
                &alice,
                &bob,
                pre_mint_amount,
                block_timestamp / 1000,
            )?,
            vec![],
            None,
        )?;
        let block1 =
            chain
                .consensus()
                .create_block(&chain, template, chain.time_service().as_ref())?;

        chain.apply(block1)?;
    }

    // block 2
    let block_number = 2;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    let chain_state = chain.chain_state();
    let alice_seq = get_sequence_number(*alice.address(), chain_state);
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        let block2 = create_new_block(
            &chain,
            *alice.address(),
            vec![create_vote_txn(
                &alice,
                alice_seq,
                vote_script,
                block_timestamp / 1000,
            )?],
        )?;
        chain.apply(block2)?;

        let chain_state = chain.chain_state();
        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, PENDING);
    }

    // block 3
    //voting delay
    let chain_state = chain.chain_state();
    let voting_power = get_balance(*alice.address(), chain_state);
    let alice_seq = get_sequence_number(*alice.address(), chain_state);
    let block_timestamp =
        block_timestamp + voting_delay(chain_state.as_super(), stc_type_tag()) * 1000 + 10000;
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        let block3 = create_new_block(
            &chain,
            *alice.address(),
            vec![cast_vote_txn(
                alice_seq,
                &alice,
                action_type_tag.clone(),
                voting_power,
                net,
                block_timestamp / 1000,
            )?],
        )?;
        chain.apply(block3)?;
    }
    // block 4
    let chain_state = chain.chain_state();
    let block_timestamp =
        block_timestamp + voting_period(chain_state.as_super(), stc_type_tag()) * 1000 - 10000;
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        let block4 = create_new_block(&chain, *alice.address(), vec![])?;
        chain.apply(block4)?;
        let chain_state = chain.chain_state();
        let quorum = quorum_vote(chain_state.as_super(), stc_type_tag());
        println!("quorum: {}", quorum);

        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, ACTIVE);
    }

    // block 5
    let block_timestamp = block_timestamp + 20 * 1000;
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        chain.apply(create_new_block(&chain, *alice.address(), vec![])?)?;
        let chain_state = chain.chain_state();
        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, AGREED);
    }

    // block 6
    let chain_state = chain.chain_state();
    let alice_seq = get_sequence_number(*alice.address(), chain_state);
    let block_timestamp = block_timestamp + 20 * 1000;
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        let block6 = create_new_block(
            &chain,
            *alice.address(),
            vec![queue_txn(
                alice_seq,
                &alice,
                net,
                action_type_tag.clone(),
                block_timestamp / 1000,
            )?],
        )?;
        chain.apply(block6)?;
        let chain_state = chain.chain_state();
        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, QUEUED);
    }

    // block 7
    let chain_state = chain.chain_state();
    let block_timestamp =
        block_timestamp + min_action_delay(chain_state.as_super(), stc_type_tag()) * 1000;
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        chain.apply(create_new_block(&chain, *alice.address(), vec![])?)?;
        let chain_state = chain.chain_state();
        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, EXECUTABLE);
    }

    let chain_state = chain.chain_state();
    let alice_seq = get_sequence_number(*alice.address(), chain_state);
    {
        let block8 = create_new_block(
            &chain,
            *alice.address(),
            vec![execute_txn(
                alice_seq,
                &alice,
                execute_script,
                block_timestamp / 1000,
            )?],
        )?;
        chain.apply(block8)?;
    }

    // block 9
    let block_timestamp = block_timestamp + 1000;
    let _chain_state = chain.chain_state();
    {
        chain.time_service().adjust(GlobalTimeOnChain {
            milliseconds: block_timestamp,
        });
        chain.apply(create_new_block(&chain, *alice.address(), vec![])?)?;
        let chain_state = chain.chain_state();
        let state = proposal_state(
            chain_state.as_super(),
            stc_type_tag(),
            action_type_tag,
            *alice.address(),
            0,
        );
        assert_eq!(state, EXTRACTED);
    }

    // return chain state for verify
    Ok(chain)
}

#[stest::test]
fn test_modify_on_chain_config_reward_by_dao() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let net = config.net();
    let chain = test_helper::gen_blockchain_for_test(net)?;
    let alice = Account::new();
    let bob = Account::new();
    let action_type_tag = reward_config_type_tag();
    let reward_delay: u64 = 10;

    modify_on_chain_config_by_dao_block(
        alice,
        bob,
        chain,
        net,
        vote_reward_scripts(&net, reward_delay),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(net, action_type_tag, 0u64),
    )?;
    Ok(())
}

#[stest::test(timeout = 120)]
fn test_modify_on_chain_config_consensus_by_dao() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let net = config.net();
    let chain = test_helper::gen_blockchain_for_test(net)?;
    let alice = Account::new();
    let bob = Account::new();
    let action_type_tag = consensus_config_type_tag();
    let strategy = 3u8;
    let mut modified_chain = modify_on_chain_config_by_dao_block(
        alice,
        bob.clone(),
        chain,
        net,
        vote_script_consensus(&net, strategy),
        on_chain_config_type_tag(action_type_tag.clone()),
        execute_script_on_chain_config(net, action_type_tag, 0u64),
    )?;

    // add block to switch epoch
    let epoch = modified_chain.epoch_info()?;
    let mut number = epoch.end_block_number()
        - epoch.start_block_number()
        - modified_chain.current_header().number;
    dbg!(number);
    while number > 0 {
        modified_chain.apply(create_new_block(&modified_chain, *bob.address(), vec![])?)?;
        number -= 1;
    }

    assert_eq!(modified_chain.consensus().value(), strategy);
    Ok(())
}
