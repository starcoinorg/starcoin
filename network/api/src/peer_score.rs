// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_metrics::{register, Opts, Registry, UIntGauge, UIntGaugeVec};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ScoreCounter {
    score: AtomicU64,
    count: AtomicU64,
}

impl ScoreCounter {
    pub fn new(score: u64) -> Self {
        Self {
            score: AtomicU64::new(if score == 0 { 1 } else { score }),
            count: AtomicU64::new(0),
        }
    }

    pub fn inc_by(&self, score: u64) {
        self.score.fetch_add(score, Ordering::SeqCst);
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn score(&self) -> u64 {
        self.score.load(Ordering::SeqCst)
    }

    pub fn avg(&self) -> u64 {
        self.score()
            .checked_div(self.count.load(Ordering::SeqCst))
            .unwrap_or_default()
    }
}

impl Default for ScoreCounter {
    fn default() -> Self {
        Self::new(1)
    }
}

pub trait Score<Entry>: Sync + Send {
    fn execute(&self, entry: Entry) -> u64;
}

#[derive(Clone)]
pub struct InverseScore {
    k: u64,
}

impl InverseScore {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            k: x.saturating_mul(y) as u64,
        }
    }
}

impl Score<u32> for InverseScore {
    fn execute(&self, time: u32) -> u64 {
        //if time is 0, treat as 1
        self.k.checked_div(time as u64).unwrap_or(self.k)
    }
}

//TODO rework on peer score.
//the future,and fail state do not used.
pub enum HandleState {
    Future,
    Succ,
    Fail,
}

pub struct BlockBroadcastEntry {
    new: bool,
    state: HandleState,
}

impl BlockBroadcastEntry {
    pub fn new(new: bool, state: HandleState) -> Self {
        Self { new, state }
    }
}

#[derive(Clone)]
pub struct LinearScore {
    base: u64,
}

impl LinearScore {
    pub fn new(base: u64) -> Self {
        Self { base }
    }

    pub fn linear(&self) -> u64 {
        self.base
    }

    pub fn percentage(&self, percent: usize) -> u64 {
        self.base
            .saturating_mul(percent as u64)
            .checked_div(100)
            .unwrap_or_default()
    }
}

impl Score<BlockBroadcastEntry> for LinearScore {
    fn execute(&self, entry: BlockBroadcastEntry) -> u64 {
        match entry.state {
            HandleState::Future => {
                if entry.new {
                    self.percentage(50)
                } else {
                    self.percentage(5)
                }
            }
            HandleState::Succ => {
                if entry.new {
                    self.linear()
                } else {
                    self.percentage(10)
                }
            }
            HandleState::Fail => 0u64,
        }
    }
}

#[derive(Clone)]
pub struct PeerScoreMetrics {
    pub peer_score: UIntGaugeVec,
    pub total_score: UIntGauge,
}

impl PeerScoreMetrics {
    pub fn register(registry: &Registry) -> Result<PeerScoreMetrics> {
        let peer_score = UIntGaugeVec::new(
            Opts::new("peer_score", "peer sync score".to_string()),
            &["peer"],
        )?;
        let total_score =
            UIntGauge::with_opts(Opts::new("total_score", "total peer score".to_string()))?;
        let peer_score = register(peer_score, registry)?;
        let total_score = register(total_score, registry)?;
        Ok(Self {
            peer_score,
            total_score,
        })
    }
}
