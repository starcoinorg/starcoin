// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus::Consensus;
use starcoin_chain_mock::MockChain;
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ChainNetwork;

#[stest::test]
fn test_block_chain_head() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number, times);
}

#[stest::test]
fn test_uncle() {
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
    let (block_template, _) = fork_block_chain
        .create_block_template(
            miner.address().clone(),
            Some(miner.get_auth_key().prefix().to_vec()),
            None,
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
    let uncle_block = block_template.into_block(0, U256::from(1u64));
    let uncle_block_header = uncle_block.header().clone();
    let _ = fork_block_chain.apply(uncle_block);

    // 3. mock chain
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let (block_template, _) = mock_chain
        .head()
        .create_block_template(
            miner.address().clone(),
            Some(miner.get_auth_key().prefix().to_vec()),
            None,
            Vec::new(),
            uncles,
            None,
        )
        .unwrap();
    let block = mock_chain
        .head()
        .consensus()
        .create_block(mock_chain.head(), block_template)
        .unwrap();
    let _ = mock_chain.apply(block);
    assert!(mock_chain.head().current_header().uncle_hash.is_some())
}
