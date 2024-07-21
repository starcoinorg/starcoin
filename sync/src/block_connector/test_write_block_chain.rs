// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::arithmetic_side_effects)]

use crate::block_connector::WriteBlockChainService;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::Store;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::startup_info::StartupInfo;
use std::sync::Arc;

use super::test_write_dag_block_chain::new_dag_block;

pub async fn create_writeable_dag_block_chain() -> (
    WriteBlockChainService<MockTxPoolService>,
    Arc<NodeConfig>,
    Arc<dyn Store>,
) {
    let node_config = NodeConfig::random_for_dag_test();
    let node_config = Arc::new(node_config);

    let (storage, chain_info, _, dag) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let txpool_service = MockTxPoolService::new();
    (
        WriteBlockChainService::new_with_dag_fork_number(
            node_config.clone(),
            StartupInfo::new(chain_info.head().id()),
            storage.clone(),
            txpool_service,
            bus,
            None,
            dag,
        )
        .unwrap(),
        node_config,
        storage,
    )
}

pub async fn create_writeable_block_chain() -> (
    WriteBlockChainService<MockTxPoolService>,
    Arc<NodeConfig>,
    Arc<dyn Store>,
) {
    let node_config = NodeConfig::random_for_dag_test();
    let node_config = Arc::new(node_config);

    let (storage, chain_info, _, dag) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let registry = RegistryService::launch();
    let bus = registry.service_ref::<BusService>().await.unwrap();
    let txpool_service = MockTxPoolService::new();
    (
        WriteBlockChainService::new(
            node_config.clone(),
            StartupInfo::new(chain_info.head().id()),
            storage.clone(),
            txpool_service,
            bus,
            None,
            dag,
        )
        .unwrap(),
        node_config,
        storage,
    )
}
