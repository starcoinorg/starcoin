// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::{
    create_writeable_block_chain, gen_blocks, new_block, WriteBlockChainService,
};
use anyhow::Result;
use chain::BlockChain;
use config::NodeConfig;
use consensus::Consensus;
use logger::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain_mock::MockChain;
use starcoin_crypto::HashValue;
use starcoin_storage::Store;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::BlockHeader;
use starcoin_types::{block::Block, U256};
use starcoin_vm_types::genesis_config::{
    BuiltinNetworkID, ChainNetwork, ConsensusStrategy, TEST_CONFIG,
};
use starcoin_vm_types::time::duration_since_epoch;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use traits::WriteableChainService;
use traits::{ChainReader, ChainWriter};

async fn new_block_and_master_with_halley() -> (Block, MockChain) {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let mut mock_chain = MockChain::new(net).unwrap();
    let times = 5;
    mock_chain.produce_and_apply_times(times).unwrap();
    let new_block = mock_chain.produce().unwrap();
    (new_block, mock_chain)
}

async fn new_block_and_master() -> (Block, BlockChain) {
    let times = 5;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    let head_id = writeable_block_chain_service
        .get_master()
        .current_header()
        .id();
    let master = BlockChain::new(net.time_service(), head_id, storage).unwrap();
    let new_block = new_block(
        None,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );
    (new_block, master)
}

async fn uncle_block_and_writeable_block_chain(
    count: u64,
    uncle: u64,
) -> (
    BlockHeader,
    WriteBlockChainService<MockTxPoolService>,
    Arc<NodeConfig>,
    Arc<dyn Store>,
) {
    // 1. chain
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        count,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );

    // 2. new branch and uncle block
    let miner_account = AccountInfo::random();
    let tmp_head = writeable_block_chain_service
        .get_master()
        .get_header_by_number(uncle)
        .unwrap()
        .unwrap()
        .id();

    let new_branch = BlockChain::new(net.time_service(), tmp_head, storage.clone()).unwrap();
    let (block_template, _) = new_branch
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.auth_key()),
            None,
            Vec::new(),
            vec![],
            None,
        )
        .unwrap();
    let new_block = writeable_block_chain_service
        .get_master()
        .consensus()
        .create_block(&new_branch, block_template, net.time_service().as_ref())
        .unwrap();
    let uncle_block_header = new_block.header().clone();
    (
        uncle_block_header,
        writeable_block_chain_service,
        node_config,
        storage,
    )
}

fn apply_with_illegal_uncle(
    net: &ChainNetwork,
    uncles: Vec<BlockHeader>,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    storage: Arc<dyn Store>,
) -> Result<Block> {
    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.auth_key()),
            None,
            Vec::new(),
            uncles,
            None,
        )?;
    let consensus_strategy = writeable_block_chain_service.get_master().consensus();
    let new_block = consensus_strategy.create_block(
        writeable_block_chain_service.get_master(),
        block_template,
        net.time_service().as_ref(),
    )?;

    let head_id = writeable_block_chain_service
        .get_master()
        .current_header()
        .id();
    let mut master = BlockChain::new(net.time_service(), head_id, storage)?;
    master.apply(new_block.clone())?;
    Ok(new_block)
}

fn apply_legal_block(
    consensus_strategy: ConsensusStrategy,
    uncles: Vec<BlockHeader>,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
) {
    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.auth_key()),
            None,
            Vec::new(),
            uncles,
            None,
        )
        .unwrap();
    let new_block = consensus_strategy
        .create_block(
            writeable_block_chain_service.get_master(),
            block_template,
            writeable_block_chain_service
                .get_master()
                .time_service()
                .as_ref(),
        )
        .unwrap();
    writeable_block_chain_service
        .try_connect(new_block)
        .unwrap();
}

async fn test_verify_gas_limit(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.gas_used = u64::MAX;
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_gas_limit_failed() {
    assert!(test_verify_gas_limit(true).await.is_ok());
    let apply_failed = test_verify_gas_limit(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_body_hash(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.body_hash = HashValue::random();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_body_hash_failed() {
    assert!(test_verify_body_hash(true).await.is_ok());
    let apply_failed = test_verify_body_hash(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_parent_id(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.parent_hash = HashValue::random();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_parent_id_failed() {
    assert!(test_verify_parent_id(true).await.is_ok());
    let apply_failed = test_verify_parent_id(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_parent_not_exist() {
    // TODO
}

async fn test_verify_timestamp(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.timestamp = master.current_header().timestamp();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_timestamp_failed() {
    assert!(test_verify_timestamp(true).await.is_ok());
    let apply_failed = test_verify_timestamp(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_future_timestamp(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 1000;
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_future_timestamp_failed() {
    assert!(test_verify_future_timestamp(true).await.is_ok());
    let apply_failed = test_verify_future_timestamp(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_consensus(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master_with_halley().await;
    if !succ {
        new_block.header.difficulty = U256::from(1024u64);
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_consensus_failed() {
    assert!(test_verify_consensus(true).await.is_ok());
    let apply_failed = test_verify_consensus(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_new_epoch_block_uncle_should_none_failed() {
    let apply_failed = test_verify_uncles_in_old_epoch(true).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_can_not_be_uncle_is_member_failed() {
    let times = 5;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );

    let uncle_header = writeable_block_chain_service
        .get_master()
        .get_header_by_number(times - 2)
        .unwrap()
        .unwrap();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_can_not_be_uncle_check_ancestor_failed() {
    // 1. chain
    let times = 7;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    let net = node_config.net();
    gen_blocks(
        times,
        &mut writeable_block_chain_service,
        net.time_service().as_ref(),
    );

    // 2. new branch
    let miner_account = AccountInfo::random();
    let tmp_head = writeable_block_chain_service
        .get_master()
        .get_header_by_number(times - 3)
        .unwrap()
        .unwrap()
        .id();
    let mut new_branch = BlockChain::new(net.time_service(), tmp_head, storage.clone()).unwrap();

    for _i in 0..2 {
        let (block_template, _) = new_branch
            .create_block_template(
                *miner_account.address(),
                Some(miner_account.public_key.auth_key()),
                None,
                Vec::new(),
                vec![],
                None,
            )
            .unwrap();
        let new_block = new_branch
            .consensus()
            .create_block(&new_branch, block_template, net.time_service().as_ref())
            .unwrap();
        new_branch.apply(new_block).unwrap();
    }

    // 3. new block
    let uncle_header = new_branch.current_header();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_illegal_uncle_future_timestamp(succ: bool) -> Result<Block> {
    let count = 5;
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    if !succ {
        uncle_header.timestamp = (duration_since_epoch().as_millis() + 1000) as u64;
    }
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    )
}

#[stest::test(timeout = 120)]
async fn test_verify_illegal_uncle_future_timestamp_failed() {
    assert!(test_verify_illegal_uncle_future_timestamp(true)
        .await
        .is_ok());
    let apply_failed = test_verify_illegal_uncle_future_timestamp(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_illegal_uncle_consensus(succ: bool) -> Result<()> {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let mut mock_chain = MockChain::new(net.clone()).unwrap();
    let mut times = 3;
    mock_chain.produce_and_apply_times(times).unwrap();

    // 1. new branch head id
    let fork_id = mock_chain.head().current_header().id();
    times = 2;
    mock_chain.produce_and_apply_times(times).unwrap();

    // 2. fork new branch and create a uncle block
    let fork_block_chain = mock_chain.fork_new_branch(Some(fork_id)).unwrap();
    let miner = mock_chain.miner();
    let (block_template, _) = fork_block_chain
        .create_block_template(
            *miner.address(),
            Some(miner.public_key.auth_key()),
            None,
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
    let uncle_block = fork_block_chain
        .consensus()
        .create_block(
            &fork_block_chain,
            block_template,
            net.time_service().as_ref(),
        )
        .unwrap();
    let mut uncle_block_header = uncle_block.header().clone();

    if !succ {
        uncle_block_header.nonce = 0;
    }

    // 3. master and create a new block with uncle block
    let mut uncles = Vec::new();
    uncles.push(uncle_block_header);
    let mut master_block_chain = mock_chain.fork_new_branch(None).unwrap();
    let (block_template, _) = master_block_chain
        .create_block_template(
            *miner.address(),
            Some(miner.public_key.auth_key()),
            None,
            Vec::new(),
            uncles,
            None,
        )
        .unwrap();
    let new_block = master_block_chain
        .consensus()
        .create_block(
            &master_block_chain,
            block_template,
            net.time_service().as_ref(),
        )
        .unwrap();

    master_block_chain.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_illegal_uncle_consensus_failed() {
    assert!(test_verify_illegal_uncle_consensus(true).await.is_ok());
    let apply_failed = test_verify_illegal_uncle_consensus(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_state_root(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.state_root = HashValue::random();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_state_root_failed() {
    assert!(test_verify_state_root(true).await.is_ok());
    let apply_failed = test_verify_state_root(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_block_used_gas(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.gas_used = 1;
    }
    master.apply(new_block)
}

#[stest::test(timeout = 360)]
async fn test_verify_block_used_gas_failed() {
    assert!(test_verify_block_used_gas(true).await.is_ok());
    let apply_failed = test_verify_block_used_gas(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_txn_count_failed() {
    // TODO: fix me
    let (mut new_block, mut master) = new_block_and_master().await;
    let mut txns = Vec::new();
    txns.push(SignedUserTransaction::mock());
    let mut body = new_block.body.clone();
    body.transactions = txns;
    new_block.body = body;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_accumulator_root(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.accumulator_root = HashValue::random();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_accumulator_root_failed() {
    assert!(test_verify_accumulator_root(true).await.is_ok());
    let apply_failed = test_verify_accumulator_root(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_block_accumulator_root(succ: bool) -> Result<()> {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        new_block.header.parent_block_accumulator_root = HashValue::random();
    }
    master.apply(new_block)
}

#[stest::test(timeout = 120)]
async fn test_verify_block_accumulator_root_failed() {
    assert!(test_verify_block_accumulator_root(true).await.is_ok());
    let apply_failed = test_verify_block_accumulator_root(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_block_number_failed(succ: bool, order: bool) {
    let (mut new_block, mut master) = new_block_and_master().await;
    if !succ {
        if order {
            new_block.header.number -= 1;
        } else {
            new_block.header.number += 1;
        }
    }
    let apply_failed = master.apply(new_block);
    if !succ {
        assert!(apply_failed.is_err());
        if let Err(apply_err) = apply_failed {
            error!("apply failed : {:?}", apply_err);
        }
    } else {
        assert!(apply_failed.is_ok());
    }
}

#[stest::test(timeout = 120)]
async fn test_verify_block_illegal_number_failed() {
    test_verify_block_number_failed(true, false).await;
    test_verify_block_number_failed(false, false).await;
    test_verify_block_number_failed(false, true).await;
}

async fn test_verify_uncles_count(succ: bool) -> Result<Block> {
    let count = 5;
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    let mut uncles = Vec::new();
    let times = if succ { 2 } else { 3 };
    for _i in 0..times {
        let mut tmp = uncle_header.clone();
        tmp.state_root = HashValue::random();
        uncles.push(tmp);
    }
    apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    )
}

#[stest::test(timeout = 240)]
async fn test_verify_uncles_count_failed() {
    assert!(test_verify_uncles_count(true).await.is_ok());
    let apply_failed = test_verify_uncles_count(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_uncles_number(succ: bool) -> Result<Block> {
    let count = 5;
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    if !succ {
        uncle_header.number = writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
            + 1;
    }
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    )
}

#[stest::test]
async fn test_verify_uncles_number_failed() {
    assert!(test_verify_uncles_number(true).await.is_ok());
    let apply_failed = test_verify_uncles_number(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_uncles_in_old_epoch(begin_epoch: bool) -> Result<Block> {
    let count = 5;
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;

    let end_number = if begin_epoch {
        TEST_CONFIG.consensus_config.epoch_block_count - 1
    } else {
        TEST_CONFIG.consensus_config.epoch_block_count + 1
    };
    let old_epoch_num = writeable_block_chain_service
        .get_master()
        .epoch_info()
        .unwrap()
        .number();
    // create block loop
    loop {
        apply_legal_block(
            writeable_block_chain_service.get_master().consensus(),
            Vec::new(),
            &mut writeable_block_chain_service,
        );
        let block_number = writeable_block_chain_service
            .get_master()
            .current_header()
            .number();
        if block_number == end_number {
            let epoch_info = writeable_block_chain_service
                .get_master()
                .epoch_info()
                .unwrap();
            if begin_epoch {
                assert_eq!(old_epoch_num, epoch_info.number());
                assert_eq!(block_number + 1, epoch_info.end_block_number());
            } else {
                assert_eq!(old_epoch_num + 1, epoch_info.number());
            }
            break;
        }
    }

    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    )
}

#[stest::test]
async fn test_verify_uncles_in_old_epoch_failed() {
    let apply_failed = test_verify_uncles_in_old_epoch(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_uncles_uncle_exist_failed() {
    let count = 5;
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    let net = node_config.net();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    info!(
        "number 1 : {}",
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
    );

    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.auth_key()),
            None,
            Vec::new(),
            uncles.clone(),
            None,
        )
        .unwrap();
    let new_block = writeable_block_chain_service
        .get_master()
        .consensus()
        .create_block(
            writeable_block_chain_service.get_master(),
            block_template,
            net.time_service().as_ref(),
        )
        .unwrap();
    writeable_block_chain_service
        .try_connect(new_block)
        .unwrap();

    info!(
        "number 2 : {}",
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
    );
    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_some_uncles_in_block_failed() {
    let count = 5;
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    let mut uncles = Vec::new();
    for _i in 0..2 {
        uncles.push(uncle_header.clone());
    }
    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_uncle_and_parent_number_failed() {
    let count = 5;
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 2).await;
    let net = node_config.net();
    uncle_header.number += 1;

    let old_number = writeable_block_chain_service
        .get_master()
        .current_header()
        .number();
    // new block
    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.auth_key()),
            None,
            Vec::new(),
            Vec::new(),
            None,
        )
        .unwrap();
    let new_block = writeable_block_chain_service
        .get_master()
        .consensus()
        .create_block(
            writeable_block_chain_service.get_master(),
            block_template,
            net.time_service().as_ref(),
        )
        .unwrap();
    writeable_block_chain_service
        .try_connect(new_block)
        .unwrap();
    let new_number = writeable_block_chain_service
        .get_master()
        .current_header()
        .number();
    assert_eq!(old_number + 1, new_number);

    let mut uncles = Vec::new();
    uncles.push(uncle_header);

    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_uncle_which_parent_is_end_block_in_last_epoch() {
    let count = TEST_CONFIG.consensus_config.epoch_block_count;
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain(count, count - 1).await;

    let epoch = writeable_block_chain_service
        .get_master()
        .epoch_info()
        .unwrap();
    assert_eq!(
        epoch.start_block_number(),
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
    );

    assert_eq!(epoch.start_block_number(), uncle_header.number());
    let uncle_parent_header = writeable_block_chain_service
        .get_master()
        .get_header(uncle_header.parent_hash())
        .unwrap()
        .unwrap();
    assert_eq!(
        epoch.start_block_number(),
        (uncle_parent_header.number() + 1)
    );

    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_ok());
}
