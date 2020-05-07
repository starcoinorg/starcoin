// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::{format_err, Result};
use futures::executor::block_on;
use starcoin_config::NodeConfig;
use starcoin_consensus::{
    argon::{ArgonConsensus, ArgonConsensusHeader},
    dummy::{DummyConsensus, DummyHeader},
};
use starcoin_logger::prelude::*;
use starcoin_traits::{Consensus, ConsensusHeader};
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

mod actor;
pub mod message;
mod node;

pub use actor::{NodeActor, NodeRef};

pub struct NodeHandle {
    runtime: Runtime,
    thread_handle: JoinHandle<()>,
    stop_sender: oneshot::Sender<()>,
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
        thread_handle: std::thread::JoinHandle<()>,
        stop_sender: oneshot::Sender<()>,
    ) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            thread_handle,
            stop_sender,
        }
    }

    pub fn join(mut self) -> Result<()> {
        self.runtime.block_on(async {
            platform::wait_signal().await;
        });
        self.stop()
    }

    pub fn stop(self) -> Result<()> {
        self.stop_sender
            .send(())
            .map_err(|_| format_err!("Stop message send fail."))?;
        self.thread_handle
            .join()
            .map_err(|e| format_err!("Waiting thread exist fail. {:?}", e))?;
        println!("Starcoin node shutdown success.");
        Ok(())
    }
}

pub fn run_dev_node(config: Arc<NodeConfig>) -> NodeHandle {
    run_node::<DummyConsensus, DummyHeader>(config)
}

pub fn run_normal_node(config: Arc<NodeConfig>) -> NodeHandle {
    run_node::<ArgonConsensus, ArgonConsensusHeader>(config)
}

/// Run node in a new Thread, and return a NodeHandle.
pub fn run_node<C, H>(config: Arc<NodeConfig>) -> NodeHandle
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static,
{
    let logger_handle = starcoin_logger::init();
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
    starcoin_metrics::metric_server::start_server(
        config.metrics.address.clone(),
        config.metrics.metrics_server_port,
    );

    let (start_sender, start_receiver) = oneshot::channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    let thread_handle = std::thread::spawn(move || {
        //TODO actix and tokio use same runtime, and config thread pool.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();
        let mut system = System::builder().stop_on_panic(true).name("main").build();
        system.block_on(async {
            //let node_actor = NodeActor::<C, H>::new(config, handle);
            //let _node_ref = node_actor.start();
            //TODO fix me, this just a work around method.
            let _handle = match node::start::<C, H>(config, logger_handle, handle).await {
                Err(e) => {
                    error!("Node start fail: {}, exist.", e);
                    System::current().stop();
                    return;
                }
                Ok(handle) => handle,
            };
            if start_sender.send(()).is_err() {
                info!("Start send error.");
            }
            if stop_receiver.await.is_err() {
                info!("Stop receiver await error.");
            }
            info!("Receive stop signal, try to stop system.");
            System::current().stop();
        });
    });
    if block_on(async { start_receiver.await }).is_err() {
        info!("Wait start receiver error.");
    }
    NodeHandle::new(thread_handle, stop_sender)
}
