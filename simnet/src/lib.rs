// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaCha8Rng,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub mod scene;

// Configuration structs
#[derive(Debug, Clone)]
pub struct DAGGenCfg {
    pub total_time: u64,
    pub block_interval: u64,
    pub miners: Vec<Miner>,
}

#[derive(Debug, Clone)]
pub struct Miner {
    pub name: &'static str,
    pub hash_power_ratio: f64,
    pub network_delay: u64,
    pub is_attacker: bool,
    pub hide_blocks_plan: Vec<HideBlocksBurst>,
}

#[derive(Debug, Clone)]
pub struct HideBlocksBurst {
    pub start_time_sec: u64,
    pub block_count: usize,
    pub release_time_sec: u64,
    pub release_interval_ms: u32,
}

// Block stream generation
#[derive(Clone)]
struct ProduceEvent {
    time: u64,
    miner_id: usize,
}

impl PartialEq for ProduceEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for ProduceEvent {}
impl PartialOrd for ProduceEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.time.partial_cmp(&self.time)
    }
}
impl Ord for ProduceEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time.cmp(&self.time)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct BlockEvent {
    /// Time when the node first sees the block (`header_time + network_delay`).
    pub arrival_time: u64,
    /// Miner-local production time; planning uses this chronological order to build templates.
    pub header_time: u64,
    pub miner_id: usize,
    pub network_delay: u64,
}

pub struct BlkStream {
    rng: ChaCha8Rng,
    queue: BinaryHeap<ProduceEvent>,
    block_intervals: Vec<u64>,
    horizon: u64,
    miners: Vec<Miner>,
    hide_plans: Vec<Vec<HidePlanRuntime>>,
}

impl BlkStream {
    pub fn from_seed(cfg: DAGGenCfg, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let total: f64 = cfg.miners.iter().map(|m| m.hash_power_ratio).sum();
        assert!(total > 0.0, "total hash power must be positive");

        let block_intervals: Vec<u64> = cfg
            .miners
            .iter()
            .map(|m| {
                if m.hash_power_ratio > 0.0 {
                    (cfg.block_interval as f64 * total / m.hash_power_ratio) as u64
                } else {
                    u64::MAX
                }
            })
            .collect();

        let mut queue = BinaryHeap::new();
        for (i, m) in cfg.miners.iter().enumerate() {
            if m.hash_power_ratio > 0.0 {
                queue.push(ProduceEvent {
                    time: exp_sample(&mut rng, block_intervals[i]),
                    miner_id: i,
                });
            }
        }

        Self {
            rng,
            queue,
            block_intervals,
            horizon: cfg.total_time,
            miners: cfg.miners.clone(),
            hide_plans: cfg
                .miners
                .iter()
                .map(|miner| {
                    miner
                        .hide_blocks_plan
                        .iter()
                        .cloned()
                        .map(HidePlanRuntime::new)
                        .collect()
                })
                .collect(),
        }
    }

    pub fn run(&mut self) -> Vec<BlockEvent> {
        let mut events = Vec::new();

        while let Some(produce) = self.queue.pop() {
            if produce.time > self.horizon {
                break;
            }

            self.release_due(produce.time, &mut events);

            let network_delay = self.miners[produce.miner_id].network_delay;
            if !self.stage_if_hidden(produce.miner_id, produce.time, network_delay) {
                events.push(BlockEvent {
                    arrival_time: produce.time + network_delay,
                    header_time: produce.time,
                    miner_id: produce.miner_id,
                    network_delay,
                });
            }

            let next =
                produce.time + exp_sample(&mut self.rng, self.block_intervals[produce.miner_id]);
            self.queue.push(ProduceEvent {
                time: next,
                miner_id: produce.miner_id,
            });
        }

        self.release_due(u64::MAX, &mut events);

        events.sort_by_key(|e| (e.arrival_time, e.miner_id));
        events
    }

    fn stage_if_hidden(&mut self, miner_id: usize, header_time: u64, network_delay: u64) -> bool {
        if let Some(plans) = self.hide_plans.get_mut(miner_id) {
            for plan in plans.iter_mut() {
                if plan.stage(header_time, network_delay) {
                    return true;
                }
            }
        }
        false
    }

    fn release_due(&mut self, now: u64, out: &mut Vec<BlockEvent>) {
        for (miner_id, plans) in self.hide_plans.iter_mut().enumerate() {
            for plan in plans.iter_mut() {
                plan.release_due(now, miner_id, out);
            }
        }
    }
}

fn exp_sample<R: RngCore + ?Sized>(rng: &mut R, mean_interval: u64) -> u64 {
    let u = next_unit_interval(rng);
    (-u.ln() * mean_interval as f64) as u64
}

fn next_unit_interval<R: RngCore + ?Sized>(rng: &mut R) -> f64 {
    // Map u64 samples to (0, 1]; clamp to avoid ln(0) during exponential sampling.
    const SCALE: f64 = 1.0 / (u64::MAX as f64 + 1.0);
    (rng.next_u64() as f64 * SCALE).clamp(f64::EPSILON, 1.0)
}

#[derive(Clone)]
struct HidePlanRuntime {
    cfg: HideBlocksBurst,
    staged: Vec<HiddenEvent>,
    released: bool,
}

impl HidePlanRuntime {
    fn new(cfg: HideBlocksBurst) -> Self {
        Self {
            cfg,
            staged: Vec::new(),
            released: false,
        }
    }

    fn start_ms(&self) -> u64 {
        self.cfg.start_time_sec * 1_000
    }

    fn release_ms(&self) -> u64 {
        self.cfg.release_time_sec * 1_000
    }

    fn stage(&mut self, header_time: u64, network_delay: u64) -> bool {
        if self.released || header_time < self.start_ms() {
            return false;
        }
        if self.staged.len() >= self.cfg.block_count {
            return false;
        }
        self.staged.push(HiddenEvent {
            header_time,
            network_delay,
        });
        true
    }

    fn release_due(&mut self, now: u64, miner_id: usize, out: &mut Vec<BlockEvent>) {
        if self.released || now < self.release_ms() {
            return;
        }

        let base = self.release_ms();
        let interval = self.cfg.release_interval_ms as u64;
        for (idx, hidden) in self.staged.drain(..).enumerate() {
            let scheduled = base.max(hidden.header_time) + idx as u64 * interval;
            out.push(hidden.into_event(miner_id, scheduled));
        }

        self.released = true;
    }
}

#[derive(Clone)]
struct HiddenEvent {
    header_time: u64,
    network_delay: u64,
}

impl HiddenEvent {
    fn into_event(self, miner_id: usize, release_time: u64) -> BlockEvent {
        BlockEvent {
            arrival_time: release_time + self.network_delay,
            header_time: self.header_time,
            miner_id,
            network_delay: self.network_delay,
        }
    }
}
