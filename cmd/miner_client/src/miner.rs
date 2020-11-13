// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::solver::create_solver;
use crate::{JobClient, SealEvent, Solver};
use anyhow::Result;
use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use futures::executor::block_on;
use futures::stream::StreamExt;
use futures::SinkExt;
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
    nonce_rx: Option<mpsc::UnboundedReceiver<(Vec<u8>, u32)>>,
    nonce_tx: mpsc::UnboundedSender<(Vec<u8>, u32)>,
    job_client: C,
    pb: Option<ProgressBar>,
    num_seals_found: Mutex<u32>,
    solver: Box<dyn Solver>,
    current_task: Option<mpsc::UnboundedSender<bool>>,
}

impl<C: JobClient> MinerClient<C> {
    pub fn new(config: MinerClientConfig, job_client: C, solver: Box<dyn Solver>) -> Result<Self> {
        let (nonce_tx, nonce_rx) = mpsc::unbounded();
        let pb = if config.enable_stderr {
            let mp = MultiProgress::new();
            let pb = mp.add(ProgressBar::new(10));
            pb.set_style(ProgressStyle::default_bar().template("{msg:.green}"));
            thread::spawn(move || {
                mp.join().expect("MultiProgress join failed");
            });
            Some(pb)
        } else {
            None
        };
        Ok(Self {
            nonce_rx: Some(nonce_rx),
            nonce_tx,
            job_client,
            pb,
            num_seals_found: Mutex::new(0),
            solver,
            current_task: None,
        })
    }
    fn submit_seal(&self, minting_blob: Vec<u8>, nonce: u32) {
        if let Err(err) = self.job_client.submit_seal(minting_blob, nonce) {
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

    fn start_mint_work(&mut self, strategy: ConsensusStrategy, minting_hash: &[u8], diff: U256) {
        let (stop_tx, stop_rx) = unbounded();
        if let Some(mut task) = self.current_task.take() {
            if let Err(e) = block_on(task.send(true)) {
                debug!(
                    "Failed to send stop event to current task, may be finished:{:?}",
                    e
                );
            };
        }
        self.current_task = Some(stop_tx);
        let nonce_tx = self.nonce_tx.clone();
        let minting_hash = minting_hash.to_owned();

        let mut solver = dyn_clone::clone_box(&*self.solver);
        //this will block on handle Sealevent if use actix spawn
        thread::spawn(move || solver.solve(strategy, &minting_hash, diff, nonce_tx, stop_rx));
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
            .map(|(minting_blob, nonce)| SealEvent {
                minting_blob,
                nonce,
            });
        ctx.add_stream(seals);
        Ok(())
    }
}

impl<C: JobClient> ServiceFactory<Self> for MinerClientService<C> {
    fn create(ctx: &mut ServiceContext<MinerClientService<C>>) -> Result<MinerClientService<C>> {
        let config = ctx.get_shared::<MinerClientConfig>()?;
        let job_client = ctx.get_shared::<C>()?;
        let solver = create_solver(config.clone(), Some(job_client.time_service()))?;
        let inner = MinerClient::new(config, job_client, solver)?;
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
            .start_mint_work(event.strategy, &event.minting_blob, event.difficulty);
    }
}

impl<C: JobClient> EventHandler<Self, SealEvent> for MinerClientService<C> {
    fn handle_event(&mut self, event: SealEvent, _ctx: &mut ServiceContext<MinerClientService<C>>) {
        self.inner.submit_seal(event.minting_blob, event.nonce)
    }
}
