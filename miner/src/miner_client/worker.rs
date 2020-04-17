use crate::miner_client::{nonce_generator, partition_nonce, set_header_nonce};
use anyhow::Result;
use config::{ConsensusStrategy, MinerConfig};
use consensus::difficult::difficult_to_target;
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::SinkExt;
use logger::prelude::*;
use std::thread;
use std::time::Duration;
use types::{H256, U256};
pub fn start_worker(
    config: &MinerConfig,
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) -> WorkerController {
    match config.consensus_strategy {
        ConsensusStrategy::Argon => {
            let thread_num = config.thread_num;
            let worker_txs = (0..thread_num)
                .map(|i| {
                    let (worker_tx, worker_rx) = mpsc::unbounded();
                    let worker_name = format!("starcoin-miner-argon-worker-{}", i);
                    let nonce_range = partition_nonce(i as u64, thread_num as u64);
                    let nonce_tx_clone = nonce_tx.clone();
                    thread::Builder::new()
                        .name(worker_name.clone())
                        .spawn(move || {
                            let mut worker = Worker::new(worker_rx, nonce_tx_clone);
                            let rng = nonce_generator(nonce_range);
                            worker.run(rng, argon_solver);
                        })
                        .expect("Start worker thread failed");
                    info!("start mine worker: {:?}", worker_name);
                    worker_tx
                })
                .collect();
            WorkerController::new(worker_txs)
        }
        ConsensusStrategy::Dummy => {
            let (worker_tx, worker_rx) = mpsc::unbounded();
            let worker_name = "starcoin-miner-dummy-worker".to_owned();
            let nonce_tx_clone = nonce_tx.clone();
            let nonce_range = partition_nonce(1 as u64, 2 as u64);
            thread::Builder::new()
                .name(worker_name)
                .spawn(move || {
                    let mut worker = Worker::new(worker_rx, nonce_tx_clone);
                    let rng = nonce_generator(nonce_range);
                    worker.run(rng, dummy_solver);
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
}

impl Worker {
    pub fn new(
        worker_rx: mpsc::UnboundedReceiver<WorkerMessage>,
        nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    ) -> Self {
        Self {
            nonce_tx,
            worker_rx,
            diff: U256::max_value(),
            pow_header: None,
            start: false,
        }
    }

    fn run<G: FnMut() -> u64, S: Fn(&[u8], u64, U256, mpsc::UnboundedSender<(Vec<u8>, u64)>)>(
        &mut self,
        mut rng: G,
        solver: S,
    ) {
        loop {
            self.refresh_new_work();
            if self.start {
                if let Some(pow_header) = self.pow_header.clone() {
                    solver(&pow_header, rng(), self.diff.clone(), self.nonce_tx.clone());
                }
            } else {
                // Wait next work
                thread::sleep(Duration::from_millis(300));
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

fn argon2_hash(input: &[u8]) -> Result<H256> {
    let mut config = argon2::Config::default();
    config.mem_cost = 1024;
    let output = argon2::hash_raw(input, input, &config)?;
    let h_256: H256 = output.as_slice().into();
    Ok(h_256)
}

fn argon_solver(
    pow_header: &[u8],
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) {
    let input = set_header_nonce(pow_header, nonce);
    if let Ok(pow_hash) = argon2_hash(&input) {
        let pow_hash_u256: U256 = pow_hash.into();
        let target = difficult_to_target(diff);
        if pow_hash_u256 <= target {
            info!("Seal found {:?}", nonce);
            if let Err(e) = block_on(nonce_tx.send((pow_header.to_vec(), nonce))) {
                error!("Failed to send nonce: {:?}", e);
            };
        }
    }
}

fn dummy_solver(
    pow_header: &[u8],
    nonce: u64,
    diff: U256,
    mut nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
) {
    let time: u64 = diff.as_u64();
    debug!("DummyConsensus rand sleep time : {}", time);
    thread::sleep(Duration::from_millis(time));
    if let Err(e) = block_on(nonce_tx.send((pow_header.to_vec(), nonce))) {
        error!("Failed to send nonce: {:?}", e);
    };
}
