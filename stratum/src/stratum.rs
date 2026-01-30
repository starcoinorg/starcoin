use crate::{rpc::*, target_hex_to_difficulty};
use anyhow::Result;
use futures::channel::mpsc;
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
        info!(
            "set downstream job diff:{:?}",
            target_hex_to_difficulty(&target).unwrap()
        );
        StratumJobResponse::from(
            upstreaum_event,
            if set_login { Some(login) } else { None },
            miner.worker_id,
            target,
        )
    }

    fn dispatch_job_to_clients(&mut self, event: MintBlockEvent) {
        let mut remove_outdated = vec![];
        for (id, (ch, worker)) in self.mint_block_subscribers.iter() {
            let job = Self::get_downstream_job(worker, false, &event);
            info!(target: "stratum", "dispatch startum job:{:?}", job);
            if let Err(err) = ch.unbounded_send(job) {
                if err.is_disconnected() {
                    warn!("stratum disconnect worker:{:?}", err);
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
    fn handle(
        &mut self,
        msg: SubscribeJobEvent,
        _ctx: &mut ServiceContext<Self>,
    ) -> anyhow::Result<mpsc::UnboundedReceiver<StratumJobResponse>> {
        let SubscribeJobEvent(login) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let sub_id = self.next_id();
        info!(target: "stratum", "receive subscribe event {:?},sub_id:{}", login, sub_id);
        let miner_worker = MinerWorker::new(sub_id, login);
        let worker_id = miner_worker.worker_id;
        self.mint_block_subscribers
            .insert(worker_id, (sender.clone(), miner_worker));
        let event = self.sync_upstream_job()?;
        let downstream_job = event.as_ref().and_then(|event| {
            self.mint_block_subscribers
                .get(&worker_id)
                .map(|(_, worker)| Self::get_downstream_job(worker, true, event))
        });
        if let Some(downstream_job) = downstream_job {
            info!(target:"stratum", "Respond to stratum subscribe:{:?}", downstream_job);
            if let Err(err) = sender.unbounded_send(downstream_job) {
                error!(target: "stratum", "Failed to send MintBlockEvent: {}", err);
            }
        } else {
            warn!(target: "stratum", "current mint job is empty");
        }
        Ok(receiver)
    }
}

impl ServiceHandler<Self, SubmitShareEvent> for Stratum {
    fn handle(&mut self, msg: SubmitShareEvent, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        info!(target: "stratum", "received submit share event:{:?}", &msg.0);
        if let Some(current_mint_event) = self.sync_upstream_job()? {
            let worker_id = WorkerId::from_hex(msg.0.id.clone())?;
            if let Some((_job_sender, worker)) = self.mint_block_subscribers.get(&worker_id) {
                let _updated_diff = worker
                    .diff_manager()
                    .write()
                    .unwrap()
                    .try_update(worker.base_info.login.clone());
            };
            let job_id = JobId::new(&msg.0.job_id)?;
            let submit_job_id = JobId::from_bob(&current_mint_event.minting_blob);
            if job_id != submit_job_id {
                warn!(target: "stratum", "received job mismatch with current job,{:?},{:?}",job_id, submit_job_id);
                return Ok(());
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
