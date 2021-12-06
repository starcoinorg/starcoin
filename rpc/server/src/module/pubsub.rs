// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use anyhow::Result;
use futures::channel::mpsc;
use futures::future::AbortHandle;
use futures::StreamExt;
use jsonrpc_pubsub::typed::Subscriber;
use jsonrpc_pubsub::SubscriptionId;
use parking_lot::RwLock;
use starcoin_abi_decoder::decode_move_value;
use starcoin_abi_resolver::ABIResolver;
use starcoin_chain_notify::message::{ContractEventNotification, Notification, ThinBlock};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_miner::{MinerService, UpdateSubscriberNumRequest};
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_api::types::{BlockView, TransactionEventResponse, TransactionEventView};
use starcoin_rpc_api::{errors, pubsub::StarcoinPubSub, types::pubsub};
use starcoin_service_registry::{
    ActorService, EventHandler as ActorEventHandler, ServiceContext, ServiceFactory,
    ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::filter::Filter;
use starcoin_types::system_events::MintBlockEvent;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::mpsc::TrySendError;
use std::sync::{atomic, Arc};

#[cfg(test)]
pub mod tests;

pub struct PubSubImpl {
    service: ServiceRef<PubSubService>,
}

impl PubSubImpl {
    pub fn new(s: ServiceRef<PubSubService>) -> Self {
        Self { service: s }
    }
}

fn map_send_err<T>(err: &TrySendError<T>) -> jsonrpc_core::Error {
    match err {
        TrySendError::Full(_) => jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::InternalError,
            message: "pubsub service is overloaded".to_string(),
            data: None,
        },
        TrySendError::Disconnected(_) => jsonrpc_core::Error {
            code: jsonrpc_core::ErrorCode::InternalError,
            message: "pubsub service is down".to_string(),
            data: None,
        },
    }
}

impl PubSubImpl {
    fn inner_subscribe(
        &self,
        _meta: Metadata,
        subscriber: Subscriber<pubsub::Result>,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    ) -> Result<(), (Subscriber<pubsub::Result>, jsonrpc_core::Error)> {
        match (kind, params) {
            (pubsub::Kind::NewHeads, None) => self
                .service
                .try_send(SubscribeNewHeads(subscriber))
                .map_err(|e| {
                    let msg = map_send_err(&e);
                    (
                        match e {
                            TrySendError::Disconnected(t) => t.0,
                            TrySendError::Full(t) => t.0,
                        },
                        msg,
                    )
                }),
            (pubsub::Kind::NewHeads, _) => Err((
                subscriber,
                errors::invalid_params("newHeads", "Expected no parameters."),
            )),
            (pubsub::Kind::NewPendingTransactions, None) => self
                .service
                .try_send(SubscribeNewPendingTxns { subscriber })
                .map_err(|e| {
                    let msg = map_send_err(&e);
                    (
                        match e {
                            TrySendError::Disconnected(t) => t.subscriber,
                            TrySendError::Full(t) => t.subscriber,
                        },
                        msg,
                    )
                }),
            (pubsub::Kind::NewPendingTransactions, _) => Err((
                subscriber,
                errors::invalid_params("newPendingTransactions", "Expected no parameters."),
            )),
            (pubsub::Kind::Events, Some(pubsub::Params::Events(param))) => {
                match param.filter.try_into() {
                    Ok(f) => self
                        .service
                        .try_send(SubscribeEvents {
                            subscriber,
                            filter: f,
                            decode: param.decode,
                        })
                        .map_err(|e| {
                            let msg = map_send_err(&e);
                            (
                                match e {
                                    TrySendError::Disconnected(t) => t.subscriber,
                                    TrySendError::Full(t) => t.subscriber,
                                },
                                msg,
                            )
                        }),
                    Err(e) => Err((subscriber, e)),
                }
            }
            (pubsub::Kind::Events, _) => Err((
                subscriber,
                errors::invalid_params("events", "Expected a filter object."),
            )),
            (pubsub::Kind::NewMintBlock, _) => self
                .service
                .try_send(SubscribeMintBlock(subscriber))
                .map_err(|e| {
                    let msg = map_send_err(&e);
                    (
                        match e {
                            TrySendError::Disconnected(t) => t.0,
                            TrySendError::Full(t) => t.0,
                        },
                        msg,
                    )
                }),
        }
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
        if let Err((subscriber, error)) = self.inner_subscribe(_meta, subscriber, kind, params) {
            let _ = subscriber.reject(error);
        }
    }

    fn unsubscribe(
        &self,
        _: Option<Self::Metadata>,
        id: SubscriptionId,
    ) -> jsonrpc_core::Result<bool> {
        match self.service.try_send(Unsubscribe(id)) {
            Ok(()) => Ok(true),
            Err(TrySendError::Full(_)) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::InternalError,
                message: "pubsub service is overloaded".to_string(),
                data: None,
            }),
            Err(TrySendError::Disconnected(_)) => Err(jsonrpc_core::Error {
                code: jsonrpc_core::ErrorCode::InternalError,
                message: "pubsub service is down".to_string(),
                data: None,
            }),
        }
    }
}

pub struct PubSubServiceFactory;

impl ServiceFactory<PubSubService> for PubSubServiceFactory {
    fn create(ctx: &mut ServiceContext<PubSubService>) -> Result<PubSubService> {
        let miner_service = ctx.service_ref::<MinerService>()?.clone();
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        Ok(PubSubService::new(
            ctx.get_shared::<TxPoolService>()?,
            miner_service,
            storage,
        ))
    }
}

pub struct PubSubService {
    subscriber_id: Arc<atomic::AtomicU64>,
    txpool: TxPoolService,
    miner_service: ServiceRef<MinerService>,
    storage: Arc<Storage>,
    new_header_subscribers: HashMap<SubscriptionId, mpsc::UnboundedSender<NewHeadNotification>>,
    new_event_subscribers:
        HashMap<SubscriptionId, mpsc::UnboundedSender<ContractEventNotification>>,
    mint_block_subscribers: HashMap<SubscriptionId, mpsc::UnboundedSender<MintBlockEvent>>,
    new_pending_txn_tasks: Arc<RwLock<HashMap<SubscriptionId, AbortHandle>>>,
}

impl PubSubService {
    fn new(
        txpool: TxPoolService,
        miner_service: ServiceRef<MinerService>,
        storage: Arc<Storage>,
    ) -> Self {
        let subscriber_id = Arc::new(atomic::AtomicU64::new(0));
        Self {
            subscriber_id,
            txpool,
            miner_service,
            storage,
            new_event_subscribers: Default::default(),
            new_header_subscribers: Default::default(),
            mint_block_subscribers: Default::default(),
            new_pending_txn_tasks: Arc::new(RwLock::new(HashMap::default())),
        }
    }
    fn next_id(&self) -> SubscriptionId {
        let id = self.subscriber_id.fetch_add(1, atomic::Ordering::SeqCst);
        SubscriptionId::Number(id)
    }
}

type NewHeadNotification = Notification<ThinBlock>;
// type NewTxns = Arc<[HashValue]>;

impl ActorService for PubSubService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<NewHeadNotification>();
        ctx.subscribe::<ContractEventNotification>();
        ctx.subscribe::<MintBlockEvent>();

        Ok(())
    }
}

impl ActorEventHandler<Self, NewHeadNotification> for PubSubService {
    fn handle_event(&mut self, msg: NewHeadNotification, _ctx: &mut ServiceContext<PubSubService>) {
        send_to_all(&mut self.new_header_subscribers, msg);
    }
}

impl ActorEventHandler<Self, ContractEventNotification> for PubSubService {
    fn handle_event(
        &mut self,
        msg: ContractEventNotification,
        _ctx: &mut ServiceContext<PubSubService>,
    ) {
        send_to_all(&mut self.new_event_subscribers, msg);
    }
}

impl ActorEventHandler<Self, MintBlockEvent> for PubSubService {
    fn handle_event(&mut self, msg: MintBlockEvent, _ctx: &mut ServiceContext<PubSubService>) {
        send_to_all(&mut self.mint_block_subscribers, msg);
    }
}

#[derive(Debug)]
struct SubscribeNewHeads(Subscriber<pubsub::Result>);

impl ServiceRequest for SubscribeNewHeads {
    type Response = ();
}

impl ServiceHandler<Self, SubscribeNewHeads> for PubSubService {
    fn handle(&mut self, msg: SubscribeNewHeads, ctx: &mut ServiceContext<Self>) {
        let SubscribeNewHeads(sink) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let subscriber_id = self.next_id();
        self.new_header_subscribers
            .insert(subscriber_id.clone(), sender);
        ctx.spawn(run_subscription(
            receiver,
            subscriber_id,
            sink,
            NewHeadHandler,
        ));
    }
}

#[derive(Debug)]
struct SubscribeMintBlock(Subscriber<pubsub::Result>);

impl ServiceRequest for SubscribeMintBlock {
    type Response = ();
}

impl ServiceHandler<Self, SubscribeMintBlock> for PubSubService {
    fn handle(&mut self, msg: SubscribeMintBlock, ctx: &mut ServiceContext<Self>) {
        let SubscribeMintBlock(subscriber) = msg;
        let (sender, receiver) = mpsc::unbounded();
        let subscriber_id = self.next_id();
        self.mint_block_subscribers
            .insert(subscriber_id.clone(), sender.clone());
        let miner_service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        ctx.spawn(run_subscription(
            receiver,
            subscriber_id,
            subscriber,
            NewMintBlockHandler,
        ));
        ctx.spawn(async move {
            match miner_service
                .send(UpdateSubscriberNumRequest {
                    number: Some(subscribers_num),
                })
                .await
            {
                Ok(Some(event)) => {
                    if let Err(err) = sender.unbounded_send(event) {
                        error!("[pubsub] Failed to send MintBlockEvent: {}", err);
                    }
                }
                Ok(None) => {}
                _ => error!("[pubsub] Failed to send NewMinerClientRequest to miner service"),
            };
        });
    }
}

#[derive(Debug)]
struct SubscribeEvents {
    subscriber: Subscriber<pubsub::Result>,
    filter: Filter,
    decode: bool,
}

impl ServiceRequest for SubscribeEvents {
    type Response = ();
}

impl ServiceHandler<Self, SubscribeEvents> for PubSubService {
    fn handle(&mut self, msg: SubscribeEvents, ctx: &mut ServiceContext<Self>) {
        let SubscribeEvents {
            subscriber,
            filter,
            decode,
        } = msg;
        let (sender, receiver) = mpsc::unbounded();
        let subscriber_id = self.next_id();
        self.new_event_subscribers
            .insert(subscriber_id.clone(), sender);
        ctx.spawn(run_subscription(
            receiver,
            subscriber_id,
            subscriber,
            ContractEventHandler {
                storage: self.storage.clone(),
                filter,
                decode,
            },
        ));
    }
}

#[derive(Debug)]
struct SubscribeNewPendingTxns {
    subscriber: Subscriber<pubsub::Result>,
}

impl ServiceRequest for SubscribeNewPendingTxns {
    type Response = ();
}

impl ServiceHandler<Self, SubscribeNewPendingTxns> for PubSubService {
    fn handle(&mut self, msg: SubscribeNewPendingTxns, ctx: &mut ServiceContext<Self>) {
        let SubscribeNewPendingTxns { subscriber } = msg;
        let subscriber_id = self.next_id();
        let tasks = self.new_pending_txn_tasks.clone();
        let subscriber_id_clone = subscriber_id.clone();
        let receiver = self.txpool.subscribe_pending_txn();
        let (f, abort_handle) = futures::future::abortable(async move {
            run_subscription(
                receiver,
                subscriber_id_clone.clone(),
                subscriber,
                TxnEventHandler,
            )
            .await;
            // remove self from task list.
            tasks.write().remove(&subscriber_id_clone);
        });

        ctx.spawn(async move {
            let _ = f.await;
        });

        self.new_pending_txn_tasks
            .write()
            .insert(subscriber_id, abort_handle);
    }
}

#[derive(Debug)]
struct Unsubscribe(SubscriptionId);

impl ServiceRequest for Unsubscribe {
    type Response = ();
}

impl ServiceHandler<Self, Unsubscribe> for PubSubService {
    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut ServiceContext<Self>) {
        self.new_header_subscribers.remove(&msg.0);
        self.new_event_subscribers.remove(&msg.0);
        self.mint_block_subscribers.remove(&msg.0);
        self.miner_service.do_send(UpdateSubscriberNumRequest {
            number: Some(self.mint_block_subscribers.len() as u32),
        });
        if let Some(h) = self.new_pending_txn_tasks.write().remove(&msg.0) {
            h.abort();
        }
    }
}

fn send_to_all<T: Clone>(
    subscriptions: &mut HashMap<SubscriptionId, mpsc::UnboundedSender<T>>,
    msg: T,
) {
    let mut remove_outdated = vec![];

    for (id, ch) in subscriptions.iter() {
        if let Err(err) = ch.unbounded_send(msg.clone()) {
            if err.is_disconnected() {
                remove_outdated.push(id.clone());
            } else if err.is_full() {
                log::error!(
                    "subscription {:?} fail to new messages, channel is full",
                    id
                );
            }
        }
    }

    // drop outdated subscribers.
    for id in remove_outdated {
        subscriptions.remove(&id);
    }
}

async fn run_subscription<M, Handler>(
    msg_channel: mpsc::UnboundedReceiver<M>,
    subscriber_id: SubscriptionId,
    subscriber: Subscriber<pubsub::Result>,
    event_handler: Handler,
) where
    M: Send + 'static,
    Handler: EventHandler<M> + Send + 'static,
{
    // TODO: should we use assgin_id_async?
    if let Ok(sink) = subscriber.assign_id(subscriber_id.clone()) {
        let forward = msg_channel
            .flat_map(move |m| {
                let r = event_handler.handle(m);
                futures::stream::iter(r.into_iter().map(Ok::<_, jsonrpc_pubsub::TransportError>))
            })
            .forward(sink)
            .await;
        if let Err(e) = forward {
            log::warn!(target: "rpc", "Unable to send notification: {}", e);
        }
    }
}

trait EventHandler<M> {
    fn handle(&self, msg: M) -> Vec<jsonrpc_core::Result<pubsub::Result>>;
}

#[derive(Copy, Clone, Debug)]
pub struct TxnEventHandler;

impl EventHandler<Arc<[HashValue]>> for TxnEventHandler {
    fn handle(&self, msg: Arc<[HashValue]>) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::TransactionHash(msg.to_vec()))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewHeadHandler;

impl EventHandler<Notification<ThinBlock>> for NewHeadHandler {
    fn handle(&self, msg: Notification<ThinBlock>) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        let Notification(block) = msg;
        vec![Ok(pubsub::Result::Block(Box::new(BlockView {
            header: block.header.into(),
            body: block.body.into(),
            uncles: vec![],
        })))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewMintBlockHandler;

impl EventHandler<MintBlockEvent> for NewMintBlockHandler {
    fn handle(&self, msg: MintBlockEvent) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::MintBlock(Box::new(msg)))]
    }
}

#[derive(Clone, Debug)]
pub struct ContractEventHandler {
    filter: Filter,
    decode: bool,
    storage: Arc<Storage>,
}

impl EventHandler<ContractEventNotification> for ContractEventHandler {
    fn handle(&self, msg: ContractEventNotification) -> Vec<jsonrpc_core::Result<pubsub::Result>> {
        let Notification((state_root, events)) = msg;
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

        let state = if self.decode {
            Some(ChainStateDB::new(self.storage.clone(), Some(state_root)))
        } else {
            None
        };
        filtered_events
            .into_iter()
            .map(|e| {
                let decoded_data = match &state {
                    Some(s) => {
                        let abi =
                            ABIResolver::new(s).resolve_type_tag(e.contract_event.type_tag())?;
                        Some(decode_move_value(&abi, e.contract_event.event_data())?)
                    }
                    None => None,
                };
                Ok(TransactionEventResponse {
                    event: TransactionEventView::new(
                        Some(e.block_hash),
                        Some(e.block_number),
                        Some(e.transaction_hash),
                        e.transaction_index,
                        e.transaction_global_index,
                        e.event_index,
                        &e.contract_event,
                    ),
                    decode_event_data: decoded_data,
                })
            })
            .map(|e| {
                e.map(|d| pubsub::Result::Event(Box::new(d)))
                    .map_err(map_err)
            })
            .collect()
    }
}
