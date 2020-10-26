// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{nonce_generator, partition_nonce};
use anyhow::{bail, Result};
use consensus::{difficult_to_target, Consensus};
use crypto::HashValue;
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::SinkExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use starcoin_config::{ConsensusStrategy, MinerClientConfig, TimeService};
use starcoin_types::U256;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const HASH_RATE_UPDATE_DURATION_MILLIS: u128 = 300;

pub fn start_worker(
    config: &MinerClientConfig,
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    mp: Option<&MultiProgress>,
    time_service: Arc<dyn TimeService>,
) -> WorkerController {
    let thread_num = config.thread_num;
    let worker_txs = (0..thread_num)
        .map(|i| {
            let (worker_tx, worker_rx) = mpsc::unbounded();
            let worker_name = format!("starcoin-miner-cpu-worker-{}", i);
            let pb =
                if let Some(mp) = mp {
                    let pb = mp.add(ProgressBar::new(100));
                    pb.set_style(ProgressStyle::default_bar().template(
                        "{prefix:.bold.dim} {spinner:.cyan/blue} [{elapsed_precise}] {msg}",
                    ));
                    pb.set_prefix(&worker_name);
                    Some(pb)
                } else {
                    None
                };
            let nonce_range = partition_nonce(i as u64, thread_num as u64);
            let nonce_tx_clone = nonce_tx.clone();
            let time_service = time_service.clone();
            thread::Builder::new()
                .name(worker_name.clone())
                .spawn(move || {
                    let mut worker = Worker::new(worker_rx, nonce_tx_clone);
                    let rng = nonce_generator(nonce_range);
                    worker.run(rng, solver, pb, time_service.as_ref());
                })
                .expect("Start worker thread failed");
            info!("start mine worker: {:?}", worker_name);
            worker_tx
        })
        .collect();
    WorkerController::new(worker_txs)
}

#[derive(Clone)]
pub enum WorkerMessage {
    Stop,
    NewWork {
        strategy: ConsensusStrategy,
        minting_hash: HashValue,
        diff: U256,
    },
}

impl Default for WorkerMessage {
    fn default() -> Self {
        Self::Stop
    }
}

pub struct WorkerController {
    inner: Vec<mpsc::UnboundedSender<WorkerMessage>>,
}

impl WorkerController {
    pub fn new(inner: Vec<mpsc::UnboundedSender<WorkerMessage>>) -> Self {
        Self { inner }
    }

    pub async fn send_message(&self, message: WorkerMessage) {
        for mut worker_tx in self.inner.iter() {
            if let Err(err) = worker_tx.send(message.clone()).await {
                error!("worker_tx send error {:?}", err);
            };
        }
    }
}

pub struct Worker {
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    worker_rx: mpsc::UnboundedReceiver<WorkerMessage>,
    strategy: Option<ConsensusStrategy>,
    diff: U256,
    minting_hash: Option<HashValue>,
    start: bool,
    num_seal_found: u64,
}

impl Worker {
    pub fn new(
        worker_rx: mpsc::UnboundedReceiver<WorkerMessage>,
        nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    ) -> Self {
        Self {
            nonce_tx,
            worker_rx,
            strategy: None,
            diff: 1.into(),
            minting_hash: None,
            start: false,
            num_seal_found: 0,
        }
    }

    fn run<
        G: FnMut() -> u64,
        S: Fn(
            ConsensusStrategy,
            HashValue,
            u64,
            U256,
            mpsc::UnboundedSender<(Vec<u8>, u64)>,
            &dyn TimeService,
        ) -> bool,
    >(
        &mut self,
        mut rng: G,
        solver: S,
        pb: Option<ProgressBar>,
        time_service: &dyn TimeService,
    ) {
        let mut hash_counter = 0usize;
        let mut start = Instant::now();
        let pb = pb.as_ref();
        loop {
            if let Err(e) = self.refresh_new_work() {
                error!("refresh new work error: {:?}", e);
                break;
            }
            if self.start {
                if let Some(minting_hash) = self.minting_hash {
                    if let Some(strategy) = self.strategy {
                        hash_counter += 1;
                        if solver(
                            strategy,
                            minting_hash,
                            rng(),
                            self.diff,
                            self.nonce_tx.clone(),
                            time_service,
                        ) {
                            self.start = false;
                            self.num_seal_found += 1;
                            if let Some(pb) = pb {
                                pb.reset_elapsed()
                            }
                        }
                        let elapsed = start.elapsed();
                        if elapsed.as_millis() > HASH_RATE_UPDATE_DURATION_MILLIS {
                            let elapsed_sec: f64 = elapsed.as_nanos() as f64 / 1_000_000_000.0;
                            if let Some(pb) = pb {
                                pb.set_message(&format!(
                                    "Hash rate: {:>10.3} Seals found: {:>3}",
                                    hash_counter as f64 / elapsed_sec,
                                    self.num_seal_found
                                ));
                            }
                            start = Instant::now();
                            hash_counter = 0;
                        }
                    }
                }
            } else {
                // Wait next work
                hash_counter = 0;
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    fn refresh_new_work(&mut self) -> Result<()> {
        match self.worker_rx.try_next() {
            Ok(msg) => match msg {
                Some(msg) => match msg {
                    WorkerMessage::NewWork {
                        strategy,
                        minting_hash,
                        diff,
                    } => {
                        self.strategy = Some(strategy);
                        self.minting_hash = Some(minting_hash);
                        self.diff = diff;
                        self.start = true;

                        Ok(())
                    }
                    WorkerMessage::Stop => {
                        self.start = false;
                        Ok(())
                    }
                },
                None => bail!("Receiver get None, channel is closed."),
            },
            Err(_) => {
                trace!("work channel is empty.");
                Ok(())
            }
        }
    }
}

fn solver(
    strategy: ConsensusStrategy,
    minting_hash: HashValue,
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    time_service: &dyn TimeService,
) -> bool {
    match strategy {
        ConsensusStrategy::Dummy => {
            let nonce = strategy.solve_consensus_nonce(minting_hash, diff, time_service);
            if let Err(e) = block_on(nonce_tx.send((minting_hash.to_vec(), nonce))) {
                error!("Failed to send nonce: {:?}", e);
                return false;
            };
            true
        }
        strategy => {
            if let Ok(pow_hash) = strategy.calculate_pow_hash(minting_hash, nonce) {
                let pow_hash_u256: U256 = pow_hash.into();
                let target = difficult_to_target(diff);
                if pow_hash_u256 <= target {
                    info!("Seal found {:?}", nonce);
                    if let Err(e) = block_on(nonce_tx.send((minting_hash.to_vec(), nonce))) {
                        error!("Failed to send nonce: {:?}", e);
                        return false;
                    };
                    return true;
                }
            }
            false
        }
    }
}
