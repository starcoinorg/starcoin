// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Ok, Result};
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_mock::MockChain;
use starcoin_config::NodeConfig;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, Genesis, PrivateKey};
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_address;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::filter::Filter;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::TypeTag;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::language_storage::StructTag;
use std::str::FromStr;
use std::sync::Arc;

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

fn gen_uncle() -> (MockChain, BlockChain, BlockHeader) {
    let mut mock_chain =
        MockChain::new(ChainNetwork::new_builtin(BuiltinNetworkID::DagTest)).unwrap();
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

fn product_a_block(branch: &BlockChain, miner: &AccountInfo, uncles: Vec<BlockHeader>) -> Block {
    product_a_block_by_tips(branch, miner, uncles, None, vec![])
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 120)]
fn test_uncle() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. mock chain apply
    let uncles = vec![uncle_block_header.clone()];
    let block = product_a_block(mock_chain.head(), miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().block.uncles().is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .block
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 120)]
fn test_uncle_exist() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    // 3. mock chain apply
    let uncles = vec![uncle_block_header.clone()];
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().block.uncles().is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .block
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);

    // 4. uncle exist
    let uncles = vec![uncle_block_header];
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 120)]
fn test_uncle_son() {
    let (mut mock_chain, mut fork_block_chain, _) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. uncle son
    let uncle_son = product_a_block(&fork_block_chain, miner, Vec::new());
    let uncle_son_block_header = uncle_son.header().clone();
    fork_block_chain.apply(uncle_son).unwrap();

    // 4. mock chain apply
    let uncles = vec![uncle_son_block_header];
    let block = product_a_block(mock_chain.head(), miner, uncles);
    assert!(mock_chain.apply(block).is_err());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 120)]
fn test_random_uncle() {
    let (mut mock_chain, _, _) = gen_uncle();
    let miner = mock_chain.miner();

    // 3. random BlockHeader and apply
    let uncles = vec![BlockHeader::random()];
    let block = product_a_block(mock_chain.head(), miner, uncles);
    assert!(mock_chain.apply(block).is_err());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 480)]
fn test_switch_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();

    // 3. mock chain apply
    let uncles = vec![uncle_block_header.clone()];
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().block.uncles().is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .block
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);

    // 4. block apply
    let begin_number = mock_chain.head().current_header().number();
    let end_number = mock_chain.head().epoch().end_block_number();
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
    assert!(mock_chain.head().head_block().block.uncles().is_none());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 480)]
fn test_uncle_in_diff_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);

    // 3. block apply
    let begin_number = mock_chain.head().current_header().number();
    let end_number = mock_chain.head().epoch().end_block_number();
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
    assert!(mock_chain.head().head_block().block.uncles().is_none());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);

    // 5. mock chain apply
    let uncles = vec![uncle_block_header];
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[ignore = "dag cannot pass it"]
#[stest::test(timeout = 480)]
///             ╭--> b3(t2)
/// Genesis--> b1--> b2(t2)
///
fn test_block_chain_txn_info_fork_mapping() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_dag_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = block_chain.current_header();
    let miner_account = AccountInfo::random();
    let (template_b1, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(header.id()),
        vec![],
        vec![],
        None,
        vec![],
        HashValue::zero(),
    )?;

    let block_b1 = block_chain
        .consensus()
        .create_block(template_b1, config.net().time_service().as_ref())?;
    block_chain.apply(block_b1.clone())?;

    let mut block_chain2 = block_chain.fork(block_b1.id()).unwrap();

    // create transaction
    let pri_key = Ed25519PrivateKey::genesis();
    let public_key = pri_key.public_key();
    let account_address = account_address::from_public_key(&public_key);
    let signed_txn_t2 = {
        let txn = build_transfer_from_association(
            account_address,
            0,
            10000,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let txn_hash = signed_txn_t2.id();
    let (template_b2, excluded) = block_chain.create_block_template(
        *miner_account.address(),
        Some(block_b1.id()),
        vec![signed_txn_t2.clone()],
        vec![],
        None,
        vec![],
        HashValue::zero(),
    )?;
    assert!(excluded.discarded_txns.is_empty(), "txn is discarded.");
    let block_b2 = block_chain
        .consensus()
        .create_block(template_b2, config.net().time_service().as_ref())?;

    block_chain.apply(block_b2.clone())?;
    let (template_b3, excluded) = block_chain2.create_block_template(
        *miner_account.address(),
        Some(block_b1.id()),
        vec![signed_txn_t2],
        vec![],
        None,
        vec![],
        HashValue::zero(),
    )?;
    assert!(excluded.discarded_txns.is_empty(), "txn is discarded.");
    let block_b3 = block_chain2
        .consensus()
        .create_block(template_b3, config.net().time_service().as_ref())?;
    block_chain2.apply(block_b3.clone())?;

    assert_ne!(
        block_chain.get_txn_accumulator().root_hash(),
        block_chain2.get_txn_accumulator().root_hash()
    );

    let vec_txn = block_chain2
        .get_storage()
        .get_transaction_info_ids_by_txn_hash(txn_hash)?;

    assert_eq!(vec_txn.len(), 2);
    let txn_info1 = block_chain.get_transaction_info(txn_hash)?;
    assert!(txn_info1.is_some());
    let txn_info1 = txn_info1.unwrap();
    assert!(vec_txn.contains(&txn_info1.id()));

    let txn_info2 = block_chain2.get_transaction_info(txn_hash)?;
    assert!(txn_info2.is_some());
    let txn_info2 = txn_info2.unwrap();
    assert!(vec_txn.contains(&txn_info2.id()));

    assert_ne!(txn_info1, txn_info2);

    assert_eq!(txn_info1.transaction_hash(), txn_hash);
    assert_eq!(
        txn_info1.block_id(),
        block_b2.id(),
        "txn_info's block id not as expect. {:?}",
        txn_info1
    );

    assert_eq!(txn_info2.transaction_hash(), txn_hash);
    assert_eq!(
        txn_info2.block_id(),
        block_b3.id(),
        "txn_info's block id not as expect. {:?}",
        txn_info2
    );

    Ok(())
}

#[stest::test]
fn test_get_blocks_by_number() -> Result<()> {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test()).unwrap();
    let blocks = mock_chain
        .head()
        .get_blocks_by_number(None, true, u64::max_value())?;
    assert_eq!(blocks.len(), 1, "at least genesis block should contains.");
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(None, true, u64::max_value())?;
    assert_eq!(blocks.len(), 11);

    let number = blocks.len() as u64;
    let result =
        mock_chain
            .head()
            .get_blocks_by_number(Some(blocks.len() as u64), true, u64::max_value());
    assert!(
        result.is_err(),
        "result cannot find block by number {}",
        number
    );

    let number = blocks.len() as u64;
    let number = number.saturating_add(2);
    let result = mock_chain
        .head()
        .get_blocks_by_number(Some(number), true, u64::max_value());
    assert!(
        result.is_err(),
        "result cannot find block by number {}",
        number
    );

    let blocks = mock_chain.head().get_blocks_by_number(Some(9), true, 3)?;
    assert_eq!(blocks.len(), 3);

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(Some(0), false, u64::max_value())?;
    assert_eq!(blocks.len(), 11);

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(Some(9), false, u64::max_value())?;
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
