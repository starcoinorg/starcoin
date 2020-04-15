use crate::miner_client::worker::{WorkerController, WorkerMessage, start_worker};
use crate::miner_client::stratum::StratumClient;
use futures::channel::mpsc;
use config::MinerConfig;
use anyhow::Result;
use types::U256;
use async_std::task;
use futures::stream::StreamExt;

pub struct Miner {
    job_rx: mpsc::UnboundedReceiver<(Vec<u8>, U256)>,
    nonce_rx: mpsc::UnboundedReceiver<(Vec<u8>, u64)>,
    worker_controller: WorkerController,
    stratum_client: StratumClient,
}

impl Miner {
    pub fn new(config: MinerConfig) -> Result<Self> {
        task::block_on(async move {
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
        self.stratum_client.submit_seal((pow_header, nonce)).await;
        self.worker_controller.send_message(WorkerMessage::Stop).await;
    }

    async fn start_mint_work(&mut self, pow_header: Vec<u8>, diff: U256) {
        self.worker_controller.send_message(WorkerMessage::NewWork { pow_header, diff }).await
    }
}
