// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::{format_err, Result};
use futures::executor::block_on;
use starcoin_config::NodeConfig;
use starcoin_consensus::{Consensus, ConsensusHeader};
use starcoin_logger::prelude::*;
use std::sync::Arc;
use tokio::sync::oneshot;

mod actor;
pub mod message;
mod node;

pub use actor::{NodeActor, NodeRef};
use starcoin_consensus::argon_consensus::{ArgonConsensus, ArgonConsensusHeader};
use starcoin_consensus::dummy::{DummyConsensus, DummyHeader};
use std::thread::JoinHandle;

pub struct NodeHandle {
    thread_handle: JoinHandle<()>,
    stop_sender: oneshot::Sender<()>,
}

impl NodeHandle {
    pub fn new(
        thread_handle: std::thread::JoinHandle<()>,
        stop_sender: oneshot::Sender<()>,
    ) -> Self {
        Self {
            thread_handle,
            stop_sender,
        }
    }

    pub fn join(self) -> Result<()> {
        block_on(async {
            tokio::signal::ctrl_c().await.unwrap();
            println!("Ctrl-C received, shutting down");
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
    let (stop_sender, stop_receiver) = oneshot::channel();
    let thread_handle = std::thread::spawn(move || {
        //TODO actix and tokio use same runtime, and config thread pool.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();
        let mut system = System::new("main");
        system.block_on(async {
            //let node_actor = NodeActor::<C, H>::new(config, handle);
            //let _node_ref = node_actor.start();
            //TODO fix me, this just a work around method.
            let _handle = match node::start::<C, H>(config, handle).await {
                Err(e) => {
                    error!("Node start fail: {}, exist.", e);
                    System::current().stop();
                    return;
                }
                Ok(handle) => handle,
            };

            stop_receiver.await.unwrap();
            info!("Receive stop signal, try to stop system.");
            System::current().stop();
        });
    });
    NodeHandle::new(thread_handle, stop_sender)
}
