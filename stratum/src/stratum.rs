use crate::rpc::*;
use anyhow::Result;
use futures::channel::mpsc;
use futures::StreamExt;
use jsonrpc_pubsub::SubscriptionId;
use starcoin_logger::prelude::*;
use starcoin_miner::{MinerClientSubscribeRequest, MinerService};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_types::system_events::{MintBlockEvent, SubmitSealEvent};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::atomic;

pub struct Stratum {
    uid: atomic::AtomicU64,
    mint_block_subscribers:
        HashMap<SubscriptionId, (mpsc::UnboundedSender<StratumJobResponse>, LoginRequest)>,
    miner_service: ServiceRef<MinerService>,
}

impl Stratum {
    fn new(miner_service: ServiceRef<MinerService>) -> Self {
        Self {
            miner_service,
            uid: atomic::AtomicU64::new(1),
            mint_block_subscribers: Default::default(),
        }
    }
    fn next_id(&self) -> SubscriptionId {
        SubscriptionId::Number(1)
    }
    fn sync_current_job(&mut self) -> Result<Option<MintBlockEvent>> {
        let service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        let current_mint_event = futures::executor::block_on(
            service.send(MinerClientSubscribeRequest::Add(subscribers_num)),
        )??;
        Ok(current_mint_event)
    }
    fn send_to_all(&mut self, event: MintBlockEvent) {
        let mut remove_outdated = vec![];
        for (id, (ch, login)) in self.mint_block_subscribers.iter() {
            let worker_id = login.get_worker_id();
            let job = StratumJobResponse::from(&event, None, worker_id);
            if let Err(err) = ch.unbounded_send(job) {
                if err.is_disconnected() {
                    remove_outdated.push(id.clone());
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
        self.send_to_all(event);
    }
}

impl ServiceHandler<Self, Unsubscribe> for Stratum {
    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut ServiceContext<Self>) {
        self.mint_block_subscribers.remove(&msg.0);
        self.uid.fetch_sub(1, atomic::Ordering::SeqCst);
        self.miner_service
            .do_send(MinerClientSubscribeRequest::Remove(
                self.mint_block_subscribers.len() as u32,
            ));
    }
}

impl ServiceHandler<Self, SubscribeJobEvent> for Stratum {
    fn handle(&mut self, msg: SubscribeJobEvent, ctx: &mut ServiceContext<Self>) {
        info!(target: "stratum", "receive subscribe event {:?}", msg);
        let SubscribeJobEvent(subscriber, login) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let sub_id = self.next_id();
        self.mint_block_subscribers
            .insert(sub_id.clone(), (sender.clone(), login.clone()));
        ctx.spawn(async move {
            if let Ok(sink) = subscriber.assign_id_async(sub_id).await {
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
        if let Ok(Some(event)) = self.sync_current_job() {
            ctx.spawn(async move {
                let worker_id = login.get_worker_id();
                let stratum_result = StratumJobResponse::from(&event, Some(login), worker_id);
                if let Err(err) = sender.unbounded_send(stratum_result) {
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
        if let Some(current_mint_event) = self.sync_current_job()? {
            let submit_job_id = msg.0.job_id.clone();
            let job_id = hex::encode(&current_mint_event.minting_blob[0..8]);
            if submit_job_id != job_id {
                warn!(target: "stratum", "received job mismatch with current job,{},{}", submit_job_id, job_id);
                return Ok(());
            }
            let mut seal: SubmitSealEvent = msg.0.try_into()?;
            seal.minting_blob = current_mint_event.minting_blob;
            self.miner_service.notify(seal)?
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
