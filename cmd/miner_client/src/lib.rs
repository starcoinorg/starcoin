// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod cpu_solver;
pub mod job_client;
pub mod miner;
mod solver;

use actix::prelude::*;
use anyhow::Result;
use dyn_clone::DynClone;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::stream::BoxStream;
use starcoin_config::TimeService;
use starcoin_types::system_events::MintBlockEvent;
use std::sync::Arc;

pub use starcoin_config::ConsensusStrategy;
pub use starcoin_types::U256;

pub trait JobClient: Send + Unpin + Sync + Clone {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>>;
    fn submit_seal(&self, minting_blob: Vec<u8>, nonce: u32) -> Result<()>;
    fn time_service(&self) -> Arc<dyn TimeService>;
}

pub trait Solver: Send + DynClone {
    fn solve(
        &mut self,
        strategy: ConsensusStrategy,
        minting_blob: &[u8],
        diff: U256,
        nonce_tx: UnboundedSender<(Vec<u8>, u32)>,
        stop_rx: UnboundedReceiver<bool>,
    );
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SealEvent {
    minting_blob: Vec<u8>,
    nonce: u32,
}
