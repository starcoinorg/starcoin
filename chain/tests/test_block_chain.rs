// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus::Consensus;
use crypto::{ed25519::Ed25519PrivateKey, Genesis, PrivateKey};
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::Accumulator;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_mock::MockChain;
use starcoin_config::NodeConfig;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_executor::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_address;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::filter::Filter;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::event::EventKey;
use std::sync::Arc;

#[stest::test(timeout = 120)]
fn test_chain_filter_events() {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test()).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    {
        let evt_key = EventKey::new_from_address(&genesis_address(), 4);
        let event_filter = Filter {
            from_block: 1,
            to_block: 5,
            event_keys: vec![evt_key],
            addrs: vec![],
            type_tags: vec![],
            limit: None,
            reverse: false,
        };
        let evts = mock_chain.head().filter_events(event_filter).unwrap();
        assert_eq!(evts.len(), 5);
        let evt = evts.first().unwrap();
        assert_eq!(evt.block_number, 1);
        assert_eq!(evt.transaction_index, 0);
        assert_eq!(evt.event.key(), &evt_key);
    }

    {
        let event_filter = Filter {
            from_block: 1,
            to_block: 10,
            event_keys: vec![EventKey::new_from_address(&genesis_address(), 4)],
            addrs: vec![],
            type_tags: vec![],
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
            event_keys: vec![EventKey::new_from_address(&genesis_address(), 4)],
            addrs: vec![],
            type_tags: vec![],
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
            event_keys: vec![EventKey::new_from_address(&genesis_address(), 4)],
            addrs: vec![],
            type_tags: vec![],
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
            event_keys: vec![EventKey::new_from_address(&genesis_address(), 4)],
            addrs: vec![],
            type_tags: vec![],
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
    let mut mock_chain = MockChain::new(ChainNetwork::new_test())?;
    mock_chain.produce_and_apply_times(3)?;
    let header = mock_chain.head().current_header();
    let mut mock_chain2 = mock_chain.fork(None)?;
    mock_chain.produce_and_apply_times(2)?;
    mock_chain2.produce_and_apply_times(3)?;

    let ancestor = mock_chain.head().find_ancestor(mock_chain2.head())?;
    assert!(ancestor.is_some());
    assert_eq!(ancestor.unwrap().id, header.id());
    Ok(())
}

fn gen_uncle() -> (MockChain, BlockChain, BlockHeader) {
    let mut mock_chain = MockChain::new(ChainNetwork::new_test()).unwrap();
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
        .create_block_template(*miner.address(), None, Vec::new(), uncles, None)
        .unwrap();
    branch
        .consensus()
        .create_block(block_template, branch.time_service().as_ref())
        .unwrap()
}

#[stest::test(timeout = 120)]
#[allow(clippy::vec_init_then_push)]
fn test_uncle() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
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

#[stest::test(timeout = 120)]
#[allow(clippy::vec_init_then_push)]
fn test_uncle_exist() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
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
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[stest::test(timeout = 120)]
#[allow(clippy::vec_init_then_push)]
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

#[stest::test(timeout = 120)]
#[allow(clippy::vec_init_then_push)]
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

#[stest::test(timeout = 480)]
#[allow(clippy::vec_init_then_push)]
fn test_switch_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();

    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
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

#[stest::test(timeout = 480)]
#[allow(clippy::vec_init_then_push)]
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
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    assert!(mock_chain.apply(block).is_err());
}

#[stest::test(timeout = 480)]
///             ╭--> b3(t2)
/// Genesis--> b1--> b2(t2)
///
fn test_block_chain_txn_info_fork_mapping() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = block_chain.current_header();
    let miner_account = AccountInfo::random();
    let (template_b1, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(header.id()),
        vec![],
        vec![],
        None,
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
        .get_blocks_by_number(None, u64::max_value())?;
    assert_eq!(blocks.len(), 1, "at least genesis block should contains.");
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();

    let blocks = mock_chain
        .head()
        .get_blocks_by_number(None, u64::max_value())?;
    assert_eq!(blocks.len(), 11);
    Ok(())
}
