// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus::Consensus;
use crypto::{ed25519::Ed25519PrivateKey, hash::PlainCryptoHash, Genesis, PrivateKey};
use starcoin_account_api::AccountInfo;
use starcoin_chain_mock::{BlockChain, MockChain};
use starcoin_config::NodeConfig;
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::account_address;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::genesis_config::ChainNetwork;
use std::sync::Arc;

#[stest::test]
fn test_block_chain_head() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number, times);
}

fn gen_uncle() -> (MockChain, BlockChain, BlockHeader) {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let mut times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();

    // 1. new branch head id
    let fork_id = mock_chain.head().current_header().id();
    times = 2;
    mock_chain.produce_and_apply_times(times).unwrap();

    // 2. fork new branch and create a uncle block
    let mut fork_block_chain = mock_chain.fork_new_branch(Some(fork_id)).unwrap();
    let miner = mock_chain.miner();
    let block = product_a_block(&fork_block_chain, miner, Vec::new());
    let uncle_block_header = block.header().clone();
    fork_block_chain.apply(block).unwrap();
    (mock_chain, fork_block_chain, uncle_block_header)
}

fn product_a_block(branch: &BlockChain, miner: &AccountInfo, uncles: Vec<BlockHeader>) -> Block {
    let (block_template, _) = branch
        .create_block_template(
            *miner.address(),
            Some(miner.public_key.clone()),
            None,
            Vec::new(),
            uncles,
            None,
        )
        .unwrap();
    branch
        .consensus()
        .create_block(branch, block_template)
        .unwrap()
}

#[stest::test]
fn test_uncle() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().current_header().uncle_hash.is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);
}

#[stest::test]
fn test_uncle_exist() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().current_header().uncle_hash.is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);

    // 4. uncle exist
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[stest::test]
fn test_uncle_son() {
    let (mut mock_chain, mut fork_block_chain, _) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. uncle son
    let uncle_son = product_a_block(&fork_block_chain, miner, Vec::new());
    let uncle_son_block_header = uncle_son.header().clone();
    fork_block_chain.apply(uncle_son).unwrap();

    // 4. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_son_block_header);
    let block = product_a_block(mock_chain.head(), miner, uncles);
    assert!(mock_chain.apply(block).is_err());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[stest::test]
fn test_random_uncle() {
    let (mut mock_chain, _, _) = gen_uncle();
    let miner = mock_chain.miner();

    // 3. random BlockHeader and apply
    let mut uncles = Vec::new();
    uncles.push(BlockHeader::random());
    let block = product_a_block(mock_chain.head(), miner, uncles);
    assert!(mock_chain.apply(block).is_err());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[stest::test]
fn test_switch_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();

    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().current_header().uncle_hash.is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);

    // 4. block apply
    let begin_number = mock_chain.head().current_header().number();
    let end_number = mock_chain.head().epoch_info().unwrap().end_number();
    assert!(begin_number < end_number);
    if begin_number < (end_number - 1) {
        for _i in begin_number..(end_number - 1) {
            let block = product_a_block(mock_chain.head(), &miner, Vec::new());
            mock_chain.apply(block).unwrap();
            assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);
        }
    }

    // 5. switch epoch
    let block = product_a_block(mock_chain.head(), &miner, Vec::new());
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().current_header().uncle_hash.is_none());
    assert!(mock_chain.head().head_block().uncles().is_none());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[stest::test]
fn test_uncle_in_diff_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);

    // 3. block apply
    let begin_number = mock_chain.head().current_header().number();
    let end_number = mock_chain.head().epoch_info().unwrap().end_number();
    assert!(begin_number < end_number);
    if begin_number < (end_number - 1) {
        for _i in begin_number..(end_number - 1) {
            let block = product_a_block(mock_chain.head(), &miner, Vec::new());
            mock_chain.apply(block).unwrap();
            assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
        }
    }

    // 4. switch epoch
    let block = product_a_block(mock_chain.head(), &miner, Vec::new());
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().current_header().uncle_hash.is_none());
    assert!(mock_chain.head().head_block().uncles().is_none());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);

    // 5. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[stest::test(timeout = 480)]
///             ╭--> b3(t2)
/// Genesis--> b1--> b2(t2)
///
async fn test_block_chain_txn_info_fork_mapping() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = block_chain.current_header();
    let miner_account = AccountInfo::random();

    let (template_b1, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        Some(header.id()),
        vec![],
        vec![],
        None,
    )?;

    let block_b1 = config
        .net()
        .consensus()
        .create_block(&block_chain, template_b1)?;
    block_chain.apply(block_b1.clone())?;

    let mut block_chain2 = block_chain.new_chain(block_b1.id()).unwrap();

    // create transaction
    let pri_key = Ed25519PrivateKey::genesis();
    let public_key = pri_key.public_key();
    let account_address = account_address::from_public_key(&public_key);
    let signed_txn_t2 = {
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = executor::build_transfer_from_association(
            account_address,
            auth_prefix,
            0,
            10000,
            config.net().consensus().now() + 40000,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let tnx_hash = signed_txn_t2.crypto_hash();
    let (template_b2, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        Some(block_b1.id()),
        vec![signed_txn_t2.clone()],
        vec![],
        None,
    )?;
    let block_b2 = config
        .net()
        .consensus()
        .create_block(&block_chain, template_b2)?;

    block_chain.apply(block_b2)?;
    let (template_b3, _) = block_chain2.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        Some(block_b1.id()),
        vec![signed_txn_t2],
        vec![],
        None,
    )?;
    let block_b3 = config
        .net()
        .consensus()
        .create_block(&block_chain2, template_b3)?;
    block_chain2.apply(block_b3)?;

    let vec_txn = block_chain2
        .get_storage()
        .get_transaction_info_ids_by_hash(tnx_hash)?;

    assert_eq!(vec_txn.len(), 2);
    let txn_info = block_chain.get_transaction_info(tnx_hash)?;
    assert!(txn_info.is_some());
    assert_eq!(txn_info.unwrap().transaction_hash(), tnx_hash);
    Ok(())
}
