// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::integer_arithmetic)]
use crate::block_connector::{BlockConnectorService, CheckBlockConnectorHashValue};
use crate::tasks::block_sync_task::SyncBlockData;
use crate::tasks::mock::{ErrorStrategy, MockBlockIdFetcher, SyncNodeMocker};
use crate::tasks::{
    full_sync_task, AccumulatorCollector, AncestorCollector, BlockAccumulatorSyncTask,
    BlockCollector, BlockFetcher, BlockLocalStore, BlockSyncTask, FindAncestorTask, SyncFetcher,
};
use crate::verified_rpc_client::RpcVerifyError;
use anyhow::{format_err, Result, anyhow};
use anyhow::{Context, Ok};
use futures::channel::mpsc::unbounded;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures_timer::Delay;
use network_api::{PeerId, PeerInfo, PeerSelector, PeerStrategy};
use pin_utils::core_reexport::time::Duration;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_chain_api::ChainReader;
use starcoin_chain_mock::MockChain;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, ChainNetworkID, NodeConfig};
use starcoin_crypto::HashValue;
use starcoin_flexidag::FlexidagService;
use starcoin_genesis::Genesis;
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::SyncTarget;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::{
    block::{Block, BlockBody, BlockHeaderBuilder, BlockIdAndNumber, BlockInfo},
    U256,
};
use stream_task::TaskHandle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use stest::actix_export::System;
use stream_task::{
    DefaultCustomErrorHandle, Generator, TaskError, TaskEventCounterHandle, TaskGenerator,
};
use test_helper::DummyNetworkService;

use super::BlockConnectedEvent;

#[stest::test(timeout = 120)]
pub async fn test_full_sync_new_node() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender_1, receiver_1) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender_1,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver_1).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(20)?;

    let (sender_1, receiver_1) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    //sync again
    let target = arc_node1.sync_target();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender_1,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver_1).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test]
pub async fn test_sync_invalid_target() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 0)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 1, 0)?;

    let mut target = arc_node1.sync_target();

    target.block_info.total_difficulty = U256::max_value();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender_1, receiver_1) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, _task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender_1,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let _join_handle = node2.process_block_connect_event(receiver_1).await;
    let sync_result = sync_task.await;
    assert!(sync_result.is_err());
    let err = sync_result.err().unwrap();
    debug!("task_error: {:?}", err);
    assert!(err.is_break_error());
    if let TaskError::BreakError(err) = err {
        let verify_err = err.downcast::<RpcVerifyError>().unwrap();
        assert_eq!(verify_err.peers[0].clone(), arc_node1.peer_id);
        debug!("{:?}", verify_err)
    } else {
        panic!("Expect BreakError, but got: {:?}", err)
    }

    Ok(())
}

#[stest::test]
pub async fn test_failed_block() -> Result<()> {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let (storage, chain_info, _) = Genesis::init_storage_for_test(&net)?;

    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        net.id().clone(),
        None,
        None,
    )?;
    let (sender, _) = unbounded();
    let chain_status = chain.status();
    let target = SyncTarget {
        target_id: BlockIdAndNumber::new(chain_status.head.id(), chain_status.head.number()),
        block_info: chain_status.info,
        peers: vec![PeerId::random()],
    };
    let mut block_collector = BlockCollector::new_with_handle(
        chain_info.status().info.clone(),
        Some(target),
        chain,
        sender,
        DummyNetworkService::default(),
        true,
        HashValue::zero(),
        None,
        storage.clone(),
    );
    let header = BlockHeaderBuilder::random().with_number(1).build();
    let body = BlockBody::new(Vec::new(), None);
    let failed_block = Block::new(header, body);
    let failed_block_id = failed_block.id();
    if block_collector
        .apply_block_for_test(failed_block, None, &mut None)
        .is_err()
    {
        assert!(storage.get_failed_block_by_id(failed_block_id)?.is_some());
        Ok(())
    } else {
        Err(format_err!("test FailedBlock fail."))
    }
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_fork() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let mut node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    //test fork

    Arc::get_mut(&mut arc_node1).unwrap().produce_block(10)?;
    node2.produce_block(5)?;

    let (sender, receiver) = unbounded();
    let target = arc_node1.sync_target();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage,
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));
    Ok(())
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_fork_from_genesis() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    //fork from genesis
    let mut node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;
    node2.produce_block(5)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());
    assert_eq!(
        arc_node1.chain().current_header().id(),
        current_block_header.id()
    );
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_continue() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 10, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    //fork from genesis
    let mut node2 = SyncNodeMocker::new(net2.clone(), 1, 50)?;
    node2.produce_block(7)?;

    // first set target to 5.
    let target = arc_node1.sync_target_by_number(5).unwrap();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;

    assert_eq!(branch.current_header().id(), target.target_id.id());
    let current_block_header = node2.chain().current_header();
    // node2's main chain not change.
    assert_ne!(target.target_id.id(), current_block_header.id());

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("task_report: {}", report));

    //set target to latest.
    let target = arc_node1.sync_target();

    let (sender, receiver) = unbounded();
    //continue sync
    //TODO find a way to verify continue sync will reuse previous task local block.
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;

    let join_handle = node2.process_block_connect_event(receiver).await;
    let branch = sync_task.await?;
    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_eq!(branch.current_header().id(), target.target_id.id());
    assert_eq!(target.target_id.id(), current_block_header.id());
    assert_eq!(
        arc_node1.chain().current_header().id(),
        current_block_header.id()
    );
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[stest::test]
pub async fn test_full_sync_cancel() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 10, 50)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let join_handle = node2.process_block_connect_event(receiver).await;
    let sync_join_handle = tokio::task::spawn(sync_task);

    Delay::new(Duration::from_millis(100)).await;

    task_handle.cancel();
    let sync_result = sync_join_handle.await?;
    assert!(sync_result.is_err());
    assert!(sync_result.err().unwrap().is_canceled());

    let node2 = join_handle.await;
    let current_block_header = node2.chain().current_header();
    assert_ne!(target.target_id.id(), current_block_header.id());
    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(())
}

#[ignore]
#[stest::test]
pub async fn test_full_sync_by_total_difficulty() {
    //TODO add a test to verify low block number but high total difficulty.
}

#[stest::test]
async fn test_accumulator_sync_by_stream_task() -> Result<()> {
    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = MerkleAccumulator::new_empty(store.clone());
    for _i in 0..100 {
        accumulator.append(&[HashValue::random()])?;
    }
    accumulator.flush().unwrap();
    let info0 = accumulator.get_info();
    assert_eq!(info0.num_leaves, 100);
    for _i in 0..100 {
        accumulator.append(&[HashValue::random()])?;
    }
    accumulator.flush().unwrap();
    let info1 = accumulator.get_info();
    assert_eq!(info1.num_leaves, 200);
    let fetcher = MockBlockIdFetcher::new(Arc::new(accumulator));
    let store2 = MockAccumulatorStore::copy_from(store.as_ref());

    let task_state =
        BlockAccumulatorSyncTask::new(info0.num_leaves, info1.clone(), fetcher, 7).unwrap();
    let ancestor = BlockIdAndNumber::new(HashValue::random(), info0.num_leaves - 1);
    let collector = AccumulatorCollector::new(Arc::new(store2), ancestor, info0, info1.clone());
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let sync_task = TaskGenerator::new(
        task_state,
        5,
        3,
        1,
        collector,
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let info2 = sync_task.await?.1.get_info();
    assert_eq!(info1, info2);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);
    Ok(())
}

#[stest::test]
pub async fn test_find_ancestor_same_number() -> Result<()> {
    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

    let fetcher = MockBlockIdFetcher::new(accumulator.clone());
    fetcher.appends(generate_hash(100).as_slice())?;
    let info0 = accumulator.get_info();

    let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
    let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));
    let task_state = FindAncestorTask::new(
        accumulator2.num_leaves() - 1,
        accumulator.num_leaves() - 1,
        7,
        fetcher.clone(),
    );
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let collector = AncestorCollector::new(accumulator2.clone());
    let task = TaskGenerator::new(
        task_state,
        5,
        3,
        1,
        collector,
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let ancestor = task.await?;
    assert_eq!(ancestor.number, info0.num_leaves - 1);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);

    Ok(())
}

#[stest::test]
pub async fn test_find_ancestor_block_number_behind() -> Result<()> {
    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

    let fetcher = MockBlockIdFetcher::new(accumulator.clone());
    fetcher.appends(generate_hash(100).as_slice())?;
    let info0 = accumulator.get_info();

    // remote node block id is greater than local.
    fetcher.appends(generate_hash(100).as_slice())?;

    let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
    let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));
    let task_state = FindAncestorTask::new(
        accumulator2.num_leaves() - 1,
        accumulator.num_leaves() - 1,
        7,
        fetcher.clone(),
    );
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let collector = AncestorCollector::new(accumulator2.clone());
    let task = TaskGenerator::new(
        task_state,
        5,
        3,
        1,
        collector,
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let ancestor = task.await?;
    assert_eq!(ancestor.number, info0.num_leaves - 1);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);

    Ok(())
}

fn generate_hash(count: usize) -> Vec<HashValue> {
    (0..count).map(|_| HashValue::random()).collect::<Vec<_>>()
}

#[stest::test]
pub async fn test_find_ancestor_chain_fork() -> Result<()> {
    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = Arc::new(MerkleAccumulator::new_empty(store.clone()));

    let fetcher = MockBlockIdFetcher::new(accumulator.clone());
    fetcher.appends(generate_hash(100).as_slice())?;
    let info0 = accumulator.get_info();

    fetcher.appends(generate_hash(100).as_slice())?;

    let store2 = Arc::new(MockAccumulatorStore::copy_from(store.as_ref()));
    let accumulator2 = Arc::new(MerkleAccumulator::new_with_info(info0.clone(), store2));

    accumulator2.append(generate_hash(100).as_slice())?;
    accumulator2.flush()?;

    assert_ne!(accumulator.get_info(), accumulator2.get_info());
    let batch_size = 7;
    let task_state = FindAncestorTask::new(
        accumulator2.num_leaves() - 1,
        accumulator.num_leaves() - 1,
        batch_size,
        fetcher.clone(),
    );
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let collector = AncestorCollector::new(accumulator2.clone());
    let task = TaskGenerator::new(
        task_state,
        5,
        3,
        1,
        collector,
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let ancestor = task.await?;
    assert_eq!(ancestor.number, info0.num_leaves - 1);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);
    //sub task not all finished, the collector return enough.
    assert!(report.ok < report.sub_task);
    assert_eq!(report.processed_items, 100 + 1);
    Ok(())
}

#[derive(Default)]
struct MockBlockFetcher {
    blocks: Mutex<HashMap<HashValue, Block>>,
}

impl MockBlockFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&self, block: Block) {
        self.blocks.lock().unwrap().insert(block.id(), block);
    }
}

impl BlockFetcher for MockBlockFetcher {
    fn fetch_blocks(
        &self,
        block_ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<(Block, Option<PeerId>)>>> {
        let blocks = self.blocks.lock().unwrap();
        let result: Result<Vec<(Block, Option<PeerId>)>> = block_ids
            .iter()
            .map(|block_id| {
                if let Some(block) = blocks.get(block_id).cloned() {
                    Ok((block, None))
                } else {
                    Err(format_err!("Can not find block by id: {:?}", block_id))
                }
            })
            .collect();
        async {
            Delay::new(Duration::from_millis(100)).await;
            result
        }
        .boxed()
    }
}

fn build_block_fetcher(total_blocks: u64) -> (MockBlockFetcher, MerkleAccumulator) {
    let fetcher = MockBlockFetcher::new();

    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = MerkleAccumulator::new_empty(store);
    for i in 0..total_blocks {
        let header = BlockHeaderBuilder::random().with_number(i).build();
        let block = Block::new(header, vec![]);
        accumulator.append(&[block.id()]).unwrap();
        fetcher.put(block);
    }
    accumulator.flush().unwrap();
    (fetcher, accumulator)
}

#[derive(Default)]
struct MockLocalBlockStore {
    store: Mutex<HashMap<HashValue, SyncBlockData>>,
}

impl MockLocalBlockStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mock(&self, block: &Block) {
        let block_id = block.id();
        let block_info = BlockInfo::new(
            block_id,
            U256::from(1),
            AccumulatorInfo::new(HashValue::random(), vec![], 0, 0),
            AccumulatorInfo::new(HashValue::random(), vec![], 0, 0),
        );
        self.store.lock().unwrap().insert(
            block.id(),
            SyncBlockData::new(block.clone(), Some(block_info), None, None, 1, None),
        );
    }
}

impl BlockLocalStore for MockLocalBlockStore {
    fn get_block_with_info(&self, block_ids: Vec<HashValue>) -> Result<Vec<Option<SyncBlockData>>> {
        let store = self.store.lock().unwrap();
        Ok(block_ids.iter().map(|id| store.get(id).cloned()).collect())
    }
}

async fn block_sync_task_test(total_blocks: u64, ancestor_number: u64) -> Result<()> {
    assert!(
        total_blocks > ancestor_number,
        "total blocks should > ancestor number"
    );
    let (fetcher, accumulator) = build_block_fetcher(total_blocks);
    let ancestor = BlockIdAndNumber::new(
        accumulator
            .get_leaf(ancestor_number)?
            .expect("ancestor should exist"),
        ancestor_number,
    );

    let block_sync_state = BlockSyncTask::new(
        accumulator,
        ancestor,
        fetcher,
        false,
        MockLocalBlockStore::new(),
        3,
    );
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let sync_task = TaskGenerator::new(
        block_sync_state,
        5,
        3,
        1,
        vec![],
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let result = sync_task.await?;
    assert!(!result.is_empty(), "task result is empty.");
    let last_block_number = result
        .iter()
        .map(|block_data| {
            assert!(block_data.info.is_none());
            block_data.block.header().number()
        })
        .fold(ancestor.number, |parent, current| {
            //ensure return block is ordered
            assert_eq!(
                parent + 1,
                current,
                "block sync task not return ordered blocks"
            );
            current
        });

    assert_eq!(last_block_number, total_blocks - 1);

    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);
    Ok(())
}

#[stest::test]
async fn test_block_sync() -> Result<()> {
    block_sync_task_test(100, 0).await
}

#[stest::test]
async fn test_block_sync_one_block() -> Result<()> {
    block_sync_task_test(2, 0).await
}

#[stest::test]
async fn test_block_sync_with_local() -> Result<()> {
    let total_blocks = 100;
    let (fetcher, accumulator) = build_block_fetcher(total_blocks);

    let local_store = MockLocalBlockStore::new();
    fetcher
        .blocks
        .lock()
        .unwrap()
        .iter()
        .for_each(|(_block_id, block)| {
            if block.header().number() % 2 == 0 {
                local_store.mock(block)
            }
        });
    let ancestor_number = 0;
    let ancestor = BlockIdAndNumber::new(
        accumulator.get_leaf(ancestor_number)?.unwrap(),
        ancestor_number,
    );
    let block_sync_state = BlockSyncTask::new(accumulator, ancestor, fetcher, true, local_store, 3);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let sync_task = TaskGenerator::new(
        block_sync_state,
        5,
        3,
        1,
        vec![],
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let result = sync_task.await?;
    let last_block_number = result
        .iter()
        .map(|block_data| {
            if block_data.block.header().number() % 2 == 0 {
                assert!(block_data.info.is_some())
            } else {
                assert!(block_data.info.is_none())
            }
            block_data.block.header().number()
        })
        .fold(ancestor_number, |parent, current| {
            //ensure return block is ordered
            assert_eq!(
                parent + 1,
                current,
                "block sync task not return ordered blocks"
            );
            current
        });

    assert_eq!(last_block_number, total_blocks - 1);

    let report = event_handle.get_reports().pop().unwrap();
    debug!("report: {}", report);
    Ok(())
}

#[stest::test(timeout = 120)]
async fn test_net_rpc_err() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new_with_strategy(net1, ErrorStrategy::MethodNotFound, 50)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new_with_strategy(net2.clone(), ErrorStrategy::MethodNotFound, 50)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let (sender, receiver) = unbounded();
    let (sender_2, _receiver_2) = unbounded();
    let (sync_task, _task_handle, _task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        net2.time_service(),
        storage.clone(),
        sender,
        arc_node1.clone(),
        sender_2,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let _join_handle = node2.process_block_connect_event(receiver).await;
    let sync_join_handle = tokio::task::spawn(sync_task);

    Delay::new(Duration::from_millis(100)).await;

    let sync_result = sync_join_handle.await?;
    assert!(sync_result.is_err());
    Ok(())
}

#[stest::test(timeout = 120)]
async fn test_err_context() -> Result<()> {
    let peer_id = PeerId::random();
    let result = std::fs::read("FileNotExist").with_context(|| peer_id.clone());
    if let Err(error) = result {
        debug!("peer: {:?}", error);
        assert_eq!(peer_id.to_string(), error.to_string());
        let err = error.downcast::<std::io::Error>().unwrap();
        debug!("err{:?}", err);
    }
    Ok(())
}

#[stest::test]
async fn test_sync_target() {
    let mut peer_infos = vec![];
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 1, 0).unwrap();
    node1.produce_block(10).unwrap();
    let low_chain_info = node1.peer_info().chain_info().clone();
    peer_infos.push(PeerInfo::new(
        PeerId::random(),
        low_chain_info.clone(),
        vec![],
        vec![],
        None,
    ));
    node1.produce_block(10).unwrap();
    let high_chain_info = node1.peer_info().chain_info().clone();
    peer_infos.push(PeerInfo::new(
        PeerId::random(),
        high_chain_info.clone(),
        vec![],
        vec![],
        None,
    ));

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let (_, genesis_chain_info, _) =
        Genesis::init_storage_for_test(&net2).expect("init storage by genesis fail.");
    let mock_chain = MockChain::new_with_chain(
        net2,
        node1.chain().fork(high_chain_info.head().id()).unwrap(),
    )
    .unwrap();

    let peer_selector = PeerSelector::new(peer_infos, PeerStrategy::default(), None);
    let node2 = Arc::new(SyncNodeMocker::new_with_chain_selector(
        PeerId::random(),
        mock_chain,
        1,
        0,
        peer_selector,
    ));
    let full_target = node2
        .get_best_target(genesis_chain_info.total_difficulty())
        .unwrap()
        .unwrap();
    let target = node2
        .get_better_target(genesis_chain_info.total_difficulty(), full_target.0, 10, 0)
        .await
        .unwrap();
    assert_eq!(target.peers.len(), 2);
    assert_eq!(target.target_id.number(), low_chain_info.head().number());
    assert_eq!(target.target_id.id(), low_chain_info.head().id());
}

fn sync_block_in_async_connection(
    mut target_node: Arc<SyncNodeMocker>,
    local_node: Arc<SyncNodeMocker>,
    storage: Arc<Storage>,
    block_count: u64,
) -> Result<Arc<SyncNodeMocker>> {
    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_block(block_count)?;
    let target = target_node.sync_target();
    let target_id = target.target_id.id();

    let (sender, mut receiver) = futures::channel::mpsc::unbounded::<BlockConnectedEvent>();
    let thread_local_node = local_node.clone();

    let process_block = move || {
        let mut chain = MockChain::new_with_storage(
            thread_local_node.chain_mocker.net().clone(),
            storage.clone(),
            thread_local_node.chain_mocker.head().status().head.id(),
            thread_local_node.chain_mocker.miner().clone(),
        )
        .unwrap();
        loop {
            if let std::result::Result::Ok(result) = receiver.try_next() {
                match result {
                    Some(event) => {
                        chain
                            .select_head(event.block)
                            .expect("select head must be successful");
                        if event.feedback.is_some() {
                            event
                                .feedback
                                .unwrap()
                                .unbounded_send(super::BlockConnectedFinishEvent)
                                .unwrap();
                            assert_eq!(target_id, chain.head().status().head.id());
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    };
    let handle = std::thread::spawn(process_block);

    let current_block_header = local_node.chain().current_header();
    let storage = local_node.chain().get_storage();

    let local_net = local_node.chain_mocker.net();
    let (local_ancestor_sender, _local_ancestor_receiver) = unbounded();

    let (sync_task, _task_handle, task_event_counter) = full_sync_task(
        current_block_header.id(),
        target.clone(),
        false,
        local_net.time_service(),
        storage.clone(),
        sender,
        target_node.clone(),
        local_ancestor_sender,
        DummyNetworkService::default(),
        15,
        ChainNetworkID::TEST,
        None,
        None,
    )?;
    let branch = async_std::task::block_on(sync_task)?;
    assert_eq!(branch.current_header().id(), target.target_id.id());

    handle.join().unwrap();

    let reports = task_event_counter.get_reports();
    reports
        .iter()
        .for_each(|report| debug!("reports: {}", report));

    Ok(target_node)
}

#[stest::test]
async fn test_sync_block_in_async_connection() -> Result<()> {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut target_node = Arc::new(SyncNodeMocker::new(net.clone(), 1, 0)?);

    let (storage, chain_info, _) =
        Genesis::init_storage_for_test(&net).expect("init storage by genesis fail.");
    let local_node = Arc::new(SyncNodeMocker::new_with_storage(
        net,
        storage.clone(),
        chain_info,
        AccountInfo::random(),
        1,
        0,
    )?);

    target_node =
        sync_block_in_async_connection(target_node, local_node.clone(), storage.clone(), 10)?;
    _ = sync_block_in_async_connection(target_node, local_node, storage, 20)?;

    Ok(())
}

#[cfg(test)]
async fn sync_block_in_block_connection_service_mock(
    mut target_node: Arc<SyncNodeMocker>,
    local_node: Arc<SyncNodeMocker>,
    registry: &ServiceRef<RegistryService>,
    block_count: u64,
) -> Result<Arc<SyncNodeMocker>> {
    Arc::get_mut(&mut target_node)
        .unwrap()
        .produce_block(block_count)?;
    loop {
        let target = target_node.sync_target();

        let storage = local_node.chain().get_storage();
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let current_block_id = startup_info.main;

        let local_net = local_node.chain_mocker.net();
        let (local_ancestor_sender, _local_ancestor_receiver) = unbounded();

        let block_chain_service = async_std::task::block_on(
            registry.service_ref::<BlockConnectorService<MockTxPoolService>>(),
        )?;

        let (sync_task, _task_handle, task_event_counter) = full_sync_task(
            current_block_id,
            target.clone(),
            false,
            local_net.time_service(),
            storage.clone(),
            block_chain_service,
            target_node.clone(),
            local_ancestor_sender,
            DummyNetworkService::default(),
            15,
            ChainNetworkID::TEST,
            None,
            None,
        )?;
        let branch = sync_task.await?;
        info!("checking branch in sync service is the same as target's branch");
        assert_eq!(branch.current_header().id(), target.target_id.id());

        let block_connector_service = registry
            .service_ref::<BlockConnectorService<MockTxPoolService>>()
            .await?
            .clone();
        let result = block_connector_service
            .send(CheckBlockConnectorHashValue {
                head_hash: target.target_id.id(),
                number: target.target_id.number(),
            })
            .await?;
        if result.is_ok() {
            break;
        }
        let reports = task_event_counter.get_reports();
        reports
            .iter()
            .for_each(|report| debug!("reports: {}", report));
    }

    Ok(target_node)
}

#[cfg(test)]
async fn sync_dag_chain(mut target_node: Arc<SyncNodeMocker>,
    local_node: Arc<SyncNodeMocker>,
    registry: &ServiceRef<RegistryService>,
) -> Result<(
    // BoxFuture<'static, Result<BlockChain, TaskError>>,
    // TaskHandle,
    // Arc<TaskEventCounterHandle>,
)> {
    Arc::get_mut(&mut target_node)
    .unwrap()
    .produce_block_and_create_dag(21)?;
    Ok(())

    // let flexidag_service = registry.service_ref::<FlexidagService>().await?;
    // let local_dag_accumulator_info = flexidag_service.send(GetDagAccumulatorInfo).await??.ok_or(anyhow!("dag accumulator is none"))?;

    // let result = sync_dag_full_task(
    //     local_dag_accumulator_info,
    //     target_accumulator_info,
    //     target_node.clone(),
    //     accumulator_store,
    //     accumulator_snapshot,
    //     local_store,
    //     local_net.time_service(),
    //     None,
    //     connector_service,
    //     network,
    //     false,
    //     dag,
    //     block_chain_service,
    //     flexidag_service,
    //     local_net.id().clone(),
    // )?;

    // Ok(result)
}

// #[cfg(test)]
// async fn sync_dag_block_from_single_chain(
//     mut target_node: Arc<SyncNodeMocker>,
//     local_node: Arc<SyncNodeMocker>,
//     registry: &ServiceRef<RegistryService>,
//     block_count: u64,
// ) -> Result<Arc<SyncNodeMocker>> {
//     use starcoin_consensus::BlockDAG;

//     Arc::get_mut(&mut target_node)
//         .unwrap()
//         .produce_block(block_count)?;
//     loop {
//         let target = target_node.sync_target();

//         let storage = local_node.chain().get_storage();
//         let startup_info = storage
//             .get_startup_info()?
//             .ok_or_else(|| format_err!("Startup info should exist."))?;
//         let current_block_id = startup_info.main;

//         let local_net = local_node.chain_mocker.net();
//         let (local_ancestor_sender, _local_ancestor_receiver) = unbounded();

//         let block_chain_service = async_std::task::block_on(
//             registry.service_ref::<BlockConnectorService<MockTxPoolService>>(),
//         )?;

//         let (sync_task, _task_handle, task_event_counter) = if local_node.chain().head_block().block.header().number()
//             > BlockDAG::dag_fork_height_with_net(local_net.id().clone()) {

//         } else {
//             full_sync_task(
//                 current_block_id,
//                 target.clone(),
//                 false,
//                 local_net.time_service(),
//                 storage.clone(),
//                 block_chain_service,
//                 target_node.clone(),
//                 local_ancestor_sender,
//                 DummyNetworkService::default(),
//                 15,
//                 ChainNetworkID::TEST,
//                 None,
//                 None,
//             )?
//         };

//         let branch = sync_task.await?;
//         info!("checking branch in sync service is the same as target's branch");
//         assert_eq!(branch.current_header().id(), target.target_id.id());

//         let block_connector_service = registry
//             .service_ref::<BlockConnectorService<MockTxPoolService>>()
//             .await?
//             .clone();
//         let result = block_connector_service
//             .send(CheckBlockConnectorHashValue {
//                 head_hash: target.target_id.id(),
//                 number: target.target_id.number(),
//             })
//             .await?;
//         if result.is_ok() {
//             break;
//         }
//         let reports = task_event_counter.get_reports();
//         reports
//             .iter()
//             .for_each(|report| debug!("reports: {}", report));
//     }

//     Ok(target_node)
// }

#[stest::test]
async fn test_sync_block_apply_failed_but_connect_success() -> Result<()> {
    let test_system = SyncTestSystem::initialize_sync_system().await?;
    let target_node = sync_block_in_block_connection_service_mock(
        test_system.target_node,
        test_system.local_node.clone(),
        &test_system.registry,
        10,
    )
    .await?;
    _ = sync_block_in_block_connection_service_mock(
        target_node,
        test_system.local_node.clone(),
        &test_system.registry,
        10,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
struct SyncTestSystem {
    pub target_node: Arc<SyncNodeMocker>,
    pub local_node: Arc<SyncNodeMocker>,
    pub registry: ServiceRef<RegistryService>,
}

#[cfg(test)]
impl SyncTestSystem {
    async fn initialize_sync_system() -> Result<SyncTestSystem> {
        let config = Arc::new(NodeConfig::random_for_test());
        let (storage, chain_info, _) = StarcoinGenesis::init_storage_for_test(config.net())
            .expect("init storage by genesis fail.");

        let target_node = Arc::new(SyncNodeMocker::new(config.net().clone(), 1, 0)?);
        let local_node = Arc::new(SyncNodeMocker::new_with_storage(
            config.net().clone(),
            storage.clone(),
            chain_info.clone(),
            AccountInfo::random(),
            1,
            0,
        )?);

        let (registry_sender, registry_receiver) = async_std::channel::unbounded();

        info!(
        "in test_sync_block_apply_failed_but_connect_success, start tokio runtime for main thread"
    );

        let _handle = timeout_join_handler::spawn(move || {
            let system = System::with_tokio_rt(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .on_thread_stop(|| debug!("main thread stopped"))
                    .thread_name("main")
                    .build()
                    .expect("failed to create tokio runtime for main")
            });
            async_std::task::block_on(async {
                let registry = RegistryService::launch();

                registry.put_shared(config.clone()).await.unwrap();
                registry.put_shared(storage.clone()).await.unwrap();
                registry.register::<FlexidagService>().await.unwrap();
                registry.put_shared(MockTxPoolService::new()).await.unwrap();

                Delay::new(Duration::from_secs(2)).await;

                registry
                    .register::<BlockConnectorService<MockTxPoolService>>()
                    .await
                    .unwrap();

                registry_sender.send(registry).await.unwrap();
            });

            system.run().unwrap();
        });

        let registry = registry_receiver.recv().await.unwrap();

        Ok(SyncTestSystem {
            target_node,
            local_node,
            registry,
        })
    }
}

// #[stest::test]
// async fn test_sync_single_chain_to_dag_chain() -> Result<()> {
//     let test_system = SyncTestSystem::initialize_sync_system().await?;
//     let target_node = sync_dag_chain(
//         test_system.target_node,
//         test_system.local_node.clone(),
//         &test_system.registry,
//         10,
//     )
//     .await?;
//     _ = sync_block_in_block_connection_service_mock(
//         target_node,
//         test_system.local_node.clone(),
//         &test_system.registry,
//         10,
//     )
//     .await?;

//     Ok(())
// }
