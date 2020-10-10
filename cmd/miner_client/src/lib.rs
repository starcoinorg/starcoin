// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod job_client;
pub mod miner;
mod worker;

use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use futures::stream::BoxStream;
use rand::Rng;
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::MintBlockEvent;
use std::ops::Range;

pub trait JobClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>>;
    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) -> Result<()>;
    fn consensus(&self) -> Result<ConsensusStrategy>;
}

fn partition_nonce(id: u64, total: u64) -> Range<u64> {
    let span = u64::max_value() / total;
    let start = span * id;
    let end = match id {
        x if x < total - 1 => start + span,
        x if x == total - 1 => u64::max_value(),
        _ => unreachable!(),
    };
    Range { start, end }
}

fn nonce_generator(range: Range<u64>) -> impl FnMut() -> u64 {
    let mut rng = rand::thread_rng();
    let Range { start, end } = range;
    move || rng.gen_range(start, end)
}

#[derive(Clone, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct SealEvent {
    pow_hash: HashValue,
    nonce: u64,
}
