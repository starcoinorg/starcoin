use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use config::{ConsensusStrategy, MinerConfig};
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::SinkExt;
use logger::prelude::*;
use rand::Rng;
use std::ops::Range;
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
                        .name(worker_name)
                        .spawn(move || {
                            let mut worker = ArgonWorker::new(worker_rx, nonce_tx_clone);
                            let rng = nonce_generator(nonce_range);
                            worker.run(rng)
                        })
                        .expect("Start worker thread failed");
                    worker_tx
                })
                .collect();
            WorkerController::new(worker_txs)
        }
        ConsensusStrategy::Dummy => unimplemented!(),
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

struct ArgonWorker {
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u64)>,
    worker_rx: mpsc::UnboundedReceiver<WorkerMessage>,
    diff: U256,
    pow_header: Option<Vec<u8>>,
    start: bool,
}

impl ArgonWorker {
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
    fn argon2_hash(input: &[u8]) -> Result<H256> {
        let config = argon2::Config::default();
        let output = argon2::hash_raw(input, input, &config)?;
        let h_256: H256 = output.as_slice().into();
        Ok(h_256)
    }

    fn solve(&mut self, pow_header: &[u8], nonce: u64) {
        let input = set_header_nonce(pow_header, nonce);
        if let Ok(pow_hash) = ArgonWorker::argon2_hash(&input) {
            let pow_hash_u256: U256 = pow_hash.into();
            if pow_hash_u256 <= self.diff {
                info!("Seal found {:?}", nonce);
                if let Err(e) = block_on(self.nonce_tx.send((pow_header.to_vec(), nonce))) {
                    error!("Failed to send nonce: {:?}", e);
                };
            }
        }
    }

    fn run<G: FnMut() -> u64>(&mut self, mut rng: G) {
        loop {
            self.refresh_new_work();
            if self.start {
                if let Some(pow_header) = self.pow_header.clone() {
                    self.solve(&pow_header, rng());
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

pub fn set_header_nonce(header: &[u8], nonce: u64) -> Vec<u8> {
    let len = header.len();
    let mut header = header.to_owned();
    header.truncate(len - 8);
    let _ = header.write_u64::<LittleEndian>(nonce);
    header
}
