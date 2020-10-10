// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node::NodeService;
use anyhow::{format_err, Result};
use futures::executor::block_on;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{NodeConfig, StarcoinOpt};
use starcoin_logger::prelude::*;
use starcoin_network::NetworkAsyncService;
use starcoin_node_api::message::NodeRequest;
use starcoin_node_api::node_service::NodeAsyncService;
use starcoin_rpc_server::service::RpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceInfo, ServiceRef};
use starcoin_types::block::BlockDetail;
use starcoin_types::system_events::{GenerateBlockEvent, NewHeadBlock};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

pub mod crash_handler;
pub mod node;
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
    use std::error::Error;

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

    pub fn bus(&self) -> Result<ServiceRef<BusService>> {
        block_on(async { self.registry.service_ref::<BusService>().await })
    }

    pub fn network(&self) -> NetworkAsyncService {
        self.registry
            .get_shared_sync::<NetworkAsyncService>()
            .expect("NetworkAsyncService must exist.")
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

    /// Just for test
    pub fn generate_block(&self) -> Result<BlockDetail> {
        let registry = &self.registry;
        block_on(async move {
            let bus = registry.service_ref::<BusService>().await?;
            let receiver = bus.oneshot::<NewHeadBlock>().await?;
            bus.broadcast(GenerateBlockEvent::new(false))?;
            let new_head_block = receiver.await?;
            Ok(new_head_block.0.as_ref().clone())
        })
    }
}

pub fn run_node_by_opt(opt: &StarcoinOpt) -> Result<(Option<NodeHandle>, Arc<NodeConfig>)> {
    let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
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
pub fn run_node(config: Arc<NodeConfig>) -> Result<NodeHandle> {
    let logger_handle = starcoin_logger::init();
    NodeService::launch(config, logger_handle)
}
