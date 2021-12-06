// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod message;

use crate::message::{ContractEventNotification, Event, Notification, ThinBlock};
use anyhow::{format_err, Result};
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{Storage, Store};
use starcoin_types::block::Block;
use starcoin_types::system_events::NewHeadBlock;
use std::sync::Arc;

/// ChainNotify watch `NewHeadBlock` message from bus,
/// and then reproduce `Notification<ThinBlock>` and `Notification<Arc<[Event]>>` message to bus.
/// User can subscribe the two notification to watch onchain events.
pub struct ChainNotifyHandlerService {
    store: Arc<dyn Store>,
}

impl ChainNotifyHandlerService {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
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
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainNotifyHandlerService {
    fn handle_event(
        &mut self,
        item: NewHeadBlock,
        ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) {
        let NewHeadBlock(block_detail) = item;
        let block = block_detail.block();
        // notify header.
        self.notify_new_block(block, ctx);

        // notify events
        if let Err(e) = self.notify_events(block, self.store.clone(), ctx) {
            error!(target: "pubsub", "fail to notify events to client, err: {}", &e);
        }
    }
}

impl ChainNotifyHandlerService {
    pub fn notify_new_block(&self, block: &Block, ctx: &mut ServiceContext<Self>) {
        let thin_block = ThinBlock::new(
            block.header().clone(),
            block.transactions().iter().map(|t| t.id()).collect(),
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
        for txn_info_id in txn_info_ids.into_iter().rev() {
            let txn_info = store
                .get_transaction_info(txn_info_id)?
                .ok_or_else(|| format_err!("cannot find txn info by it's id {}", &txn_info_id))?;
            // get events directly by txn_info_id
            let events = store.get_contract_events(txn_info_id)?.unwrap_or_default();
            all_events.extend(events.into_iter().enumerate().map(|(idx, evt)| {
                Event::new(
                    block_id,
                    block_number,
                    txn_info.transaction_hash(),
                    Some(txn_info.transaction_index),
                    Some(txn_info.transaction_global_index),
                    Some(idx as u32),
                    evt,
                )
            }));
        }
        let events_notification: ContractEventNotification =
            Notification((block.header.state_root(), all_events.into()));
        ctx.broadcast(events_notification);
        Ok(())
    }
}
