// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Ok, Result};
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_mock::MockChain;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::filter::Filter;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::TypeTag;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::language_storage::StructTag;
use std::str::FromStr;

#[stest::test(timeout = 120)]
fn test_chain_filter_events() {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test()).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();

    let event_type_tag = TypeTag::Struct(Box::new(StructTag {
        address: genesis_address(),
        module: Identifier::from_str("Block").unwrap(),
        name: Identifier::from_str("NewBlockEventV2").unwrap(),
        type_params: vec![],
    }));

    // Origin block event index is 4, after https://github.com/starcoinorg/starcoin-framework/pull/42 , Genesis account create more event_handles, so the block event index is 7.
    // So we should use type_tags to filter event, do not dependent on event key.
    // let evt_key = EventKey::new_from_address(&genesis_address(), 7);
    {
        let event_filter = Filter {
            from_block: 1,
            to_block: 5,
            event_keys: vec![],
            addrs: vec![],
            type_tags: vec![event_type_tag.clone()],
            limit: None,
            reverse: false,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 5);
        let evt = evts.first().unwrap();
        assert_eq!(evt.block_number, 1);
        assert_eq!(evt.transaction_index, 0);
        assert_eq!(evt.event.type_tag(), &event_type_tag);
    }

    {
        let event_filter = Filter {
            from_block: 1,
            to_block: 10,
            event_keys: vec![],
            addrs: vec![],
            type_tags: vec![event_type_tag.clone()],
            limit: Some(5),
            reverse: false,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 5);
        let evt = evts.last().unwrap();
        assert_eq!(evt.block_number, 5);
        assert_eq!(evt.transaction_index, 0);
    }
    {
        let event_filter = Filter {
            from_block: 1,
            to_block: 10,
            event_keys: vec![],
            addrs: vec![],
            type_tags: vec![event_type_tag.clone()],
            limit: Some(5),
            reverse: true,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 5);
        let evt = evts.first().unwrap();
        assert_eq!(evt.block_number, 10);
        assert_eq!(evt.transaction_index, 0);
    }

    // test on from_block is 0
    {
        let event_filter = Filter {
            from_block: 0,
            to_block: 10,
            event_keys: vec![],
            addrs: vec![],
            type_tags: vec![event_type_tag.clone()],
            limit: Some(20),
            reverse: true,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 10);
        let evt = evts.first().unwrap();
        assert_eq!(evt.block_number, 10);
        assert_eq!(evt.transaction_index, 0);
    }

    // test on to_block is too large
    {
        let event_filter = Filter {
            from_block: 0,
            to_block: 20,
            event_keys: vec![],
            addrs: vec![],
            type_tags: vec![event_type_tag],
            limit: Some(20),
            reverse: true,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 10);
        let evt = evts.first().unwrap();
        assert_eq!(evt.block_number, 10);
        assert_eq!(evt.transaction_index, 0);
    }
}

#[stest::test]
fn test_block_chain() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    let block = mock_chain.produce()?;
    assert_eq!(block.header().number(), 1);
    mock_chain.apply(block)?;
    assert_eq!(mock_chain.head().current_header().number(), 1);
    let block = mock_chain.produce()?;
    assert_eq!(block.header().number(), 2);
    mock_chain.apply(block)?;
    assert_eq!(mock_chain.head().current_header().number(), 2);
    Ok(())
}

#[stest::test(timeout = 480)]
fn test_halley_consensus() {
    let mut mock_chain =
        MockChain::new(ChainNetwork::new_builtin(BuiltinNetworkID::Halley)).unwrap();
    let times = 20;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number(), times);
}

#[stest::test(timeout = 240)]
fn test_dev_consensus() {
    let mut mock_chain = MockChain::new(ChainNetwork::new_builtin(BuiltinNetworkID::Dev)).unwrap();
    let times = 20;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number(), times);
}

#[stest::test]
fn test_find_ancestor_genesis() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    mock_chain.produce_and_apply_times(3)?;

    let mut mock_chain2 = MockChain::new(ChainNetwork::new_test())?;
    mock_chain2.produce_and_apply_times(4)?;
    let ancestor = mock_chain.head().find_ancestor(mock_chain2.head())?;
    assert!(ancestor.is_some());
    assert_eq!(ancestor.unwrap().number, 0);
    Ok(())
}

#[stest::test]
fn test_find_ancestor_fork() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_builtin(BuiltinNetworkID::DagTest))?;
    mock_chain.produce_and_apply_times(3)?;
    let header = mock_chain.head().current_header().clone();

    let mut mock_chain2 = mock_chain.fork(None)?;
    let last2 = mock_chain2.produce_and_apply_times_for_fork(header.clone(), 3)?;

    let last = mock_chain.produce_and_apply_times_for_fork(header.clone(), 2)?;

    let compare_chain = mock_chain.fork(Some(last.id()))?;
    let compare_chain2 = mock_chain2.fork(Some(last2.id()))?;

    let ancestor = compare_chain.head().find_ancestor(compare_chain2.head())?;
    assert!(ancestor.is_some());
    assert_eq!(ancestor.unwrap().id, header.id());
    Ok(())
}

fn product_a_block_by_tips(
    branch: &BlockChain,
    miner: &AccountInfo,
    uncles: Vec<BlockHeader>,
    parent_hash: Option<HashValue>,
    tips: Vec<HashValue>,
) -> Block {
    let (block_template, _) = branch
        .create_block_template(
            *miner.address(),
            parent_hash,
            Vec::new(),
            uncles,
            None,
            tips,
            HashValue::zero(),
        )
        .unwrap();

    branch
        .consensus()
        .create_block(block_template, branch.time_service().as_ref())
        .unwrap()
}

#[stest::test]
fn test_get_blocks_by_number() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test()).unwrap();
    let blocks = mock_chain
        .head()
        .get_blocks_by_number(None, true, u64::MAX)?;
    assert_eq!(blocks.len(), 1, "at least genesis block should contains.");
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(None, true, u64::MAX)?;
    assert_eq!(blocks.len(), 11);

    let number = blocks.len() as u64;
    let result = mock_chain
        .head()
        .get_blocks_by_number(Some(blocks.len() as u64), true, u64::MAX);
    assert!(
        result.is_err(),
        "result cannot find block by number {}",
        number
    );

    let number = blocks.len() as u64;
    let number = number.saturating_add(2);
    let result = mock_chain
        .head()
        .get_blocks_by_number(Some(number), true, u64::MAX);
    assert!(
        result.is_err(),
        "result cannot find block by number {}",
        number
    );

    let blocks = mock_chain.head().get_blocks_by_number(Some(9), true, 3)?;
    assert_eq!(blocks.len(), 3);

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(Some(0), false, u64::MAX)?;
    assert_eq!(blocks.len(), 11);

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(Some(9), false, u64::MAX)?;
    assert_eq!(blocks.len(), 2);

    let blocks = mock_chain.head().get_blocks_by_number(Some(6), false, 3)?;
    assert_eq!(blocks.len(), 3);

    Ok(())
}

#[stest::test]
fn test_block_chain_for_dag_fork() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;

    // generate the fork chain
    mock_chain.produce_and_apply_times(3).unwrap();
    let fork_id = mock_chain.head().current_header().id();

    // create the dag chain
    mock_chain.produce_and_apply_times(10).unwrap();

    // create the dag chain at the fork chain
    let mut fork_block_chain = mock_chain.fork_new_branch(Some(fork_id)).unwrap();
    let mut other_tips = vec![fork_id];
    for _ in 0..15 {
        let block = product_a_block_by_tips(
            &fork_block_chain,
            mock_chain.miner(),
            Vec::new(),
            other_tips.first().cloned(),
            other_tips.clone(),
        );
        other_tips = vec![block.id()];
        fork_block_chain.apply(block)?;
    }

    Ok(())
}
