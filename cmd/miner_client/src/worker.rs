// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{nonce_generator, partition_nonce};
use config::{ConsensusStrategy, MinerConfig};
use consensus::{argon, dev, difficulty::difficult_to_target, dummy};
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::SinkExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use std::thread;
use std::time::{Duration, Instant};
use traits::Consensus;
use types::U256;

const HASH_RATE_UPDATE_DURATION_MILLIS: u128 = 300;

pub fn start_worker(
    config: &MinerConfig,
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    mp: Option<&MultiProgress>,
) -> WorkerController {
    match config.consensus_strategy {
        ConsensusStrategy::Argon(thread_num) => {
            let worker_txs = (0..thread_num)
                .map(|i| {
                    let (worker_tx, worker_rx) = mpsc::unbounded();
                    let worker_name = format!("starcoin-miner-argon-cpu-worker-{}", i);
                    let pb = if let Some(mp) = mp {
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
                    thread::Builder::new()
                        .name(worker_name.clone())
                        .spawn(move || {
                            let mut worker = Worker::new(worker_rx, nonce_tx_clone);
                            let rng = nonce_generator(nonce_range);
                            worker.run(rng, argon_solver, pb);
                        })
                        .expect("Start worker thread failed");
                    info!("start mine worker: {:?}", worker_name);
                    worker_tx
                })
                .collect();
            WorkerController::new(worker_txs)
        }
        strategy => {
            let (worker_tx, worker_rx) = mpsc::unbounded();
            let worker_name = format!("starcoin-miner-{}-worker", strategy);
            let pb =
                if let Some(mp) = mp.as_ref() {
                    let pb = mp.add(ProgressBar::new(100));
                    pb.set_style(ProgressStyle::default_bar().template(
                        "{prefix:.bold.dim} {spinner:.cyan/blue} [{elapsed_precise}] {msg}",
                    ));
                    pb.set_prefix(&worker_name);
                    Some(pb)
                } else {
                    None
                };
            let nonce_range = partition_nonce(1 as u64, 2 as u64);
            let solver = match strategy {
                ConsensusStrategy::Dev => dev_solver,
                ConsensusStrategy::Dummy => dummy_solver,
                _ => unreachable!("Unsupported consensus {:?}", strategy),
            };
            thread::Builder::new()
                .name(worker_name)
                .spawn(move || {
                    let mut worker = Worker::new(worker_rx, nonce_tx);
                    let rng = nonce_generator(nonce_range);
                    worker.run(rng, solver, pb);
                })
                .expect("Start worker thread failed");
            WorkerController::new(vec![worker_tx])
        }
    }
}

#[derive(Clone)]
pub enum WorkerMessage {
    Stop,
    NewWork { pow_header: Vec<u8>, diff: U256 },
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
    diff: U256,
    pow_header: Option<Vec<u8>>,
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
            diff: 1.into(),
            pow_header: None,
            start: false,
            num_seal_found: 0,
        }
    }

    fn run<
        G: FnMut() -> u64,
        S: Fn(&[u8], u64, U256, mpsc::UnboundedSender<(Vec<u8>, u64)>) -> bool,
    >(
        &mut self,
        mut rng: G,
        solver: S,
        pb: Option<ProgressBar>,
    ) {
        let mut hash_counter = 0usize;
        let mut start = Instant::now();
        let pb = pb.as_ref();
        loop {
            self.refresh_new_work();
            if self.start {
                if let Some(pow_header) = self.pow_header.clone() {
                    hash_counter += 1;
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
                        if solver(&pow_header, rng(), self.diff, self.nonce_tx.clone()) {
                            self.start = false;
                            self.num_seal_found += 1;
                            if let Some(pb) = pb {
                                pb.reset_elapsed()
                            }
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

    fn refresh_new_work(&mut self) {
        if let Ok(msg) = self.worker_rx.try_next() {
            if let Some(msg) = msg {
                match msg {
                    WorkerMessage::NewWork { pow_header, diff } => {
                        self.pow_header = Some(pow_header);
                        self.diff = diff;
                        self.start = true;
                    }
                    WorkerMessage::Stop => {
                        self.start = false;
                    }
                }
            }
        }
    }
}

fn argon_solver(
    pow_header: &[u8],
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) -> bool {
    let input = consensus::set_header_nonce(pow_header, nonce);
    if let Ok(pow_hash) = argon::calculate_hash(&input) {
        let pow_hash_u256: U256 = pow_hash.into();
        let target = difficult_to_target(diff);
        if pow_hash_u256 <= target {
            info!("Seal found {:?}", nonce);
            if let Err(e) = block_on(nonce_tx.send((pow_header.to_vec(), nonce))) {
                error!("Failed to send nonce: {:?}", e);
                return false;
            };
            return true;
        }
    }
    false
}

fn dev_solver(
    pow_header: &[u8],
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) -> bool {
    dev::DevConsensus::solve_consensus_nonce(pow_header, diff);
    if let Err(e) = block_on(nonce_tx.send((pow_header.to_vec(), nonce))) {
        error!("Failed to send nonce: {:?}", e);
        return false;
    };
    true
}

fn dummy_solver(
    pow_header: &[u8],
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) -> bool {
    dummy::DummyConsensus::solve_consensus_nonce(pow_header, diff);
    if let Err(e) = block_on(nonce_tx.send((pow_header.to_vec(), nonce))) {
        error!("Failed to send nonce: {:?}", e);
        return false;
    };
    true
}
