// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus::Consensus;
use starcoin_account_api::AccountInfo;
use starcoin_chain_mock::{BlockChain, MockChain};
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::block::{Block, BlockHeader};
use starcoin_vm_types::genesis_config::ChainNetwork;

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
