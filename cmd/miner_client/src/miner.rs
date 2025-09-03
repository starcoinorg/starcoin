// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
use crate::solver::create_solver;
use crate::{JobClient, SealEvent};
use anyhow::Result;
use futures::channel::mpsc;
use futures::channel::mpsc::unbounded;
use futures::executor::block_on;
use futures::SinkExt;
use parking_lot::Mutex;
use starcoin_config::MinerClientConfig;
use starcoin_logger::prelude::*;
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
}

pub struct MinerClientService<C: JobClient> {
    inner: MinerClient<C>,
}

impl<C: JobClient> ActorService for MinerClientService<C> {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        let job_client = self.inner.job_client.clone();
        let jobs = block_on(job_client.subscribe())?;
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
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<MinerClientConfig>()?;
        let job_client = ctx.get_shared::<C>()?;
        let solver = create_solver(config.clone(), Some(job_client.time_service()))?;
        let inner = MinerClient::new(config, job_client, solver)?;
        Ok(Self { inner })
    }
}

impl<C: JobClient> EventHandler<Self, MintBlockEvent> for MinerClientService<C> {
    fn handle_event(&mut self, event: MintBlockEvent, ctx: &mut ServiceContext<Self>) {
        let (stop_tx, stop_rx) = unbounded();
        if let Some(mut task) = self.inner.current_task.take() {
            ctx.wait(async move {
                if let Err(e) = task.send(true).await {
                    error!(
                        "Failed to send stop event to current task, may be finished:{:?}",
                        e
                    );
                }
            });
        }
        self.inner.current_task = Some(stop_tx);
        let nonce_tx = self.inner.nonce_tx.clone();
        let mut solver = dyn_clone::clone_box(&*self.inner.solver);
        //this will block on handle Sealevent if use ctx spawn
        thread::spawn(move || solver.solve(event, nonce_tx, stop_rx));
    }
}

impl<C: JobClient> EventHandler<Self, SealEvent> for MinerClientService<C> {
    fn handle_event(&mut self, event: SealEvent, ctx: &mut ServiceContext<Self>) {
        {
            *self.inner.num_seals_found.lock() += 1;
            let msg = format!(
                "Miner client Total seals found: {:>3}",
                *self.inner.num_seals_found.lock()
            );
            info!("{}", msg)
        }
        let job_client = self.inner.job_client.clone();
        let fut = async move {
            if let Err(err) = job_client.submit_seal(event).await {
                error!("Submit seal to failed: {}", err);
            }
        };
        ctx.spawn(fut);
    }
}
