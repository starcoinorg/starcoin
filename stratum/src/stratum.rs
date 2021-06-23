use crate::rpc::*;
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
use std::convert::{TryFrom, TryInto};
use std::sync::atomic;

pub struct Stratum {
    uid: atomic::AtomicU32,
    mint_block_subscribers: HashMap<u32, (mpsc::UnboundedSender<StratumJobResponse>, LoginRequest)>,
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
    fn sync_current_job(&mut self) -> Result<Option<MintBlockEvent>> {
        let service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        futures::executor::block_on(service.send(UpdateSubscriberNumRequest {
            number: Some(subscribers_num),
        }))
    }
    fn send_to_all(&mut self, event: MintBlockEvent) {
        let mut remove_outdated = vec![];
        for (id, (ch, login)) in self.mint_block_subscribers.iter() {
            let worker_id = login.get_worker_id(*id);
            let job = StratumJobResponse::from(&event, None, worker_id);
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
        self.send_to_all(event);
    }
}

impl ServiceHandler<Self, Unsubscribe> for Stratum {
    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut ServiceContext<Self>) {
        if let SubscriptionId::Number(id) = &msg.0 {
            if let Ok(id) = u32::try_from(*id) {
                if self
                    .miner_service
                    .try_send(UpdateSubscriberNumRequest {
                        number: Some(self.mint_block_subscribers.len() as u32 - 1),
                    })
                    .is_ok()
                {
                    self.mint_block_subscribers.remove(&id);
                    self.uid.fetch_sub(1, atomic::Ordering::SeqCst);
                    return;
                }
            }
        }
        error!(target: "stratum", "Failed to send unsubscribe message to miner service")
    }
}

impl ServiceHandler<Self, SubscribeJobEvent> for Stratum {
    fn handle(&mut self, msg: SubscribeJobEvent, ctx: &mut ServiceContext<Self>) {
        info!(target: "stratum", "receive subscribe event {:?}", msg);
        let SubscribeJobEvent(subscriber, login) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let sub_id = self.next_id();
        self.mint_block_subscribers
            .insert(sub_id, (sender.clone(), login.clone()));
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
        if let Ok(Some(event)) = self.sync_current_job() {
            ctx.spawn(async move {
                let worker_id = login.get_worker_id(sub_id);
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
            let job_id = hex::encode(&current_mint_event.minting_blob[0..8]);
            let submit_job_id = msg.0.job_id.clone();
            if submit_job_id != job_id {
                warn!(target: "stratum", "received job mismatch with current job,{},{}", submit_job_id, job_id);
                return Ok(());
            };
            let mut seal: MinerSubmitSealRequest = msg.0.try_into()?;

            seal.minting_blob = current_mint_event.minting_blob;
            let _ = self.miner_service.try_send(seal)?;
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
