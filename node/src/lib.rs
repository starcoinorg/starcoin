// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::genesis_parameter_resolve::RpcFutureBlockParameterResolver;
use crate::node::NodeService;
use anyhow::{bail, format_err, Result};
use futures::executor::block_on;
use futures_timer::Delay;
use starcoin_chain_service::{ChainAsyncService, ChainReaderService};
use starcoin_config::{BaseConfig, NodeConfig, StarcoinOpt};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_network::NetworkServiceRef;
use starcoin_node_api::errors::NodeStartError;
use starcoin_node_api::message::NodeRequest;
use starcoin_node_api::node_service::NodeAsyncService;
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceInfo, ServiceRef};
use starcoin_storage::Storage;
use starcoin_sync::sync::SyncService;
use starcoin_txpool::TxPoolService;
use starcoin_types::block::Block;
use starcoin_types::system_events::{GenerateBlockEvent, NewHeadBlock};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

pub mod crash_handler;
mod genesis_parameter_resolve;
mod metrics;
pub mod network_service_factory;
pub mod node;
pub mod peer_message_handler;
pub mod rpc_service_factory;

pub struct NodeHandle {
    runtime: Runtime,
    join_handle: timeout_join_handler::TimeoutJoinHandle<Result<()>>,
    node_service: ServiceRef<NodeService>,
    registry: ServiceRef<RegistryService>,
}

#[cfg(unix)]
mod platform {
    use futures::{future::FutureExt, pin_mut, select};
    use tokio::signal::unix::{signal, SignalKind};

    pub async fn wait_signal() {
        println!("Waiting SIGINT or SIGTERM ...");
        let mut sigint = signal(SignalKind::interrupt()).expect("register signal error");
        let sigint_fut = sigint.recv().fuse();
        let mut sigterm = signal(SignalKind::terminate()).expect("register signal error");
        let sigterm_fut = sigterm.recv().fuse();
        pin_mut!(sigint_fut, sigterm_fut);
        select! {
            _ = sigterm_fut => {
                println!("received SIGTERM");
            }
             _ = sigint_fut => {
                 println!("received SIGINT");
             }
        }
    }
}

#[cfg(not(unix))]
mod platform {
    pub async fn wait_signal() {
        println!("Waiting Ctrl-C ...");
        tokio::signal::ctrl_c().await.unwrap();
        println!("Ctrl-C received, shutting down");
    }
}

impl NodeHandle {
    pub fn new(
        join_handle: timeout_join_handler::TimeoutJoinHandle<Result<()>>,
        node_service: ServiceRef<NodeService>,
        registry: ServiceRef<RegistryService>,
    ) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            join_handle,
            node_service,
            registry,
        }
    }

    pub fn join(mut self) -> Result<()> {
        self.runtime.block_on(async {
            //TODO also wait actor system stop signal, support stop system by command.
            platform::wait_signal().await;
        });
        self.stop()
    }

    pub fn stop(self) -> Result<()> {
        self.node_service
            .try_send(NodeRequest::ShutdownSystem)
            .map_err(|_| format_err!("Stop message send fail."))?;
        self.join_handle
            .join(Duration::from_millis(5000))
            .map_err(|_| format_err!("Waiting thread exist timeout."))??;
        println!("Starcoin node shutdown success.");
        Ok(())
    }

    pub fn registry(&self) -> &ServiceRef<RegistryService> {
        &self.registry
    }

    pub fn node_service(&self) -> &ServiceRef<NodeService> {
        &self.node_service
    }

    pub fn config(&self) -> Arc<NodeConfig> {
        self.registry
            .get_shared_sync::<Arc<NodeConfig>>()
            .expect("NodeConfig must exist.")
    }

    pub fn storage(&self) -> Arc<Storage> {
        self.registry
            .get_shared_sync::<Arc<Storage>>()
            .expect("Storage must exist.")
    }

    pub fn genesis(&self) -> Genesis {
        self.registry
            .get_shared_sync::<Genesis>()
            .expect("Genesis must exist.")
    }

    pub fn bus(&self) -> Result<ServiceRef<BusService>> {
        block_on(async { self.registry.service_ref::<BusService>().await })
    }

    pub fn network(&self) -> NetworkServiceRef {
        self.registry
            .get_shared_sync::<NetworkServiceRef>()
            .expect("NetworkAsyncService must exist.")
    }

    pub fn sync_service(&self) -> Result<ServiceRef<SyncService>> {
        block_on(async { self.registry.service_ref::<SyncService>().await })
    }

    pub fn rpc_service(&self) -> Result<ServiceRef<RpcService>> {
        block_on(async { self.registry.service_ref::<RpcService>().await })
    }

    pub fn chain_service(&self) -> Result<ServiceRef<ChainReaderService>> {
        block_on(async { self.registry.service_ref::<ChainReaderService>().await })
    }

    pub fn list_service(&self) -> Result<Vec<ServiceInfo>> {
        let node_addr = self.node_service();
        block_on(async { node_addr.list_service().await })
    }

    pub fn stop_service(&self, service_name: String) -> Result<()> {
        let node_addr = self.node_service();
        block_on(async { node_addr.stop_service(service_name).await })
    }

    pub fn start_service(&self, service_name: String) -> Result<()> {
        let node_addr = self.node_service();
        block_on(async { node_addr.start_service(service_name).await })
    }

    pub fn txpool(&self) -> TxPoolService {
        self.registry
            .get_shared_sync::<TxPoolService>()
            .expect("TxPoolService must exist.")
    }

    /// Just for test
    pub fn generate_block(&self) -> Result<Block> {
        let registry = &self.registry;
        block_on(async move {
            let bus = registry.service_ref::<BusService>().await?;
            let chain_service = registry.service_ref::<ChainReaderService>().await?;
            let head = chain_service.main_head_block().await?;
            debug!("generate_block: current head block: {:?}", head.header);
            let receiver = bus.oneshot::<NewHeadBlock>().await?;
            bus.broadcast(GenerateBlockEvent::new(true))?;
            let block = if let Ok(Ok(event)) =
                async_std::future::timeout(Duration::from_secs(5), receiver).await
            {
                //wait for new block event to been processed.
                Delay::new(Duration::from_millis(100)).await;
                event.0.block().clone()
            } else {
                let latest_head = chain_service.main_head_block().await?;
                debug!(
                    "generate_block: head before generate:{:?}, head after generate:{:?}",
                    head.header(),
                    latest_head.header
                );
                if latest_head.header().number() > head.header().number() {
                    latest_head
                } else {
                    bail!("Wait timeout for generate_block")
                }
            };
            Ok(block)
        })
    }
}

pub fn run_node_by_opt(
    opt: &StarcoinOpt,
) -> Result<(Option<NodeHandle>, Arc<NodeConfig>), NodeStartError> {
    //check genesis config is ready
    let mut base_config =
        BaseConfig::load_with_opt(opt).map_err(NodeStartError::LoadConfigError)?;
    if !base_config.net().is_ready() {
        let future_block_resolve =
            RpcFutureBlockParameterResolver::new(base_config.net().id().clone());
        base_config
            .resolve(&future_block_resolve)
            .map_err(NodeStartError::LoadConfigError)?;
    }
    let config = Arc::new(
        base_config
            .into_node_config(opt)
            .map_err(NodeStartError::LoadConfigError)?,
    );
    let ipc_file = config.rpc.get_ipc_file();
    let node_handle = if !ipc_file.exists() {
        let node_handle = run_node(config.clone())?;
        Some(node_handle)
    } else {
        //TODO check ipc file is available.
        info!("Node has started at {:?}", ipc_file);
        None
    };
    Ok((node_handle, config))
}

/// Run node in a new Thread, and return a NodeHandle.
pub fn run_node(config: Arc<NodeConfig>) -> Result<NodeHandle, NodeStartError> {
    crash_handler::setup_panic_handler();
    let logger_handle = starcoin_logger::init();
    NodeService::launch(config, logger_handle)
}
