use crate::{difficulty_to_target_hex, rpc::*};
use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use jsonrpc_pubsub::SubscriptionId;
use starcoin_logger::prelude::*;
use starcoin_miner::{
    MinerService, SubmitSealRequest as MinerSubmitSealRequest, UpdateSubscriberNumRequest,
};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_types::system_events::MintBlockEvent;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::atomic;

pub struct Stratum {
    uid: atomic::AtomicU32,
    mint_block_subscribers:
        HashMap<WorkerId, (mpsc::UnboundedSender<StratumJobResponse>, MinerWorker)>,
    miner_service: ServiceRef<MinerService>,
}

impl Stratum {
    fn new(miner_service: ServiceRef<MinerService>) -> Self {
        Self {
            miner_service,
            uid: atomic::AtomicU32::new(1),
            mint_block_subscribers: Default::default(),
        }
    }

    fn next_id(&self) -> u32 {
        self.uid.fetch_add(1, atomic::Ordering::SeqCst)
    }

    fn sync_upstream_job(&mut self) -> Result<Option<MintBlockEvent>> {
        let service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        futures::executor::block_on(service.send(UpdateSubscriberNumRequest {
            number: Some(subscribers_num),
        }))
    }

    fn get_downstream_job(
        miner: &MinerWorker,
        set_login: bool,
        upstreaum_event: &MintBlockEvent,
    ) -> StratumJobResponse {
        let login = miner.base_info.clone();
        
        let target = miner.diff_manager.read().unwrap().get_target();

        let job = StratumJobResponse::from(
            upstreaum_event,
            if set_login { Some(login) } else { None },
            miner.worker_id,
            target,
        );
        return job;
    }

    fn dispatch_job_to_clients(&mut self, event: MintBlockEvent) {
        let mut remove_outdated = vec![];
        for (id, (ch, worker)) in self.mint_block_subscribers.iter() {
            let job = Self::get_downstream_job(&worker, false, &event);
            info!(target: "stratum", "dispatch startum job:{:?}", job);
            if let Err(err) = ch.unbounded_send(job) {
                if err.is_disconnected() {
                    remove_outdated.push(*id);
                } else if err.is_full() {
                    error!(target: "stratum", "subscription {:?} fail to new messages, channel is full", id);
                }
            }
        }
        for id in remove_outdated {
            self.mint_block_subscribers.remove(&id);
        }
    }
}

impl ActorService for Stratum {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<MintBlockEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MintBlockEvent>();
        Ok(())
    }
}

impl EventHandler<Self, MintBlockEvent> for Stratum {
    fn handle_event(&mut self, event: MintBlockEvent, _ctx: &mut ServiceContext<Stratum>) {
        self.dispatch_job_to_clients(event);
    }
}

impl ServiceHandler<Self, SubscribeJobEvent> for Stratum {
    fn handle(&mut self, msg: SubscribeJobEvent, ctx: &mut ServiceContext<Self>) {
        info!(target: "stratum", "receive subscribe event {:?}", msg);
        let SubscribeJobEvent(subscriber, login) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let sub_id = self.next_id();

        ctx.spawn(async move {
            if let Ok(sink) = subscriber
                .assign_id_async(SubscriptionId::Number(sub_id as u64))
                .await
            {
                let forward = receiver
                    .flat_map(move |m| {
                        let r = vec![Ok(m)];
                        futures::stream::iter(
                            r.into_iter().map(Ok::<_, jsonrpc_pubsub::TransportError>),
                        )
                    })
                    .forward(sink)
                    .await;
                if let Err(e) = forward {
                    warn!(target: "stratum", "Unable to send notification: {}", e);
                }
            } else {
                error!(target: "stratum", "Subscriber assign is failed");
            }
        });
        if let Ok(Some(event)) = self.sync_upstream_job() {
            let miner_worker = MinerWorker::new(sub_id, login.clone());
            let downstream_job = Self::get_downstream_job(&miner_worker, true, &event);
            self.mint_block_subscribers.insert(
                miner_worker.worker_id.clone(),
                (sender.clone(), miner_worker),
            );
            ctx.spawn(async move {
                info!(target:"stratum", "Respond to stratum subscribe:{:?}", downstream_job);
                if let Err(err) = sender.unbounded_send(downstream_job) {
                    error!(target: "stratum", "Failed to send MintBlockEvent: {}", err);
                }
            });
        } else {
            warn!(target: "stratum", "current mint job is empty");
        }
    }
}

impl ServiceHandler<Self, SubmitShareEvent> for Stratum {
    fn handle(&mut self, msg: SubmitShareEvent, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        info!(target: "stratum", "received submit share event:{:?}", &msg.0);

        if let Some(current_mint_event) = self.sync_upstream_job()? {
            let job_id = JobId::new(&msg.0.job_id)?;
            let submit_job_id = JobId::from_bob(&current_mint_event.minting_blob);
            if job_id != submit_job_id {
                warn!(target: "stratum", "received job mismatch with current job,{:?},{:?}",job_id, submit_job_id);
                return Ok(());
            };
            let worker_id = WorkerId::from_hex(msg.0.id.clone())?;
            if let Some((_, worker)) = self.mint_block_subscribers.get_mut(&worker_id) {
                worker.process_seal(&msg.0, &current_mint_event);
            };            
            let mut seal: MinerSubmitSealRequest = msg.0.try_into()?;

            seal.minting_blob = current_mint_event.minting_blob;
            self.miner_service.try_send(seal)?;
        }
        Ok(())
    }
}

pub struct StratumFactory;

impl ServiceFactory<Stratum> for StratumFactory {
    fn create(ctx: &mut ServiceContext<Stratum>) -> Result<Stratum> {
        let miner_service = ctx.service_ref::<MinerService>()?.clone();
        Ok(Stratum::new(miner_service))
    }
}
