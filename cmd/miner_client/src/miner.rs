// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::JobClient;
use actix::{Actor, Arbiter, Context};
use anyhow::Result;
use crypto::HashValue;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use starcoin_config::{ConsensusStrategy, MinerClientConfig};
use starcoin_types::U256;
use std::sync::Mutex;
use std::thread;

pub struct MinerClient<C>
where
    C: JobClient,
{
    nonce_rx: mpsc::UnboundedReceiver<(Vec<u8>, u64)>,
    worker_controller: WorkerController,
    job_client: C,
    pb: Option<ProgressBar>,
    num_seals_found: Mutex<u64>,
}

impl<C> MinerClient<C>
where
    C: JobClient,
{
    pub fn new(
        config: MinerClientConfig,
        consensus_strategy: ConsensusStrategy,
        job_client: C,
    ) -> Self {
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
        Self {
            nonce_rx,
            worker_controller,
            job_client,
            pb,
            num_seals_found: Mutex::new(0),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Miner client started");
        let mut job_rx = self.job_client.subscribe()?.fuse();

        loop {
            debug!("In miner client select loop");
            futures::select! {
                job = job_rx.select_next_some() => {
                    match job{
                        Ok(job)=>{
                            let (pow_hash, diff) =job;
                            self.start_mint_work(pow_hash, diff).await;

                        }
                        Err(e)=>{error!("read subscribed job error:{}",e)}
                    }
                },

                seal = self.nonce_rx.select_next_some() => {
                    let (pow_header, nonce) = seal;
                    let hash = HashValue::from_slice(&pow_header).expect("Inner error, invalid hash");
                    self.submit_seal(hash, nonce).await;
                }
            }
        }
    }

    async fn submit_seal(&self, pow_header: HashValue, nonce: u64) {
        self.worker_controller
            .send_message(WorkerMessage::Stop)
            .await;
        if let Err(err) = self.job_client.submit_seal(pow_header, nonce) {
            error!("Submit seal to failed: {:?}", err);
            return;
        }
        {
            *self.num_seals_found.lock().unwrap() += 1;
            let msg = format!(
                "Miner client Total seals found: {:>3}",
                *self.num_seals_found.lock().unwrap()
            );
            if let Some(pb) = self.pb.as_ref() {
                pb.set_message(&msg);
                pb.inc(1);
            } else {
                info!("{}", msg)
            }
        }
    }

    async fn start_mint_work(&self, pow_header: HashValue, diff: U256) {
        self.worker_controller
            .send_message(WorkerMessage::NewWork {
                pow_header: pow_header.to_vec(),
                diff,
            })
            .await
    }
}

pub struct MinerClientActor<C>
where
    C: JobClient,
{
    config: MinerClientConfig,
    consensus_strategy: ConsensusStrategy,
    job_client: C,
}

impl<C> MinerClientActor<C>
where
    C: JobClient,
{
    pub fn new(
        config: MinerClientConfig,
        consensus_strategy: ConsensusStrategy,
        job_client: C,
    ) -> Self {
        MinerClientActor {
            config,
            consensus_strategy,
            job_client,
        }
    }
}

impl<C> Actor for MinerClientActor<C>
where
    C: JobClient + Unpin + 'static + Clone + Send + Sync,
{
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        let config = self.config.clone();
        let consensus_strategy = self.consensus_strategy;
        let job_client = self.job_client.clone();
        let arbiter = Arbiter::new();
        let fut = async move {
            let mut miner_cli = MinerClient::new(config, consensus_strategy, job_client);
            miner_cli.start().await.unwrap();
        };
        arbiter.send(Box::pin(fut));
        info!("MinerClientActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("MinerClientActor stopped");
    }
}
