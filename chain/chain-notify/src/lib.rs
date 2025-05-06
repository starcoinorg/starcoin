// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod message;

use crate::message::{ContractEventNotification, Event, Event2, Notification, ThinBlock};
use anyhow::{format_err, Result};
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{Storage, Store};
use starcoin_types::block::Block;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_vm2_storage::{Storage as Storage2, Store as Store2};
use std::sync::Arc;

/// ChainNotify watch `NewHeadBlock` message from bus,
/// and then reproduce `Notification<ThinBlock>` and `Notification<Arc<[Event]>>` message to bus.
/// User can subscribe the two notification to watch onchain events.
pub struct ChainNotifyHandlerService {
    store: Arc<dyn Store>,
    store2: Arc<dyn Store2>,
}

impl ChainNotifyHandlerService {
    pub fn new(store: Arc<dyn Store>, store2: Arc<dyn Store2>) -> Self {
        Self { store, store2 }
    }
}

impl ServiceFactory<Self> for ChainNotifyHandlerService {
    fn create(
        ctx: &mut ServiceContext<ChainNotifyHandlerService>,
    ) -> Result<ChainNotifyHandlerService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
        Ok(Self::new(storage, storage2))
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
        if let Err(e) = self.notify_events(block, self.store.clone(), self.store2.clone(), ctx) {
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
        store2: Arc<dyn Store2>,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        let block_number = block.header().number();
        let block_id = block.id();
        let multi_state_root = store.get_vm_multi_state(block_id)?;
        let txn_info_ids = store.get_block_txn_info_ids(block_id)?;
        let mut all_events: Vec<Event> = vec![];
        let mut all_events2 = vec![];
        for txn_info_id in txn_info_ids.into_iter().rev() {
            let txn_info = store
                .get_transaction_info(txn_info_id)?
                .ok_or_else(|| format_err!("cannot find txn info by it's id {}", &txn_info_id))?;
            // get events directly by txn_info_id
            let (in_vm1, events) = store
                .get_contract_events(txn_info_id)?
                .map(|e| (true, e))
                .unwrap_or((false, vec![]));
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
            if !in_vm1 {
                let events = store2.get_contract_events(txn_info_id)?.unwrap_or_default();
                all_events2.extend(events.into_iter().enumerate().map(|(idx, evt)| {
                    Event2::new(
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
        }
        let (state_root1, state_root2) = multi_state_root
            .map(|s| (s.state_root1(), Some(s.state_root2())))
            .unwrap_or((block.header.state_root(), None));
        let events_notification: ContractEventNotification = Notification((
            state_root1,
            all_events.into(),
            state_root2,
            all_events2.into(),
        ));
        ctx.broadcast(events_notification);
        Ok(())
    }
}
