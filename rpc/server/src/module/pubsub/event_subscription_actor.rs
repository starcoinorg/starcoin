use super::notify;
use super::pubsub;

use super::EventSubscribers;
use actix;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::event::Event;
use starcoin_storage::Store;
use starcoin_types::contract_event::ContractEvent;
use std::sync::Arc;
pub struct EventSubscriptionActor {
    subscribers: EventSubscribers,
    store: Arc<dyn Store>,
}
impl EventSubscriptionActor {
    pub fn new(subscribers: EventSubscribers, store: Arc<dyn Store>) -> Self {
        Self { subscribers, store }
    }
}

impl actix::Actor for EventSubscriptionActor {
    type Context = actix::Context<Self>;
}

impl EventSubscriptionActor {
    pub fn logs(&self, block_id: HashValue, store: Arc<dyn Store>) -> Result<()> {
        let block = store.get_block(block_id)?;
        if block.is_none() {
            return Ok(());
        }
        let block = block.unwrap();

        let block_number = block.header().number();
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
                .filter(|e| {
                    filter.matching(*e)
                        && filter.from_block <= block_number
                        && block_number <= filter.to_block
                })
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
