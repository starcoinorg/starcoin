// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod message;

use crate::message::{Event, Notification, ThinBlock};
use anyhow::{format_err, Result};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{Storage, Store};
use starcoin_types::block::Block;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{NewHeadBlock, SyncStatusChangeEvent};
use std::sync::Arc;

/// ChainNotify watch `NewHeadBlock` message from bus,
/// and then reproduce `Notification<ThinBlock>` and `Notification<Arc<Vec<Event>>>` message to bus.
/// User can subscribe the two notification to watch onchain events.
pub struct ChainNotifyHandlerService {
    store: Arc<dyn Store>,
    sync_status: Option<SyncStatus>,
}

impl ChainNotifyHandlerService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self {
            store,
            sync_status: None,
        }
    }
}

impl ServiceFactory<Self> for ChainNotifyHandlerService {
    fn create(
        ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) -> Result<ChainNotifyHandlerService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        Ok(Self::new(storage))
    }
}

impl ActorService for ChainNotifyHandlerService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for ChainNotifyHandlerService {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainNotifyHandlerService {
    fn handle_event(
        &mut self,
        item: NewHeadBlock,
        ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) {
        if let Some(sync_status) = self.sync_status.as_ref() {
            if sync_status.is_nearly_synced() {
                let NewHeadBlock(block_detail) = item;
                let block = block_detail.get_block();
                // notify header.
                self.notify_new_block(block, ctx);

                // notify events
                if let Err(e) = self.notify_events(block, self.store.clone(), ctx) {
                    error!(target: "pubsub", "fail to notify events to client, err: {}", &e);
                }
            }
        }
    }
}

impl ChainNotifyHandlerService {
    pub fn notify_new_block(&self, block: &Block, ctx: &mut ServiceContext<Self>) {
        let thin_block = ThinBlock::new(
            block.header().clone(),
            block
                .transactions()
                .iter()
                .map(|t| t.crypto_hash())
                .collect(),
        );
        ctx.broadcast(Notification(thin_block));
    }

    pub fn notify_events(
        &self,
        block: &Block,
        store: Arc<dyn Store>,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
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
        ctx.broadcast(Notification(events));
        Ok(())
    }
}
