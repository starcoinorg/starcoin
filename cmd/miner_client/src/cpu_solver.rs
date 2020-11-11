use crate::Solver;
use consensus::{difficult_to_target, Consensus};
use futures::executor::block_on;
use futures::{SinkExt, StreamExt};
use futures_channel::mpsc;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use logger::prelude::*;
use rand::Rng;
use starcoin_config::{ConsensusStrategy, MinerClientConfig, TimeService};
use starcoin_types::U256;
use std::ops::Range;
use std::sync::Arc;
use std::thread;

#[derive(Clone)]
pub struct CpuSolver {
    config: MinerClientConfig,
    time_service: Arc<dyn TimeService>,
}

impl CpuSolver {
    pub fn new(config: MinerClientConfig, time_service: Arc<dyn TimeService>) -> CpuSolver {
        Self {
            config,
            time_service,
        }
    }

    fn nonce_generator(nonce_range: &Range<u32>) -> u32 {
        let mut rng = rand::thread_rng();
        let Range { start, end } = nonce_range;
        rng.gen_range(start, end)
    }

    fn partition_nonce(id: u32, total: u32) -> Range<u32> {
        let span = u32::max_value() / total;
        let start = span * id;
        let end = match id {
            x if x < total - 1 => start + span,
            x if x == total - 1 => u32::max_value(),
            _ => unreachable!(),
        };
        Range { start, end }
    }
}

impl Solver for CpuSolver {
    fn solve(
        &mut self,
        strategy: ConsensusStrategy,
        minting_blob: &[u8],
        diff: U256,
        nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u32)>,
        mut stop_rx: mpsc::UnboundedReceiver<bool>,
    ) {
        let thread_num = self.config.thread_num;
        let worker_txs = (0..thread_num)
            .map(|i| {
                let worker_name = format!("starcoin-miner-cpu-worker-{}", i);
                let nonce_range = Self::partition_nonce(i as u32, thread_num as u32);
                let (tx, mut rx) = unbounded::<bool>();
                let mut nonce_tx = nonce_tx.clone();
                let time_service = self.time_service.clone();
                let minting_blob = minting_blob.to_owned();
                let _ = thread::Builder::new()
                    .name(worker_name)
                    .spawn(move || loop {
                        if rx.try_next().is_ok() {
                            break;
                        }
                        match strategy {
                            ConsensusStrategy::Dummy => {
                                let nonce = strategy.solve_consensus_nonce(
                                    &minting_blob,
                                    diff,
                                    time_service.as_ref(),
                                );
                                if let Err(e) = block_on(nonce_tx.send((minting_blob, nonce))) {
                                    error!("Failed to send nonce: {:?}", e);
                                };
                                break;
                            }
                            strategy => {
                                let nonce = Self::nonce_generator(&nonce_range);
                                if let Ok(pow_hash) =
                                    strategy.calculate_pow_hash(&minting_blob, nonce)
                                {
                                    let pow_hash_u256: U256 = pow_hash.into();
                                    let target = difficult_to_target(diff);
                                    if pow_hash_u256 <= target {
                                        info!("Seal found {:?}", nonce);
                                        if let Err(e) =
                                            block_on(nonce_tx.send((minting_blob, nonce)))
                                        {
                                            error!("Failed to send nonce: {:?}", e);
                                        };
                                        break;
                                    }
                                }
                            }
                        }
                    });
                tx
            })
            .collect::<Vec<UnboundedSender<bool>>>();
        block_on(async {
            stop_rx.next().await;
            for mut tx in worker_txs {
                let _ = tx.send(true).await;
            }
        });
    }
}
