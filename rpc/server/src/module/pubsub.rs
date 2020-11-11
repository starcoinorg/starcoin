// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::channel::mpsc;
use futures::compat::Sink01CompatExt;
use futures::future::AbortHandle;
use futures::{compat::Future01CompatExt, StreamExt};
use jsonrpc_core::Result;
use jsonrpc_pubsub::typed::Subscriber;
use jsonrpc_pubsub::SubscriptionId;
use parking_lot::RwLock;
use starcoin_chain_notify::message::{Event, Notification, ThinBlock};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_api::types::pubsub::{MintBlock, ThinHeadBlock};
use starcoin_rpc_api::{errors, pubsub::StarcoinPubSub, types::pubsub};
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::filter::Filter;
use starcoin_types::system_events::MintBlockEvent;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::{atomic, Arc};

#[cfg(test)]
pub mod tests;

pub struct PubSubImpl {
    service: PubSubService,
}

impl PubSubImpl {
    pub fn new(s: PubSubService) -> Self {
        Self { service: s }
    }
}

impl StarcoinPubSub for PubSubImpl {
    type Metadata = Metadata;
    fn subscribe(
        &self,
        _meta: Metadata,
        subscriber: Subscriber<pubsub::Result>,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    ) {
        let error = match (kind, params) {
            (pubsub::Kind::NewHeads, None) => {
                self.service.add_new_header_subscription(subscriber);
                return;
            }
            (pubsub::Kind::NewHeads, _) => {
                errors::invalid_params("newHeads", "Expected no parameters.")
            }
            (pubsub::Kind::NewPendingTransactions, None) => {
                self.service.add_new_txn_subscription(subscriber);
                return;
            }
            (pubsub::Kind::NewPendingTransactions, _) => {
                errors::invalid_params("newPendingTransactions", "Expected no parameters.")
            }
            (pubsub::Kind::Events, Some(pubsub::Params::Events(filter))) => {
                match filter.try_into() {
                    Ok(f) => {
                        self.service.add_event_subscription(subscriber, f);
                        return;
                    }
                    Err(e) => e,
                }
            }
            (pubsub::Kind::Events, _) => {
                errors::invalid_params("events", "Expected a filter object.")
            }
            (pubsub::Kind::NewMintBlock, _) => {
                self.service.add_mint_block_subscription(subscriber);
                return;
            }
        };

        let _ = subscriber.reject(error);
    }

    fn unsubscribe(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
        self.service.unsubscribe(id)
    }
}

//TODO refactor this to ActorService
#[derive(Clone)]
pub struct PubSubService {
    subscriber_id: Arc<atomic::AtomicU64>,
    bus: ServiceRef<BusService>,
    subscribers: Arc<RwLock<HashMap<SubscriptionId, AbortHandle>>>,
    txpool: TxPoolService,
    spawner: actix_rt::Arbiter,
}

impl PubSubService {
    pub fn new(bus: ServiceRef<BusService>, txpool: TxPoolService) -> Self {
        let subscriber_id = Arc::new(atomic::AtomicU64::new(0));
        Self {
            spawner: actix_rt::Arbiter::new(),
            subscriber_id,
            bus,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            txpool,
        }
    }

    fn start_subscription<M, Handler>(
        &self,
        msg_channel: mpsc::UnboundedReceiver<M>,
        subscriber: Subscriber<pubsub::Result>,
        event_handler: Handler,
    ) where
        M: Send + 'static,
        Handler: EventHandler<M> + Send + 'static,
    {
        let subscriber_id = self.next_id();

        // TODO: should we use assgin_id_async?
        if let Ok(sink) = subscriber.assign_id(subscriber_id.clone()) {
            let subscribers = self.subscribers.clone();
            let subscribers_clone = subscribers.clone();
            let subscriber_id_clone = subscriber_id.clone();
            let sink = sink.sink_compat();
            let (f, abort_handle) = futures::future::abortable(async move {
                let forward = msg_channel
                    .flat_map(move |m| {
                        let r = event_handler.handle(m);
                        futures::stream::iter(
                            r.into_iter().map(Ok::<_, jsonrpc_pubsub::TransportError>),
                        )
                    })
                    .forward(sink)
                    .await;
                if let Err(e) = forward {
                    log::warn!(target: "rpc", "Unable to send notification: {}", e);
                    // if any error happen, we need to remove self from subscribers.
                    subscribers_clone.write().remove(&subscriber_id_clone);
                }
            });

            self.spawner.send(Box::pin(async move {
                let _ = f.await;
            }));
            subscribers.write().insert(subscriber_id, abort_handle);
        }
    }
    fn next_id(&self) -> SubscriptionId {
        let id = self.subscriber_id.fetch_add(1, atomic::Ordering::SeqCst);
        SubscriptionId::Number(id)
    }

    pub fn add_new_txn_subscription(&self, subscriber: Subscriber<pubsub::Result>) {
        let txn_events = self.txpool.subscribe_pending_txn();
        self.start_subscription(txn_events, subscriber, TxnEventHandler);
    }

    pub fn add_new_header_subscription(&self, subscriber: Subscriber<pubsub::Result>) {
        let myself = self.clone();
        self.spawner.send(Box::pin(async move {
            let channel = myself.bus.channel().await;
            match channel {
                Err(_e) => {
                    let _ = subscriber
                        .reject_async(jsonrpc_core::Error::internal_error())
                        .compat()
                        .await;
                }
                Ok(receiver) => {
                    myself.start_subscription(receiver, subscriber, NewHeadHandler);
                }
            }
        }));
    }

    pub fn add_mint_block_subscription(&self, subscriber: Subscriber<pubsub::Result>) {
        let myself = self.clone();
        self.spawner.send(Box::pin(async move {
            let channel = myself.bus.channel().await;
            match channel {
                Err(_e) => {
                    let _ = subscriber
                        .reject_async(jsonrpc_core::Error::internal_error())
                        .compat()
                        .await;
                }
                Ok(receiver) => {
                    myself.start_subscription(receiver, subscriber, NewMintBlockHandler);
                }
            }
        }));
    }

    pub fn add_event_subscription(&self, subscriber: Subscriber<pubsub::Result>, filter: Filter) {
        let myself = self.clone();
        self.spawner.send(Box::pin(async move {
            let channel = myself.bus.channel().await;
            match channel {
                Err(_) => {
                    let _ = subscriber
                        .reject_async(jsonrpc_core::Error::internal_error())
                        .compat()
                        .await;
                }
                Ok(receiver) => {
                    myself.start_subscription(
                        receiver,
                        subscriber,
                        ContractEventHandler { filter },
                    );
                }
            }
        }));
    }
    pub fn unsubscribe(&self, id: SubscriptionId) -> Result<bool> {
        match self.subscribers.write().remove(&id) {
            Some(handle) => {
                handle.abort();
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

trait EventHandler<M> {
    fn handle(&self, msg: M) -> Vec<jsonrpc_core::Result<pubsub::Result>>;
}

#[derive(Copy, Clone, Debug)]
pub struct TxnEventHandler;

impl EventHandler<Arc<Vec<HashValue>>> for TxnEventHandler {
    fn handle(&self, msg: Arc<Vec<HashValue>>) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::TransactionHash(msg.to_vec()))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewHeadHandler;

impl EventHandler<Notification<ThinBlock>> for NewHeadHandler {
    fn handle(&self, msg: Notification<ThinBlock>) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        let Notification(block) = msg;
        vec![Ok(pubsub::Result::Block(Box::new(ThinHeadBlock::new(
            block.header,
            block.body,
        ))))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewMintBlockHandler;

impl EventHandler<MintBlockEvent> for NewMintBlockHandler {
    fn handle(&self, msg: MintBlockEvent) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::MintBlock(Box::new(MintBlock {
            strategy: msg.strategy,
            minting_blob: msg.minting_blob,
            difficulty: msg.difficulty,
        })))]
    }
}

#[derive(Clone, Debug)]
pub struct ContractEventHandler {
    filter: Filter,
}

impl EventHandler<Notification<Arc<Vec<Event>>>> for ContractEventHandler {
    fn handle(
        &self,
        msg: Notification<Arc<Vec<Event>>>,
    ) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        let Notification(events) = msg;
        let filtered = events
            .as_ref()
            .iter()
            .filter(|e| self.filter.matching(e.block_number, &e.contract_event));
        let filtered_events: Vec<_> = match self.filter.limit {
            None => filtered.collect(),
            Some(l) => {
                let mut evts: Vec<_> = filtered.rev().take(l).collect();
                evts.reverse();
                evts
            }
        };

        filtered_events
            .into_iter()
            .map(|e| {
                pubsub::Event::new(
                    Some(e.block_hash),
                    Some(e.block_number),
                    Some(e.transaction_hash),
                    e.transaction_index,
                    &e.contract_event,
                )
            })
            .map(|e| Ok(pubsub::Result::Event(Box::new(e))))
            .collect()
    }
}
