// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::crash_handler::setup_panic_handler;
use crate::node::Node;
use actix::prelude::*;
use anyhow::{bail, format_err, Result};
use futures::executor::block_on;
use starcoin_config::{NodeConfig, StarcoinOpt};
use starcoin_logger::prelude::*;
use starcoin_node_api::message::{NodeRequest, NodeResponse};
use starcoin_node_api::service_registry::ServiceInfo;
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

pub mod crash_handler;
pub mod node;

pub struct NodeHandle {
    runtime: Runtime,
    thread_handle: JoinHandle<Result<()>>,
    node_addr: Addr<Node>,
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
    pub fn new(thread_handle: std::thread::JoinHandle<Result<()>>, node_addr: Addr<Node>) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            thread_handle,
            node_addr,
        }
    }

    pub fn join(mut self) -> Result<()> {
        self.runtime.block_on(async {
            platform::wait_signal().await;
        });
        self.stop()
    }

    pub fn stop(self) -> Result<()> {
        self.node_addr
            .try_send(NodeRequest::StopSystem)
            .map_err(|_| format_err!("Stop message send fail."))?;
        self.thread_handle
            .join()
            .map_err(|e| format_err!("Waiting thread exist fail. {:?}", e))??;
        println!("Starcoin node shutdown success.");
        Ok(())
    }

    pub fn node_addr(&self) -> Addr<Node> {
        self.node_addr.clone()
    }

    pub fn list_service(&self) -> Result<Vec<ServiceInfo>> {
        let node_addr = self.node_addr();
        block_on(async {
            match node_addr.send(NodeRequest::ListService).await?? {
                NodeResponse::Services(services) => Ok(services),
                resp => bail!("Unexpected response: {:?}", resp),
            }
        })
    }

    pub fn stop_service(&self, service_name: String) -> Result<()> {
        let node_addr = self.node_addr();
        block_on(async {
            match node_addr
                .send(NodeRequest::StopService(service_name))
                .await??
            {
                NodeResponse::Result(r) => r,
                resp => bail!("Unexpected response: {:?}", resp),
            }
        })
    }

    pub fn start_service(&self, service_name: String) -> Result<()> {
        let node_addr = self.node_addr();
        block_on(async {
            match node_addr
                .send(NodeRequest::StartService(service_name))
                .await??
            {
                NodeResponse::Result(r) => r,
                resp => bail!("Unexpected response: {:?}", resp),
            }
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
    info!("Final data-dir is : {:?}", config.data_dir());
    if config.logger.enable_file() {
        let file_log_path = config.logger.get_log_path();
        info!("Write log to file: {:?}", file_log_path);
        logger_handle.enable_file(
            file_log_path,
            config.logger.max_file_size,
            config.logger.max_backup,
        );
    }
    if config.logger.enable_stderr {
        logger_handle.enable_stderr();
    } else {
        logger_handle.disable_stderr();
    }

    // start metric server
    if config.metrics.enable_metrics {
        starcoin_metrics::metric_server::start_server(
            config.metrics.address.clone(),
            config.metrics.port,
        );
    }

    let (start_sender, start_receiver) = oneshot::channel();
    let thread_handle = std::thread::spawn(move || {
        setup_panic_handler();
        let mut system = System::builder().stop_on_panic(true).name("main").build();
        system.block_on(async {
            match node::start(config, Some(logger_handle)).await {
                Err(e) => {
                    error!("Node start fail: {:?}.", e);
                    if start_sender.send(Err(e)).is_err() {
                        info!("Start send error.");
                    };
                }
                Ok(node_handle) => {
                    if start_sender.send(Ok(node_handle.node_addr)).is_err() {
                        info!("Start send error.");
                    }
                }
            };
        });
        system.run().map_err(|e| e.into())
    });
    let node_addr = block_on(async { start_receiver.await }).expect("Wait node start error.")?;
    Ok(NodeHandle::new(thread_handle, node_addr))
}
