// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::BlockChain as BlockChainNotMock;
use anyhow::Result;
use consensus::Consensus;
use crypto::HashValue;
use crypto::{ed25519::Ed25519PrivateKey, hash::PlainCryptoHash, Genesis, PrivateKey};
use logger::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain_mock::{BlockChain, MockChain};
use starcoin_config::NodeConfig;
use starcoin_traits::{ChainReader, ChainWriter};
use starcoin_types::account_address;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::filter::Filter;
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionInfo};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::{event::EventKey, vm_status::KeptVMStatus};
use std::sync::Arc;

#[stest::test(timeout = 120)]
fn test_chain_filter_events() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    {
        let evt_key = EventKey::new_from_address(&genesis_address(), 4);
        let event_filter = Filter {
            from_block: 1,
            to_block: 5,
            event_keys: vec![evt_key],
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

#[stest::test(timeout = 120)]
fn test_block_chain_head() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number, times);
}

#[stest::test(timeout = 480)]
fn test_halley_consensus() {
    let mut mock_chain = MockChain::new(&ChainNetwork::HALLEY).unwrap();
    let times = 20;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number, times);
}

#[stest::test(timeout = 240)]
fn test_dev_consensus() {
    let mut mock_chain = MockChain::new(&ChainNetwork::DEV).unwrap();
    let times = 20;
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

#[stest::test(timeout = 120)]
fn test_uncle() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().uncles().is_some());
    assert!(mock_chain
        .head()
        .head_block()
        .uncles()
        .unwrap()
        .contains(&uncle_block_header));
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 1);
}

#[stest::test(timeout = 120)]
fn test_uncle_exist() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();
    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().uncles().is_some());
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

#[stest::test(timeout = 120)]
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
fn test_switch_epoch() {
    let (mut mock_chain, _, uncle_block_header) = gen_uncle();
    let miner = mock_chain.miner().clone();

    // 3. mock chain apply
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header.clone());
    let block = product_a_block(mock_chain.head(), &miner, uncles);
    mock_chain.apply(block).unwrap();
    assert!(mock_chain.head().head_block().uncles().is_some());
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
    assert!(mock_chain.head().head_block().uncles().is_none());
    assert_eq!(mock_chain.head().current_epoch_uncles_size(), 0);
}

#[stest::test(timeout = 480)]
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
        Some(miner_account.public_key.clone()),
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
        let txn = executor::build_transfer_from_association(
            account_address,
            public_key.to_bytes().to_vec(),
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
        Some(miner_account.public_key.clone()),
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
        Some(miner_account.public_key.clone()),
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

#[stest::test]
fn test_verify_txn() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    mock_chain.produce_and_apply_times(10).unwrap();
    let block = mock_chain.head().head_block();
    let master_read = BlockChain::new(
        mock_chain.head().consensus(),
        block.header().parent_hash(),
        mock_chain.head().get_storage(),
    )
    .unwrap();
    let mut master_write = BlockChain::new(
        mock_chain.head().consensus(),
        block.header().parent_hash(),
        mock_chain.head().get_storage(),
    )
    .unwrap();
    let result = master_write.apply_without_execute(block, master_read.chain_state_reader());
    assert!(result.is_ok());
}

fn verify_txn_failed(txns: &[Transaction]) {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    mock_chain.produce_and_apply_times(10).unwrap();
    let header = mock_chain.head().current_header();
    let master = BlockChainNotMock::new(
        mock_chain.head().consensus(),
        header.parent_hash(),
        mock_chain.head().get_storage(),
    )
    .unwrap();
    let result = master.verify_txns_for_test(header.id(), txns);
    assert!(result.is_err());
    error!("verify txns failed : {:?}", result);
}

#[stest::test]
fn test_verify_txn_len() {
    verify_txn_failed(Vec::new().as_slice())
}

#[stest::test]
fn test_verify_txn_hash() {
    let mut txns = Vec::new();
    txns.push(Transaction::UserTransaction(SignedUserTransaction::mock()));
    verify_txn_failed(txns.as_slice())
}

fn test_save(txn_infos: Option<(Vec<TransactionInfo>, Vec<Vec<ContractEvent>>)>) -> Result<()> {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    mock_chain.produce_and_apply_times(10).unwrap();
    let block = mock_chain.head().head_block();
    let parent_block_header = mock_chain
        .head()
        .get_header(block.header().parent_hash())
        .unwrap()
        .unwrap();
    let block_metadata = block.clone().into_metadata(parent_block_header.gas_used());
    let mut txns = vec![Transaction::BlockMetadata(block_metadata)];
    txns.extend(
        block
            .transactions()
            .iter()
            .cloned()
            .map(Transaction::UserTransaction),
    );
    let mut master = BlockChainNotMock::new(
        mock_chain.head().consensus(),
        block.header().parent_hash(),
        mock_chain.head().get_storage(),
    )
    .unwrap();
    master.save_fot_test(block.id(), txns, txn_infos)
}

#[stest::test]
fn test_save_txn_len_failed() {
    let txn_infos = Vec::new();
    let events = Vec::new();
    let result = test_save(Some((txn_infos, events)));
    assert!(result.is_err());
    error!("verify txns failed : {:?}", result);
}

#[stest::test]
fn test_save_event_len_failed() {
    let mut txn_infos = Vec::new();
    let txn_info = TransactionInfo::new(
        HashValue::random(),
        HashValue::random(),
        Vec::new().as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    txn_infos.push(txn_info);

    let events = Vec::new();
    let result = test_save(Some((txn_infos, events)));
    assert!(result.is_err());
    error!("verify txns failed : {:?}", result);
}

#[stest::test]
fn test_save_succ() {
    let mut txn_infos = Vec::new();
    let txn_info = TransactionInfo::new(
        HashValue::random(),
        HashValue::random(),
        Vec::new().as_slice(),
        100,
        KeptVMStatus::Executed,
    );
    txn_infos.push(txn_info);

    let mut events = Vec::new();
    let mut event = Vec::new();
    let mut data: Vec<u8> = Vec::new();
    data.push(1);
    let e = ContractEvent::new(EventKey::random(), 10, TypeTag::U64, data);
    event.push(e);
    events.push(event);

    let result = test_save(Some((txn_infos, events)));
    assert!(result.is_ok());
}
