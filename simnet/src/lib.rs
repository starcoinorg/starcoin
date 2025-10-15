// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub mod scene;
pub mod tips;

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
    pub arrival_time: u64,
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
            miners: cfg.miners,
        }
    }

    pub fn run(&mut self) -> Vec<BlockEvent> {
        let mut events = Vec::new();

        while let Some(produce) = self.queue.pop() {
            if produce.time > self.horizon {
                break;
            }

            let miner = &self.miners[produce.miner_id];
            let raw_arrival = produce.time + miner.network_delay;
            let slot = 200;
            let arrival_time = (raw_arrival / slot) * slot;

            events.push(BlockEvent {
                arrival_time,
                header_time: produce.time,
                miner_id: produce.miner_id,
                network_delay: miner.network_delay,
            });

            let next =
                produce.time + exp_sample(&mut self.rng, self.block_intervals[produce.miner_id]);
            self.queue.push(ProduceEvent {
                time: next,
                miner_id: produce.miner_id,
            });
        }

        events.sort_by_key(|e| (e.arrival_time, e.miner_id));
        events
    }
}

fn exp_sample<R: Rng>(rng: &mut R, mean_interval: u64) -> u64 {
    let u: f64 = rng.gen::<f64>().max(f64::EPSILON);
    (-u.ln() * mean_interval as f64) as u64
}
