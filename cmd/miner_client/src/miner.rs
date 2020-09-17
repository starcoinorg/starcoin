// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::JobClient;
use actix_rt::Arbiter;
use anyhow::Result;
use crypto::HashValue;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use starcoin_config::MinerClientConfig;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory};
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
    pub fn new(config: MinerClientConfig, job_client: C) -> Result<Self> {
        let consensus_strategy = job_client.consensus()?;
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
            nonce_rx,
            worker_controller,
            job_client,
            pb,
            num_seals_found: Mutex::new(0),
        })
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
                            let (minting_hash, diff) =job;
                            self.start_mint_work(minting_hash, diff).await;

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

    async fn submit_seal(&self, pow_hash: HashValue, nonce: u64) {
        self.worker_controller
            .send_message(WorkerMessage::Stop)
            .await;
        if let Err(err) = self.job_client.submit_seal(pow_hash, nonce) {
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

    async fn start_mint_work(&self, minting_hash: HashValue, diff: U256) {
        self.worker_controller
            .send_message(WorkerMessage::NewWork { minting_hash, diff })
            .await
    }
}

pub struct MinerClientService<C>
where
    C: JobClient + Send + Unpin + Clone + Sync + 'static,
{
    config: MinerClientConfig,
    job_client: C,
}

impl<C> ActorService for MinerClientService<C>
where
    C: JobClient + Send + Unpin + Clone + Sync,
{
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) {
        let config = self.config.clone();
        let job_client = self.job_client.clone();
        let arbiter = Arbiter::new();
        let fut = async move {
            let mut miner_cli = match MinerClient::new(config, job_client) {
                Err(e) => {
                    error!("Create MinerClient error: {:?}", e);
                    return;
                }
                Ok(cli) => cli,
            };
            if let Err(e) = miner_cli.start().await {
                error!("Start MinerClient error: {:?}", e);
            }
        };
        arbiter.send(Box::pin(fut));
        //FIXME if use cxt.wait, actor can not quit graceful, because MinerClient.start is a loop
        //TODO refactor MinerClient, and support graceful quit.
        //ctx.wait(fut)
    }
}

impl<C> MinerClientService<C>
where
    C: JobClient + Send + Unpin + Clone + Sync,
{
    pub fn new(config: MinerClientConfig, job_client: C) -> Self {
        MinerClientService { config, job_client }
    }
}

impl<C> ServiceFactory<Self> for MinerClientService<C>
where
    C: JobClient + Send + Unpin + Clone + Sync,
{
    fn create(ctx: &mut ServiceContext<MinerClientService<C>>) -> Result<MinerClientService<C>> {
        let config = ctx.get_shared::<MinerClientConfig>()?;
        let job_client = ctx.get_shared::<C>()?;
        Ok(Self::new(config, job_client))
    }
}
