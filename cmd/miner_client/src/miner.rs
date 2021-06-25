// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::solver::create_solver;
use crate::{JobClient, SealEvent};
use anyhow::Result;
use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use futures::executor::block_on;
use futures::SinkExt;
use logger::prelude::*;
use parking_lot::Mutex;
use starcoin_config::MinerClientConfig;
use starcoin_miner_client_api::Solver;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_types::system_events::MintBlockEvent;
use std::thread;

pub struct MinerClient<C: JobClient> {
    nonce_rx: Option<mpsc::UnboundedReceiver<SealEvent>>,
    nonce_tx: mpsc::UnboundedSender<SealEvent>,
    job_client: C,
    num_seals_found: Mutex<u32>,
    solver: Box<dyn Solver>,
    current_task: Option<mpsc::UnboundedSender<bool>>,
}

impl<C: JobClient> MinerClient<C> {
    pub fn new(_config: MinerClientConfig, job_client: C, solver: Box<dyn Solver>) -> Result<Self> {
        let (nonce_tx, nonce_rx) = mpsc::unbounded();
        Ok(Self {
            nonce_rx: Some(nonce_rx),
            nonce_tx,
            job_client,
            num_seals_found: Mutex::new(0),
            solver,
            current_task: None,
        })
    }
    fn submit_seal(&self, seal: SealEvent) {
        if let Err(err) = self.job_client.submit_seal(seal) {
            error!("Submit seal to failed: {}", err);
            return;
        }
        {
            *self.num_seals_found.lock() += 1;
            let msg = format!(
                "Miner client Total seals found: {:>3}",
                *self.num_seals_found.lock()
            );
            info!("{}", msg)
        }
    }

    fn start_mint_work(&mut self, event: MintBlockEvent) {
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
        let mut solver = dyn_clone::clone_box(&*self.solver);
        //this will block on handle Sealevent if use actix spawn
        thread::spawn(move || solver.solve(event, nonce_tx, stop_rx));
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
            .expect("Inner error for take nonce rx");
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
        self.inner.start_mint_work(event);
    }
}

impl<C: JobClient> EventHandler<Self, SealEvent> for MinerClientService<C> {
    fn handle_event(&mut self, event: SealEvent, _ctx: &mut ServiceContext<MinerClientService<C>>) {
        self.inner.submit_seal(event)
    }
}
