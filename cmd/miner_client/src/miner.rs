// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::stratum::StratumClient;
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use actix::{Actor, Arbiter, Context, System};
use anyhow::Result;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use starcoin_config::{ConsensusStrategy, MinerClientConfig};
use starcoin_types::U256;
use std::thread;

pub struct Miner {
    job_rx: mpsc::UnboundedReceiver<(Vec<u8>, U256)>,
    nonce_rx: mpsc::UnboundedReceiver<(Vec<u8>, u64)>,
    worker_controller: WorkerController,
    stratum_client: StratumClient,
    pb: Option<ProgressBar>,
    num_seals_found: u64,
}

impl Miner {
    pub async fn new(
        config: MinerClientConfig,
        consensus_strategy: ConsensusStrategy,
    ) -> Result<Self> {
        let mut stratum_client = StratumClient::new(&config)?;
        let job_rx = stratum_client.subscribe().await?;
        let (nonce_tx, nonce_rx) = mpsc::unbounded();
        let (worker_controller, pb) = if config.enable_stderr {
            let mp = MultiProgress::new();
            let pb = mp.add(ProgressBar::new(10));
            pb.set_style(ProgressStyle::default_bar().template("{msg:.green}"));
            let worker_controller = start_worker(&config, consensus_strategy, nonce_tx, Some(&mp));
            thread::spawn(move || {
                mp.join().expect("MultiProgress join failed");
            });
            (worker_controller, Some(pb))
        } else {
            let worker_controller = start_worker(&config, consensus_strategy, nonce_tx, None);
            (worker_controller, None)
        };

        Ok(Self {
            job_rx,
            nonce_rx,
            worker_controller,
            stratum_client,
            pb,
            num_seals_found: 0,
        })
    }

    pub async fn start(&mut self) {
        info!("Miner client started");
        loop {
            debug!("In miner client select loop");
            futures::select! {
                job = self.job_rx.select_next_some() => {
                     let (pow_header, diff) = job;
                     self.start_mint_work(pow_header, diff).await;
                },
                seal = self.nonce_rx.select_next_some() => {
                     let (pow_header, nonce) = seal;
                     self.submit_seal(pow_header, nonce).await;
                }
            }
        }
    }

    async fn submit_seal(&mut self, pow_header: Vec<u8>, nonce: u64) {
        self.worker_controller
            .send_message(WorkerMessage::Stop)
            .await;
        if let Err(err) = self
            .stratum_client
            .submit_seal((pow_header.clone(), nonce))
            .await
        {
            error!("Submit seal to stratum failed: {:?}", err);
            return;
        }
        {
            self.num_seals_found += 1;
            let msg = format!(
                "Miner client Total seals found: {:>3}",
                self.num_seals_found
            );
            if let Some(pb) = self.pb.as_ref() {
                pb.set_message(&msg);
                pb.inc(1);
            } else {
                info!("{}", msg)
            }
        }
    }

    async fn start_mint_work(&mut self, pow_header: Vec<u8>, diff: U256) {
        self.worker_controller
            .send_message(WorkerMessage::NewWork { pow_header, diff })
            .await
    }
}

pub struct MinerClientActor {
    config: MinerClientConfig,
    consensus_strategy: ConsensusStrategy,
}

impl MinerClientActor {
    pub fn new(config: MinerClientConfig, consensus_strategy: ConsensusStrategy) -> Self {
        MinerClientActor {
            config,
            consensus_strategy,
        }
    }
}

impl Actor for MinerClientActor {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        let config = self.config.clone();
        let consensus_strategy = self.consensus_strategy;
        let arbiter = Arbiter::new();
        let fut = async move {
            let miner_cli = Miner::new(config, consensus_strategy).await;
            match miner_cli {
                Err(e) => {
                    error!("Start miner client failed: {:?}", e);
                    System::current().stop();
                }
                Ok(mut miner_cli) => miner_cli.start().await,
            }
        };
        arbiter.send(Box::pin(fut));
        info!("MinerClientActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MinerClientActor stopped");
    }
}
