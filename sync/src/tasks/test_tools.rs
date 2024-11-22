// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::arithmetic_side_effects)]
use crate::block_connector::BlockConnectorService;
use crate::tasks::full_sync_task;
use crate::tasks::mock::SyncNodeMocker;
use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures_timer::Delay;
use pin_utils::core_reexport::time::Duration;
use starcoin_account_api::AccountInfo;
use starcoin_chain_api::ChainReader;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, NodeConfig, RocksdbConfig};
use starcoin_dag::blockdag::DEFAULT_GHOSTDAG_K;
use starcoin_dag::consensusdb::prelude::FlexiDagStorageConfig;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
#[cfg(test)]
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::blockhash::KType;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use stest::actix_export::System;
use test_helper::DummyNetworkService;

#[cfg(test)]
pub struct SyncTestSystem {
    pub target_node: SyncNodeMocker,
    pub local_node: SyncNodeMocker,
    pub registry: ServiceRef<RegistryService>,
}

#[cfg(test)]
impl SyncTestSystem {
    pub async fn initialize_sync_system() -> Result<Self> {
        Self::initialize_sync_system_with_k(DEFAULT_GHOSTDAG_K).await
    }
    pub async fn initialize_sync_system_with_k(k: KType) -> Result<Self> {
        let config = Arc::new(NodeConfig::random_for_dag_test());

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
        let dag = starcoin_dag::blockdag::BlockDAG::new(k, dag_storage); // local dag

        let chain_info =
            genesis.execute_genesis_block(config.net(), storage.clone(), dag.clone())?;

        let target_node = SyncNodeMocker::new_with_k(config.net().clone(), 300, 0, k)?;
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

        Ok(Self {
            target_node,
            local_node,
            registry,
        })
    }
}

#[cfg(test)]
pub async fn full_sync_new_node() -> Result<()> {
    let net1 = ChainNetwork::new_builtin(BuiltinNetworkID::DagTest);
    let mut node1 = SyncNodeMocker::new(net1, 300, 0)?;
    node1.produce_block(10)?;

    let mut arc_node1 = Arc::new(node1);

    let net2 = ChainNetwork::new_builtin(BuiltinNetworkID::DagTest);

    let node2 = SyncNodeMocker::new(net2.clone(), 300, 0)?;

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
        node2.sync_dag_store.clone(),
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
        node2.sync_dag_store.clone(),
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
