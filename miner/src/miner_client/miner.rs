use crate::miner_client::stratum::StratumClient;
use crate::miner_client::worker::{start_worker, WorkerController, WorkerMessage};
use actix::{Actor, Arbiter, Context, System};
use anyhow::Result;
use config::MinerConfig;
use futures::channel::mpsc;
use futures::stream::StreamExt;
use logger::prelude::*;
use types::U256;

pub struct Miner {
    job_rx: mpsc::UnboundedReceiver<(Vec<u8>, U256)>,
    nonce_rx: mpsc::UnboundedReceiver<(Vec<u8>, u64)>,
    worker_controller: WorkerController,
    stratum_client: StratumClient,
}

impl Miner {
    pub async fn new(config: MinerConfig) -> Result<Self> {
        let mut stratum_client = StratumClient::new(&config)?;
        let job_rx = stratum_client.subscribe().await?;
        let (nonce_tx, nonce_rx) = mpsc::unbounded();
        let worker_controller = start_worker(&config, nonce_tx);
        Ok(Self {
            job_rx,
            nonce_rx,
            worker_controller,
            stratum_client,
        })
    }

    pub async fn start(&mut self) {
        loop {
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
        if let Err(err) = self.stratum_client.submit_seal((pow_header, nonce)).await {
            error!("Submit seal to stratum failed: {:?}", err);
        }
    }

    async fn start_mint_work(&mut self, pow_header: Vec<u8>, diff: U256) {
        self.worker_controller
            .send_message(WorkerMessage::NewWork { pow_header, diff })
            .await
    }
}

pub struct MinerClientActor {
    config: MinerConfig,
}

impl MinerClientActor {
    pub fn new(config: MinerConfig) -> Self {
        MinerClientActor { config }
    }
}

impl Actor for MinerClientActor {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        let config = self.config.clone();
        Arbiter::spawn(async move {
            let miner_cli = Miner::new(config).await;
            match miner_cli {
                Err(e) => {
                    error!("Start miner client failed: {:?}", e);
                    System::current().stop();
                }
                Ok(mut miner_cli) => miner_cli.start().await,
            }
        });
    }
}
