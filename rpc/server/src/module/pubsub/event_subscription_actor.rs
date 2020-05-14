use super::notify;
use super::pubsub;

use super::EventSubscribers;
use super::NewHeaderSubscribers;
use actix;
use actix::{ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, WrapFuture};
use anyhow::Result;
use starcoin_bus::{Bus, BusActor};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::event::Event;
use starcoin_storage::Store;
use starcoin_types::block::Block;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::system_events::SystemEvents;
use std::sync::Arc;

pub struct ChainNotifyHandlerActor {
    subscribers: EventSubscribers,
    new_header_subscribers: NewHeaderSubscribers,
    bus: actix::Addr<BusActor>,
    store: Arc<dyn Store>,
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
        }
    }
}

impl actix::Actor for ChainNotifyHandlerActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.bus
            .clone()
            .channel::<SystemEvents>()
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
impl actix::StreamHandler<SystemEvents> for ChainNotifyHandlerActor {
    fn handle(&mut self, item: SystemEvents, _ctx: &mut Self::Context) {
        if let SystemEvents::NewHeadBlock(block_detail) = item {
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
        let txns = store.get_block_transactions(block_id)?;
        // in reverse order to do limit
        let mut all_events: Vec<ContractEvent> = vec![];
        for (_i, txn_hash) in txns.into_iter().enumerate().rev() {
            let txn_info = store.get_transaction_info(txn_hash)?;
            if txn_info.is_none() {
                continue;
            }
            let txn_info = txn_info.unwrap();
            let events = txn_info.events();
            let events = events.iter().rev().cloned();
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
