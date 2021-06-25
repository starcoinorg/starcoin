// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
mod cpu_solver;
pub mod job_bus_client;
pub mod job_client;
pub mod miner;
mod solver;
pub mod stratum_client;
pub mod stratum_client_service;

use anyhow::Result;
use futures::stream::BoxStream;
use starcoin_config::TimeService;
use starcoin_types::system_events::SealEvent;
pub use starcoin_types::{
    block::BlockHeaderExtra,
    genesis_config::ConsensusStrategy,
    system_events::{MintBlockEvent, MintEventExtra},
    U256,
};
use std::sync::Arc;

pub trait JobClient: Send + Unpin + Sync + Clone {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>>;

    fn submit_seal(&self, seal: SealEvent) -> Result<()>;
    fn time_service(&self) -> Arc<dyn TimeService>;
}
