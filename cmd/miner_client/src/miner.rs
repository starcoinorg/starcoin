// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::worker::{start_worker, WorkerController, WorkerMessage};
use crate::{JobClient, SealEvent};
use anyhow::Result;
use crypto::HashValue;
use futures::channel::mpsc;
use futures::executor::block_on;
use futures::stream::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use logger::prelude::*;
use starcoin_config::MinerClientConfig;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::U256;
use std::sync::Mutex;
use std::thread;

pub struct MinerClient<C: JobClient> {
    nonce_rx: Option<mpsc::UnboundedReceiver<(Vec<u8>, u64)>>,
    worker_controller: WorkerController,
    job_client: C,
    pb: Option<ProgressBar>,
    num_seals_found: Mutex<u64>,
}

impl<C: JobClient> MinerClient<C> {
    pub fn new(config: MinerClientConfig, job_client: C) -> Result<Self> {
        let (nonce_tx, nonce_rx) = mpsc::unbounded();
        let (worker_controller, pb) = if config.enable_stderr {
            let mp = MultiProgress::new();
            let pb = mp.add(ProgressBar::new(10));
            pb.set_style(ProgressStyle::default_bar().template("{msg:.green}"));
            let worker_controller =
                start_worker(&config, nonce_tx, Some(&mp), job_client.time_service());
            thread::spawn(move || {
                mp.join().expect("MultiProgress join failed");
            });
            (worker_controller, Some(pb))
        } else {
            let worker_controller =
                start_worker(&config, nonce_tx, None, job_client.time_service());
            (worker_controller, None)
        };
        Ok(Self {
            nonce_rx: Some(nonce_rx),
            worker_controller,
            job_client,
            pb,
            num_seals_found: Mutex::new(0),
        })
    }
    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) {
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

    fn start_mint_work(&self, strategy: ConsensusStrategy, minting_hash: HashValue, diff: U256) {
        block_on(self.worker_controller.send_message(WorkerMessage::NewWork {
            strategy,
            minting_hash,
            diff,
        }))
    }
}

pub struct MinerClientService<C: JobClient> {
    inner: MinerClient<C>,
}

impl<C: JobClient> ActorService for MinerClientService<C> {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let jobs = self.inner.job_client.subscribe()?;
        ctx.add_stream(jobs);
        let seals = self
            .inner
            .nonce_rx
            .take()
            .expect("Inner error for take nonce rx")
            .map(|(pow_hash, nonce)| {
                let pow_hash = HashValue::from_slice(&pow_hash).expect("Inner error, invalid hash");
                SealEvent { pow_hash, nonce }
            });
        ctx.add_stream(seals);
        Ok(())
    }
}

impl<C: JobClient> ServiceFactory<Self> for MinerClientService<C> {
    fn create(ctx: &mut ServiceContext<MinerClientService<C>>) -> Result<MinerClientService<C>> {
        let config = ctx.get_shared::<MinerClientConfig>()?;
        let job_client = ctx.get_shared::<C>()?;
        let inner = MinerClient::new(config, job_client)?;
        Ok(Self { inner })
    }
}

impl<C: JobClient> EventHandler<Self, MintBlockEvent> for MinerClientService<C> {
    fn handle_event(
        &mut self,
        event: MintBlockEvent,
        _ctx: &mut ServiceContext<MinerClientService<C>>,
    ) {
        self.inner
            .start_mint_work(event.strategy, event.minting_hash, event.difficulty);
    }
}

impl<C: JobClient> EventHandler<Self, SealEvent> for MinerClientService<C> {
    fn handle_event(&mut self, event: SealEvent, _ctx: &mut ServiceContext<MinerClientService<C>>) {
        self.inner.submit_seal(event.pow_hash, event.nonce)
    }
}
