// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rand::prelude::IndexedRandom;
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_transaction_builder::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::block::Block;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_crypto::{ed25519::Ed25519PrivateKey, Genesis, PrivateKey};
use starcoin_vm2_state_api::AccountStateReader;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_test_helper::{build_transfer_from_association, build_transfer_txn};
use starcoin_vm2_types::account_address;
use starcoin_vm2_types::account_address::AccountAddress as VM2AccountAddress;
use std::collections::HashMap;
use std::sync::Arc;
use test_helper::chain::gen_blockchain_for_test;

fn get_balance(block_chain: &BlockChain, address: &AccountAddress) -> u128 {
    let state_root2 = block_chain.chain_state_reader2().state_root();
    let chain_state2 = ChainStateDB2::new(
        block_chain.get_storage2().clone().into_super_arc(),
        Some(state_root2),
    );
    let reader2 = AccountStateReader::new(&chain_state2);
    let miner_vm2_addr = VM2AccountAddress::from_bytes(address.to_vec()).unwrap();
    reader2.get_balance(&miner_vm2_addr).unwrap_or(0)
}

fn create_block_template_and_apply(
    block_chain: &mut BlockChain,
    miner: &AccountInfo,
    txns: Vec<MultiSignedUserTransaction>,
    config: Arc<NodeConfig>,
) -> Block {
    let (template, excluded) = block_chain
        .create_block_template(
            *miner.address(),
            None,
            txns,
            None,
            None,
            None,
            HashValue::zero(),
        )
        .unwrap();
    assert!(
        excluded.discarded_txns.is_empty(),
        "txn is discarded by miner"
    );
    let block = block_chain
        .consensus()
        .create_block(template, config.net().time_service().as_ref())
        .unwrap();
    block_chain.apply(block.clone()).unwrap();
    block
}

/// What did this test do?
/// 1. create multi accounts, record gas fee used, check whether gas fee in block eqs accumulated gas used of txns
/// 2. mutli account transfer to each other randomly, check if gas fee that miner received matchs previous block's output
/// 3. create a empty block, check if gas fee that miner received matchs previous block's output
#[stest::test(timeout = 480)]
fn test_distribute_transaction_fee() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = gen_blockchain_for_test(config.net())?;
    let miner = AccountInfo::random();

    // 1. create n accounts from accosiation account, record gas used
    let mut accounts = vec![];
    let total_accounts = 10;
    for sequence_number in 0..total_accounts {
        let private_key = Ed25519PrivateKey::genesis();
        let public_key = private_key.public_key();
        let account_address = account_address::from_public_key(&public_key);
        let signed_txn = {
            let txn = build_transfer_from_association(
                account_address,
                sequence_number,
                10000000,
                config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
                config.net(),
            );
            txn.as_signed_user_txn()?.clone()
        };
        accounts.push((private_key, public_key, signed_txn));
    }

    let block = create_block_template_and_apply(
        &mut block_chain,
        &miner,
        accounts
            .iter()
            .map(|(_, _, txn)| MultiSignedUserTransaction::from(txn.clone()))
            .collect(),
        config.clone(),
    );

    let mut gas_used = 0;
    for (_, _, signed_txn) in accounts.iter() {
        let txn_hash = signed_txn.id();
        let txn_info_ids = block_chain
            .get_storage()
            .get_transaction_info_ids_by_txn_hash(txn_hash)?;
        assert_eq!(txn_info_ids.len(), 1, "should have 1 transaction infos");

        gas_used += block_chain
            .get_storage()
            .get_transaction_info(txn_info_ids[0])?
            .unwrap()
            .transaction_info
            .to_v2()
            .unwrap()
            .gas_used();
    }

    assert_eq!(block.header().gas_used(), gas_used);
    assert!(gas_used > 0);
    let balance = get_balance(&block_chain, miner.address());
    // this is second block of chain, genesis do not conatins gas or block reward
    // gas and block reward will be payed in next block prologue
    assert_eq!(balance, 0);

    // 2. mutli account transfer to each other randomly, check if gas fee matchs previous block's output
    let mut sequence_number_map = HashMap::new();
    let mut txns = vec![];
    for _ in 0..total_accounts {
        let numbers: Vec<u32> = (0..total_accounts).map(|x| x as u32).collect();
        let mut rng = rand::rng();
        let selected: Vec<_> = numbers.choose_multiple(&mut rng, 2).cloned().collect();
        let sender_idx = selected[0];
        let receiver_idx = selected[1];
        let sender = &accounts[sender_idx as usize];
        let receiver = &accounts[receiver_idx as usize];

        let sequence_number = sequence_number_map.entry(&sender.1).or_insert(0);
        let raw_txn = build_transfer_txn(
            account_address::from_public_key(&receiver.1),
            account_address::from_public_key(&sender.1),
            *sequence_number as u64,
            1,
            1,
            DEFAULT_MAX_GAS_AMOUNT,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net().chain_id().id(),
        );
        let signed_txn = raw_txn.sign(&sender.0, sender.0.public_key())?.into_inner();
        txns.push(signed_txn);
        *sequence_number += 1;
    }
    let block = create_block_template_and_apply(
        &mut block_chain,
        &miner,
        txns.into_iter().map(|txn| txn.into()).collect(),
        config.clone(),
    );
    let balance = get_balance(&block_chain, miner.address());
    let block_reward = balance - gas_used as u128;
    assert!(block_reward > 0);
    assert!(balance > 0);

    // 3. create a empty block, check previous gas fee
    let gas_used = block.header().gas_used();
    assert!(gas_used > 0);
    create_block_template_and_apply(&mut block_chain, &miner, vec![], config.clone());
    let prev_balance = balance;
    let balance = get_balance(&block_chain, miner.address());
    assert_eq!(balance, prev_balance + block_reward + gas_used as u128);

    Ok(())
}
