// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod message;

use crate::message::{Event, Notification, ThinBlock};
use actix::Addr;
use anyhow::{format_err, Result};
use starcoin_bus::{Broadcast, BusActor};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{Storage, Store};
use starcoin_types::block::Block;
use starcoin_types::system_events::{NewHeadBlock, SyncBegin, SyncDone};
use std::sync::Arc;

/// ChainNotify watch `NewHeadBlock` message from bus,
/// and then reproduce `Notification<ThinBlock>` and `Notification<Arc<Vec<Event>>>` message to bus.
/// User can subscribe the two notification to watch onchain events.
pub struct ChainNotifyHandlerService {
    bus: Addr<BusActor>,
    store: Arc<dyn Store>,
    broadcast_txn: bool,
}

impl ChainNotifyHandlerService {
    pub fn new(bus: Addr<BusActor>, store: Arc<dyn Store>) -> Self {
        Self {
            bus,
            store,
            broadcast_txn: true,
        }
    }
}

impl ServiceFactory<Self> for ChainNotifyHandlerService {
    fn create(
        ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) -> Result<ChainNotifyHandlerService> {
        let bus = ctx.get_shared::<Addr<BusActor>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        Ok(Self::new(bus, storage))
    }
}

impl ActorService for ChainNotifyHandlerService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncBegin>();
        ctx.subscribe::<SyncDone>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncBegin>();
        ctx.unsubscribe::<SyncDone>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, SyncBegin> for ChainNotifyHandlerService {
    fn handle_event(
        &mut self,
        _msg: SyncBegin,
        _ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) {
        self.broadcast_txn = false;
    }
}

impl EventHandler<Self, SyncDone> for ChainNotifyHandlerService {
    fn handle_event(
        &mut self,
        _msg: SyncDone,
        _ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) {
        self.broadcast_txn = true;
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainNotifyHandlerService {
    fn handle_event(
        &mut self,
        item: NewHeadBlock,
        _ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) {
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

impl ChainNotifyHandlerService {
    pub fn notify_new_block(&self, block: &Block) {
        let thin_block = ThinBlock::new(
            block.header().clone(),
            block
                .transactions()
                .iter()
                .map(|t| t.crypto_hash())
                .collect(),
        );
        self.bus.do_send(Broadcast {
            msg: Notification(thin_block),
        });
    }

    pub fn notify_events(&self, block: &Block, store: Arc<dyn Store>) -> Result<()> {
        let block_number = block.header().number();
        let block_id = block.id();
        let txn_info_ids = store.get_block_txn_info_ids(block_id)?;
        let mut all_events: Vec<Event> = vec![];
        for (_i, txn_info_id) in txn_info_ids.into_iter().enumerate().rev() {
            let txn_hash = store
                .get_transaction_info(txn_info_id)?
                .map(|info| info.transaction_hash())
                .ok_or_else(|| format_err!("cannot find txn info by it's id {}", &txn_info_id))?;
            // get events directly by txn_info_id
            let events = store.get_contract_events(txn_info_id)?.unwrap_or_default();
            all_events.extend(
                events
                    .into_iter()
                    .map(|evt| Event::new(block_id, block_number, txn_hash, None, evt)),
            );
        }
        let events = Arc::new(all_events);
        self.bus.do_send(Broadcast {
            msg: Notification(events),
        });
        Ok(())
    }
}
