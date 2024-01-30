// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::integer_arithmetic)]
use crate::block_connector::BlockConnectorService;
use crate::tasks::{full_sync_task, BlockSyncTask};
use crate::tasks::mock::{MockLocalBlockStore, SyncNodeMocker};
use anyhow::{format_err, Result};
use futures::channel::mpsc::unbounded;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures_timer::Delay;
use network_api::PeerId;
use pin_utils::core_reexport::time::Duration;
use starcoin_account_api::AccountInfo;
use starcoin_accumulator::tree_store::mock::MockAccumulatorStore;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain_api::ChainReader;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, NodeConfig, RocksdbConfig};
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::prelude::FlexiDagStorageConfig;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
// use starcoin_txpool_mock_service::MockTxPoolService;
#[cfg(test)]
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::{Block, BlockHeaderBuilder, BlockIdAndNumber, BlockNumber, TEST_FLEXIDAG_FORK_HEIGHT_FOR_DAG};
use starcoin_types::U256;
use stream_task::{DefaultCustomErrorHandle, Generator, TaskEventCounterHandle, TaskGenerator};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use stest::actix_export::System;
use test_helper::DummyNetworkService;

use super::mock::MockBlockFetcher;
use super::BlockFetcher;

#[cfg(test)]
pub struct SyncTestSystem {
    pub target_node: SyncNodeMocker,
    pub local_node: SyncNodeMocker,
    pub registry: ServiceRef<RegistryService>,
}

#[cfg(test)]
impl SyncTestSystem {
    pub async fn initialize_sync_system() -> Result<SyncTestSystem> {
        let config = Arc::new(NodeConfig::random_for_test());

        // let (storage, chain_info, _, _) = StarcoinGenesis::init_storage_for_test(config.net())
        //     .expect("init storage by genesis fail.");

        let temp_path = PathBuf::from(starcoin_config::temp_dir().as_ref());
        let storage_path = temp_path.join(Path::new("local/storage"));
        let dag_path = temp_path.join(Path::new("local/dag"));
        fs::create_dir_all(storage_path.clone())?;
        fs::create_dir_all(dag_path.clone())?;
        let storage = Arc::new(
            Storage::new(StorageInstance::new_db_instance(
                DBStorage::new(storage_path.as_path(), RocksdbConfig::default(), None).unwrap(),
            ))
            .unwrap(),
        );
        let genesis = Genesis::load_or_build(config.net())?;
        // init dag
        let dag_storage = starcoin_dag::consensusdb::prelude::FlexiDagStorage::create_from_path(
            dag_path.as_path(),
            FlexiDagStorageConfig::new(),
        )
        .expect("init dag storage fail.");
        let dag = starcoin_dag::blockdag::BlockDAG::new(8, dag_storage); // local dag

        let chain_info =
            genesis.execute_genesis_block(config.net(), storage.clone(), dag.clone())?;

        let target_node = SyncNodeMocker::new(config.net().clone(), 300, 0)?;
        let local_node = SyncNodeMocker::new_with_storage(
            config.net().clone(),
            storage.clone(),
            chain_info.clone(),
            AccountInfo::random(),
            300,
            0,
            dag.clone(),
        )?;

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
                registry
                    .put_shared(dag)
                    .await
                    .expect("failed to put dag in registry");
                registry.put_shared(MockTxPoolService::new()).await.unwrap();

                Delay::new(Duration::from_secs(2)).await;

                registry.register::<ChainReaderService>().await.unwrap();
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

#[cfg(test)]
pub async fn full_sync_new_node(fork_number: BlockNumber) -> Result<()> {
    let count_blocks = 10;
    assert!(fork_number < count_blocks, "The fork number should be smaller than the count block");
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.set_dag_fork_number(fork_number)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 300, 0)?;
    node2.set_dag_fork_number(fork_number)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();

    let storage = node2.chain().get_storage();
    let dag = node2.chain().dag();
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
        None,
        None,
        dag.clone(),
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
        None,
        None,
        dag,
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

#[cfg(test)]
pub async fn sync_invalid_target(fork_number: BlockNumber) -> Result<()> {
    use stream_task::TaskError;

    use crate::verified_rpc_client::RpcVerifyError;

    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.set_dag_fork_number(fork_number)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 300, 0)?;
    node2.set_dag_fork_number(fork_number)?;
    let dag = node2.chain().dag();
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
        None,
        None,
        dag,
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

#[cfg(test)]
pub async fn full_sync_fork(fork_number: BlockNumber) -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.set_dag_fork_number(fork_number)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 300, 0)?;
    node2.set_dag_fork_number(fork_number)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();
    let dag = node2.chain().dag();
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
        None,
        None,
        dag.clone(),
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
        None,
        None,
        dag,
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

// #[cfg(test)]
// pub async fn generate_red_dag_block() -> Result<Block> {
//     let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
//     let mut node = SyncNodeMocker::new(net, 300, 0)?;
//     node.produce_block(10)?;
//     let block = node.produce_block(1)?;
//     Ok(block)
// }

pub async fn full_sync_fork_from_genesis(fork_number: BlockNumber) -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.set_dag_fork_number(fork_number)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    //fork from genesis
    let mut node2 = SyncNodeMocker::new(net2.clone(), 300, 0)?;
    node2.set_dag_fork_number(fork_number)?;
    node2.produce_block(5)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();
    let dag = node2.chain().dag();
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
        None,
        None,
        dag,
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

pub async fn full_sync_continue(fork_number: BlockNumber) -> Result<()> {
    // let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let test_system = SyncTestSystem::initialize_sync_system().await?;
    let mut node1 = test_system.target_node; // SyncNodeMocker::new(net1, 10, 50)?;
    node1.set_dag_fork_number(fork_number)?;
    let dag = node1.chain().dag();
    node1.produce_block(10)?;
    let arc_node1 = Arc::new(node1);
    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    //fork from genesis
    let mut node2 = test_system.local_node; // SyncNodeMocker::new(net2.clone(), 1, 50)?;
    node2.set_dag_fork_number(fork_number)?;
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
        None,
        None,
        dag.clone(),
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
        None,
        None,
        dag,
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

pub async fn full_sync_cancel(fork_number: BlockNumber) -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.set_dag_fork_number(fork_number)?;
    node1.produce_block(10)?;

    let arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::Test);

    let node2 = SyncNodeMocker::new(net2.clone(), 10, 50)?;
    node2.set_dag_fork_number(fork_number)?;

    let target = arc_node1.sync_target();

    let current_block_header = node2.chain().current_header();
    let dag = node2.chain().dag();
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
        None,
        None,
        dag,
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



pub fn build_block_fetcher(total_blocks: u64, fork_number: BlockNumber) -> (MockBlockFetcher, MerkleAccumulator) {
    let fetcher = MockBlockFetcher::new();

    let store = Arc::new(MockAccumulatorStore::new());
    let accumulator = MerkleAccumulator::new_empty(store);
    for i in 0..total_blocks {
        let header = if i > fork_number {
            BlockHeaderBuilder::random_for_dag().with_number(i).build()
        } else {
            BlockHeaderBuilder::random().with_number(i).build()
        };
        let block = Block::new(header, vec![]);
        accumulator.append(&[block.id()]).unwrap();
        fetcher.put(block);
    }
    accumulator.flush().unwrap();
    (fetcher, accumulator)
}

pub async fn block_sync_task_test(total_blocks: u64, ancestor_number: u64, fork_number: BlockNumber) -> Result<()> {
    assert!(
        total_blocks > ancestor_number,
        "total blocks should > ancestor number"
    );
    let (fetcher, accumulator) = build_block_fetcher(total_blocks, fork_number);
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
        300,
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