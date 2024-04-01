// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::integer_arithmetic)]
use crate::tasks::mock::MockBlockIdFetcher;
use crate::tasks::{
    AccumulatorCollector, AncestorCollector, BlockAccumulatorSyncTask,
    BlockCollector, FindAncestorTask,
};
use anyhow::{format_err, Result};
use anyhow::{Context, Ok};
use futures::channel::mpsc::unbounded;
use network_api::PeerId;
use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_chain_api::ChainReader;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::BlockBody;
use starcoin_storage::BlockStore;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{
    Block, BlockHeaderBuilder, BlockIdAndNumber, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH,
};
use std::sync::Arc;
use stream_task::{DefaultCustomErrorHandle, Generator, TaskEventCounterHandle, TaskGenerator};
use test_helper::DummyNetworkService;

use super::mock::MockBlockFetcher;
use super::test_tools::{
    block_sync_task_test, block_sync_with_local, full_sync_cancel, full_sync_continue, full_sync_fork, full_sync_fork_from_genesis, full_sync_new_node, net_rpc_err, sync_block_in_async_connection, sync_invalid_target, sync_target,
};

#[stest::test(timeout = 120)]
pub async fn test_full_sync_new_node() -> Result<()> {
    full_sync_new_node(10, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test]
pub async fn test_sync_invalid_target() -> Result<()> {
    sync_invalid_target(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test]
pub async fn test_failed_block() -> Result<()> {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Halley);
    let (storage, chain_info, _, dag) =
        Genesis::init_storage_for_test(&net)?;

    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        None,
        dag,
    )?;
    let fetcher = MockBlockFetcher::new();
    let (sender, _) = unbounded();
    let chain_status = chain.status();
    let target = SyncTarget {
        target_id: BlockIdAndNumber::new(chain_status.head.id(), chain_status.head.number()),
        block_info: chain_status.info,
        peers: vec![PeerId::random()],
    };
    let mut block_collector = BlockCollector::new_with_handle(
        chain_info.status().info.clone(),
        target,
        chain,
        sender,
        DummyNetworkService::default(),
        true,
        storage.clone(),
        Arc::new(fetcher),
    );
    let header = BlockHeaderBuilder::random().with_number(1).build();
    let body = BlockBody::new(Vec::new(), None);
    let failed_block = Block::new(header, body);
    let failed_block_id = failed_block.id();
    if block_collector.apply_block_for_test(failed_block).is_err() {
        assert!(storage.get_failed_block_by_id(failed_block_id)?.is_some());
        Ok(())
    } else {
        Err(format_err!("test FailedBlock fail."))
    }
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_fork() -> Result<()> {
    full_sync_fork(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_fork_from_genesis() -> Result<()> {
    full_sync_fork_from_genesis(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test(timeout = 120)]
pub async fn test_full_sync_continue() -> Result<()> {
    full_sync_continue(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test]
pub async fn test_full_sync_cancel() -> Result<()> {
    full_sync_cancel(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
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
        300,
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
        300,
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
        300,
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
        300,
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

#[stest::test]
async fn test_block_sync() -> Result<()> {
    block_sync_task_test(100, 0, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test]
async fn test_block_sync_one_block() -> Result<()> {
    block_sync_task_test(2, 0, TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test]
async fn test_block_sync_with_local() -> Result<()> {
    block_sync_with_local(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

#[stest::test(timeout = 120)]
async fn test_net_rpc_err() -> Result<()> {
    net_rpc_err(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
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
    sync_target(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await;
}

#[stest::test]
async fn test_sync_block_in_async_connection() -> Result<()> {
    sync_block_in_async_connection(TEST_FLEXIDAG_FORK_HEIGHT_NEVER_REACH).await
}

// #[cfg(test)]
// async fn sync_dag_chain(
//     mut target_node: Arc<SyncNodeMocker>,
//     local_node: Arc<SyncNodeMocker>,
//     registry: &ServiceRef<RegistryService>,
// ) -> Result<()> {
//     Arc::get_mut(&mut target_node)
//         .unwrap()
//         .produce_block_and_create_dag(21)?;
//     Ok(())

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
// }

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
