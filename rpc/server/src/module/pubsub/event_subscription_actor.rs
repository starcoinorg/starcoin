use super::notify;
use super::pubsub;
use super::EventSubscribers;
use super::NewHeaderSubscribers;
use actix::{ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, WrapFuture};
use anyhow::Result;
use starcoin_bus::{Bus, BusActor, Subscription};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::event::Event;
use starcoin_storage::Store;
use starcoin_types::block::Block;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::system_events::{NewHeadBlock, SyncBegin, SyncDone};
use std::sync::Arc;

pub struct ChainNotifyHandlerActor {
    subscribers: EventSubscribers,
    new_header_subscribers: NewHeaderSubscribers,
    bus: actix::Addr<BusActor>,
    store: Arc<dyn Store>,
    broadcast_txn: bool,
}

impl ChainNotifyHandlerActor {
    pub fn new(
        subscribers: EventSubscribers,
        new_header_subscribers: NewHeaderSubscribers,
        bus: actix::Addr<BusActor>,
        store: Arc<dyn Store>,
    ) -> Self {
        Self {
            subscribers,
            new_header_subscribers,
            bus,
            store,
            broadcast_txn: true,
        }
    }
}

impl actix::Actor for ChainNotifyHandlerActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let sync_begin_recipient = ctx.address().recipient::<SyncBegin>();
        self.bus
            .send(Subscription {
                recipient: sync_begin_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        let sync_done_recipient = ctx.address().recipient::<SyncDone>();
        self.bus
            .send(Subscription {
                recipient: sync_done_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        self.bus
            .clone()
            .channel::<NewHeadBlock>()
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Err(e) => {
                        error!(target: "pubsub", "fail to start event subscription actor, err: {}", &e);
                        ctx.terminate();
                    }
                    Ok(r) => {
                        ctx.add_stream(r);
                    }
                };
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

impl actix::Handler<SyncBegin> for ChainNotifyHandlerActor {
    type Result = ();

    fn handle(&mut self, _begin: SyncBegin, _ctx: &mut Self::Context) -> Self::Result {
        self.broadcast_txn = false;
    }
}

impl actix::Handler<SyncDone> for ChainNotifyHandlerActor {
    type Result = ();
    fn handle(&mut self, _done: SyncDone, _ctx: &mut Self::Context) -> Self::Result {
        self.broadcast_txn = true;
    }
}

impl actix::StreamHandler<NewHeadBlock> for ChainNotifyHandlerActor {
    fn handle(&mut self, item: NewHeadBlock, _ctx: &mut Self::Context) {
        if self.broadcast_txn {
            let NewHeadBlock(block_detail) = item;
            let block = block_detail.get_block();
            // notify header.
            self.notify_new_block(block);
            // notify events
            if let Err(e) = self.notify_events(block, self.store.clone()) {
                error!(target: "pubsub", "fail to notify events to client, err: {}", &e);
            }
        }
    }
}

impl ChainNotifyHandlerActor {
    pub fn notify_new_block(&self, block: &Block) {
        for subscriber in self.new_header_subscribers.read().values() {
            let thin_block = pubsub::ThinBlock::new(
                block.header().clone(),
                block
                    .transactions()
                    .iter()
                    .map(|t| t.crypto_hash())
                    .collect(),
            );
            notify::notify(
                subscriber,
                pubsub::Result::Block(Box::new(thin_block.clone())),
            );
        }
    }

    pub fn notify_events(&self, block: &Block, store: Arc<dyn Store>) -> Result<()> {
        // let block = store.get_block(block_id)?;
        // if block.is_none() {
        //     return Ok(());
        // }
        // let block = block.unwrap();

        let block_number = block.header().number();
        let block_id = block.id();
        let txn_info_ids = store.get_block_txn_info_ids(block_id)?;
        // in reverse order to do limit
        let mut all_events: Vec<ContractEvent> = vec![];
        for (_i, txn_info_id) in txn_info_ids.into_iter().enumerate().rev() {
            // get events directly by txn_info_id
            let mut events = store.get_contract_events(txn_info_id)?.unwrap_or_default();
            events.reverse();
            // .map(|e| Event::new(Some(block_id), None, Some(txn_hash), Some(i as u64), e));
            all_events.extend(events);
        }

        for (_id, (c, filter)) in self.subscribers.read().iter() {
            let filtered_events = all_events
                .iter()
                .filter(|e| filter.matching(block_number, *e))
                .take(filter.limit.unwrap_or(std::usize::MAX));
            let mut to_send_events = Vec::new();
            for evt in filtered_events {
                let e = Event::new(Some(block_id), Some(block_number), None, None, evt);
                to_send_events.push(pubsub::Result::Event(Box::new(e)));
            }
            to_send_events.reverse();
            notify::notify_many(c, to_send_events);
        }
        Ok(())
    }
}
