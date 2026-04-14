// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::channel::mpsc;
use futures::future::{self, AbortHandle, Either, FutureExt};
use futures::stream::BoxStream;
use futures::StreamExt;
use jsonrpsee::core::{
    async_trait, to_json_raw_value, RegisterMethodError, SubscriptionError, SubscriptionResult,
};
use jsonrpsee::server::{PendingSubscriptionSink, SubscriptionMessage};
use jsonrpsee::types::error::{INTERNAL_ERROR_CODE, INVALID_PARAMS_CODE};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::{Methods, RpcModule};
use parking_lot::RwLock;
use starcoin_abi_decoder::decode_move_value;
use starcoin_abi_resolver::ABIResolver;
use starcoin_chain_notify::message::{ContractEventNotification, Notification, ThinBlock};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_miner::{MinerService, UpdateSubscriberNumRequest};
use starcoin_rpc_api::pubsub::StarcoinPubSubApiServer;
use starcoin_rpc_api::types::pubsub::{EventParams, EventParamsV2, Params};
use starcoin_rpc_api::types::{BlockView, TransactionEventResponse, TransactionEventView};
use starcoin_rpc_api::{errors, types::pubsub};
use starcoin_service_registry::{
    ActorService, EventHandler as ActorEventHandler, ServiceContext, ServiceFactory,
    ServiceHandler, ServiceRef, ServiceRequest,
};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Storage;
use starcoin_storage::Storage2;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::contract_event::StcContractEvent;
use starcoin_types::filter::Filter;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_vm2_abi_decoder::decode_move_value as decode_move_value2;
use starcoin_vm2_abi_resolver::ABIResolver as ABIResolver2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_types::view::{
    TransactionEventResponse as TransactionEventResponse2,
    TransactionEventView as TransactionEventView2,
};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::Arc;

type LocalSubscriptionId = u64;
type NewHeadNotification = Notification<ThinBlock>;

pub fn pubsub_methods(api: PubSubImpl) -> std::result::Result<Methods, RegisterMethodError> {
    let mut module = StarcoinPubSubApiServer::into_rpc(api.clone());
    register_legacy_pubsub_methods(&mut module)?;
    Ok(module.into())
}

fn register_legacy_pubsub_methods(
    module: &mut RpcModule<PubSubImpl>,
) -> std::result::Result<(), RegisterMethodError> {
    module.register_subscription(
        "starcoin_subscribe",
        "starcoin_subscription",
        "starcoin_unsubscribe",
        |params, pending, api, _| async move {
            let parsed = parse_subscribe_params(&params);
            let (kind, sub_params) = match parsed {
                Ok(input) => input,
                Err(err) => {
                    pending.reject(err).await;
                    return Ok(());
                }
            };

            let local_sub = match api.subscribe(kind, sub_params).await {
                Ok(sub) => sub,
                Err(err) => {
                    pending.reject(map_anyhow_err(err)).await;
                    return Ok(());
                }
            };

            forward_subscription_stream(api.as_ref(), pending, local_sub, |item| {
                serialize_subscription_item(&item)
            })
            .await;
            Ok(())
        },
    )?;
    Ok(())
}

fn serialize_subscription_item<T: serde::Serialize>(
    item: &T,
) -> Option<Box<serde_json::value::RawValue>> {
    match to_json_raw_value(item) {
        Ok(raw) => Some(raw),
        Err(err) => {
            warn!("failed to serialize pubsub event: {}", err);
            None
        }
    }
}

fn map_anyhow_subscription_err(err: anyhow::Error) -> SubscriptionError {
    err.to_string().into()
}

async fn forward_subscription_stream<F>(
    api: &PubSubImpl,
    pending: PendingSubscriptionSink,
    mut local_sub: LocalSubscription,
    mut map_item: F,
) where
    F: FnMut(pubsub::Result) -> Option<Box<serde_json::value::RawValue>>,
{
    let sink = match pending.accept().await {
        Ok(sink) => sink,
        Err(_) => {
            let _ = api.unsubscribe(local_sub.id).await;
            return;
        }
    };

    loop {
        let next_item = local_sub.stream.next().fuse();
        let closed = sink.closed().fuse();
        futures::pin_mut!(next_item, closed);

        match future::select(next_item, closed).await {
            Either::Left((maybe_item, _)) => match maybe_item {
                Some(Ok(item)) => {
                    if let Some(raw) = map_item(item) {
                        if sink.send(SubscriptionMessage::from(raw)).await.is_err() {
                            break;
                        }
                    }
                }
                Some(Err(err)) => {
                    warn!("failed to handle pubsub event: {}", err);
                }
                None => break,
            },
            Either::Right((_, _)) => break,
        }
    }

    let _ = api.unsubscribe(local_sub.id).await;
}

fn parse_subscribe_params(
    params: &jsonrpsee::types::Params<'_>,
) -> std::result::Result<(pubsub::Kind, Option<pubsub::Params>), ErrorObjectOwned> {
    let raw = params
        .parse::<serde_json::Value>()
        .map_err(|_| invalid_params_err("Invalid starcoin_subscribe params"))?;
    parse_subscribe_params_value(raw)
}

fn parse_subscribe_params_value(
    raw: serde_json::Value,
) -> std::result::Result<(pubsub::Kind, Option<pubsub::Params>), ErrorObjectOwned> {
    let parse_kind =
        |value: serde_json::Value| -> std::result::Result<pubsub::Kind, ErrorObjectOwned> {
            if let Ok(kind) = serde_json::from_value::<pubsub::Kind>(value.clone()) {
                return Ok(kind);
            }
            if let Ok(kinds) = serde_json::from_value::<Vec<pubsub::Kind>>(value) {
                if let [kind] = kinds.as_slice() {
                    return Ok(kind.clone());
                }
            }
            Err(invalid_params_err("Invalid starcoin_subscribe params"))
        };

    match raw {
        serde_json::Value::Array(mut args) => match args.len() {
            1 => {
                let kind = parse_kind(args.remove(0))?;
                Ok((kind, None))
            }
            2 => {
                let kind = parse_kind(args.remove(0))?;
                let raw_params = args.remove(0);
                let sub_params = if raw_params.is_null() {
                    None
                } else {
                    Some(
                        serde_json::from_value::<pubsub::Params>(raw_params)
                            .map_err(|_| invalid_params_err("Invalid starcoin_subscribe params"))?,
                    )
                };
                Ok((kind, sub_params))
            }
            _ => Err(invalid_params_err("Invalid starcoin_subscribe params")),
        },
        _ => Err(invalid_params_err("Invalid starcoin_subscribe params")),
    }
}

fn invalid_params_err(msg: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INVALID_PARAMS_CODE, msg.into(), None::<()>)
}

fn internal_err(msg: impl Into<String>) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(INTERNAL_ERROR_CODE, msg.into(), None::<()>)
}

fn map_anyhow_err(err: anyhow::Error) -> ErrorObjectOwned {
    let msg = err.to_string();
    if msg.starts_with("Couldn't parse parameters:") {
        invalid_params_err(msg)
    } else {
        internal_err(msg)
    }
}

struct LocalSubscription {
    id: LocalSubscriptionId,
    stream: BoxStream<'static, anyhow::Result<pubsub::Result>>,
}

#[derive(Clone)]
pub struct PubSubImpl {
    service: ServiceRef<PubSubService>,
}

impl PubSubImpl {
    pub fn new(service: ServiceRef<PubSubService>) -> Self {
        Self { service }
    }

    async fn subscribe(
        &self,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    ) -> anyhow::Result<LocalSubscription> {
        self.service.send(Subscribe { kind, params }).await?
    }

    async fn unsubscribe(&self, id: LocalSubscriptionId) -> anyhow::Result<bool> {
        self.service.send(Unsubscribe(id)).await
    }
}

#[async_trait]
impl StarcoinPubSubApiServer for PubSubImpl {
    async fn subscribe_new_heads(&self, pending: PendingSubscriptionSink) -> SubscriptionResult {
        let local_sub = self
            .subscribe(pubsub::Kind::NewHeads, None)
            .await
            .map_err(map_anyhow_subscription_err)?;
        forward_subscription_stream(self, pending, local_sub, |item| match item {
            pubsub::Result::Block(block) => serialize_subscription_item(&*block),
            other => {
                warn!(
                    "unexpected pubsub payload for starcoin_subscribeNewHeads: {:?}",
                    other
                );
                None
            }
        })
        .await;
        Ok(())
    }

    async fn subscribe_events(
        &self,
        pending: PendingSubscriptionSink,
        filter: pubsub::EventFilter,
        decode: bool,
    ) -> SubscriptionResult {
        let local_sub = self
            .subscribe(
                pubsub::Kind::Events,
                Some(Params::Events(EventParams { filter, decode })),
            )
            .await
            .map_err(map_anyhow_subscription_err)?;
        forward_subscription_stream(self, pending, local_sub, |item| match item {
            pubsub::Result::Event(event) => serialize_subscription_item(&event.event),
            other => {
                warn!(
                    "unexpected pubsub payload for starcoin_subscribeEvents: {:?}",
                    other
                );
                None
            }
        })
        .await;
        Ok(())
    }

    async fn subscribe_events_v2(
        &self,
        pending: PendingSubscriptionSink,
        filter: pubsub::EventFilterV2,
        decode: bool,
    ) -> SubscriptionResult {
        let local_sub = self
            .subscribe(
                pubsub::Kind::Events,
                Some(Params::EventsV2(EventParamsV2::new(filter, decode))),
            )
            .await
            .map_err(map_anyhow_subscription_err)?;
        forward_subscription_stream(self, pending, local_sub, |item| match item {
            pubsub::Result::EventV2(event) => serialize_subscription_item(&event.event),
            other => {
                warn!(
                    "unexpected pubsub payload for starcoin_subscribeEventsV2: {:?}",
                    other
                );
                None
            }
        })
        .await;
        Ok(())
    }

    async fn subscribe_new_pending_transactions(
        &self,
        pending: PendingSubscriptionSink,
    ) -> SubscriptionResult {
        let local_sub = self
            .subscribe(pubsub::Kind::NewPendingTransactions, None)
            .await
            .map_err(map_anyhow_subscription_err)?;
        forward_subscription_stream(self, pending, local_sub, |item| match item {
            pubsub::Result::TransactionHash(hashes) => serialize_subscription_item(&hashes),
            other => {
                warn!(
                    "unexpected pubsub payload for starcoin_subscribeNewPendingTransactions: {:?}",
                    other
                );
                None
            }
        })
        .await;
        Ok(())
    }

    async fn subscribe_new_mint_block(
        &self,
        pending: PendingSubscriptionSink,
    ) -> SubscriptionResult {
        let local_sub = self
            .subscribe(pubsub::Kind::NewMintBlock, None)
            .await
            .map_err(map_anyhow_subscription_err)?;
        forward_subscription_stream(self, pending, local_sub, |item| match item {
            pubsub::Result::MintBlock(block) => serialize_subscription_item(&*block),
            other => {
                warn!(
                    "unexpected pubsub payload for starcoin_subscribeNewMintBlock: {:?}",
                    other
                );
                None
            }
        })
        .await;
        Ok(())
    }
}

pub struct PubSubServiceFactory;

impl ServiceFactory<PubSubService> for PubSubServiceFactory {
    fn create(ctx: &mut ServiceContext<PubSubService>) -> Result<PubSubService> {
        let miner_service = ctx.service_ref::<MinerService>()?.clone();
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
        Ok(PubSubService::new(
            ctx.get_shared::<TxPoolService>()?,
            miner_service,
            storage,
            storage2,
        ))
    }
}

pub struct PubSubService {
    subscriber_id: LocalSubscriptionId,
    txpool: TxPoolService,
    miner_service: ServiceRef<MinerService>,
    storage: Arc<Storage>,
    storage2: Arc<Storage2>,
    new_header_subscribers:
        HashMap<LocalSubscriptionId, mpsc::UnboundedSender<NewHeadNotification>>,
    new_event_subscribers:
        HashMap<LocalSubscriptionId, mpsc::UnboundedSender<ContractEventNotification>>,
    mint_block_subscribers: HashMap<LocalSubscriptionId, mpsc::UnboundedSender<MintBlockEvent>>,
    new_pending_txn_tasks: Arc<RwLock<HashMap<LocalSubscriptionId, AbortHandle>>>,
}

impl PubSubService {
    fn new(
        txpool: TxPoolService,
        miner_service: ServiceRef<MinerService>,
        storage: Arc<Storage>,
        storage2: Arc<Storage2>,
    ) -> Self {
        Self {
            subscriber_id: 0,
            txpool,
            miner_service,
            storage,
            storage2,
            new_event_subscribers: Default::default(),
            new_header_subscribers: Default::default(),
            mint_block_subscribers: Default::default(),
            new_pending_txn_tasks: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    fn next_id(&mut self) -> LocalSubscriptionId {
        let id = self.subscriber_id;
        self.subscriber_id = self.subscriber_id.saturating_add(1);
        id
    }

    fn subscribe_new_heads(&mut self) -> LocalSubscription {
        let (sender, receiver) = mpsc::unbounded();
        let id = self.next_id();
        self.new_header_subscribers.insert(id, sender);
        let stream = receiver
            .flat_map(move |msg| futures::stream::iter(NewHeadHandler.handle(msg)))
            .boxed();

        LocalSubscription { id, stream }
    }

    fn subscribe_mint_block(&mut self, ctx: &mut ServiceContext<Self>) -> LocalSubscription {
        let (sender, receiver) = mpsc::unbounded();
        let id = self.next_id();
        self.mint_block_subscribers.insert(id, sender.clone());

        let miner_service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
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
                Err(err) => {
                    error!(
                        "[pubsub] Failed to send UpdateSubscriberNumRequest to miner service: {}",
                        err
                    );
                }
            };
        });

        let stream = receiver
            .flat_map(move |msg| futures::stream::iter(NewMintBlockHandler.handle(msg)))
            .boxed();

        LocalSubscription { id, stream }
    }

    fn subscribe_events(&mut self, filter: Filter, decode: bool) -> LocalSubscription {
        let (sender, receiver) = mpsc::unbounded();
        let id = self.next_id();
        self.new_event_subscribers.insert(id, sender);

        let handler = ContractEventHandler {
            storage2: self.storage2.clone(),
            storage: self.storage.clone(),
            filter,
            decode,
        };
        let stream = receiver
            .flat_map(move |msg| futures::stream::iter(handler.handle(msg)))
            .boxed();

        LocalSubscription { id, stream }
    }

    fn subscribe_new_pending_txns(&mut self, ctx: &mut ServiceContext<Self>) -> LocalSubscription {
        let id = self.next_id();
        let tasks = self.new_pending_txn_tasks.clone();
        let id_clone = id;
        let (sender, receiver) = mpsc::unbounded();
        let mut txpool_stream = self.txpool.subscribe_pending_txn();
        let (fut, abort_handle) = future::abortable(async move {
            while let Some(msg) = txpool_stream.next().await {
                if sender.unbounded_send(msg).is_err() {
                    break;
                }
            }
            tasks.write().remove(&id_clone);
        });
        ctx.spawn(async move {
            let _ = fut.await;
        });

        self.new_pending_txn_tasks.write().insert(id, abort_handle);
        let stream = receiver
            .flat_map(move |msg| futures::stream::iter(TxnEventHandler.handle(msg)))
            .boxed();

        LocalSubscription { id, stream }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_subscribe_params_value;
    use serde_json::json;
    use starcoin_rpc_api::types::pubsub::Kind;

    #[test]
    fn parse_kind_only() {
        let parsed =
            parse_subscribe_params_value(json!([{ "type_name": "newHeads" }])).expect("parse kind");
        assert_eq!(parsed.0, Kind::NewHeads);
        assert!(parsed.1.is_none());
    }

    #[test]
    fn parse_kind_vec_only() {
        let parsed = parse_subscribe_params_value(json!([[{ "type_name": "newHeads" }]]))
            .expect("parse kind vec");
        assert_eq!(parsed.0, Kind::NewHeads);
        assert!(parsed.1.is_none());
    }

    #[test]
    fn parse_kind_and_null_params() {
        let parsed = parse_subscribe_params_value(json!([{ "type_name": "newHeads" }, null]))
            .expect("parse kind and null params");
        assert_eq!(parsed.0, Kind::NewHeads);
        assert!(parsed.1.is_none());
    }
}

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
struct Subscribe {
    kind: pubsub::Kind,
    params: Option<pubsub::Params>,
}

impl ServiceRequest for Subscribe {
    type Response = anyhow::Result<LocalSubscription>;
}

impl ServiceHandler<Self, Subscribe> for PubSubService {
    fn handle(
        &mut self,
        msg: Subscribe,
        ctx: &mut ServiceContext<Self>,
    ) -> anyhow::Result<LocalSubscription> {
        match (msg.kind, msg.params) {
            (pubsub::Kind::NewHeads, None) => Ok(self.subscribe_new_heads()),
            (pubsub::Kind::NewHeads, _) => Err(errors::invalid_params(
                "newHeads",
                "Expected no parameters.",
            )),
            (pubsub::Kind::NewPendingTransactions, None) => {
                Ok(self.subscribe_new_pending_txns(ctx))
            }
            (pubsub::Kind::NewPendingTransactions, _) => Err(errors::invalid_params(
                "newPendingTransactions",
                "Expected no parameters.",
            )),
            (pubsub::Kind::Events, Some(param)) if param != Params::None => {
                let (decode, filter) = match param {
                    Params::Events(e) => (e.decode, e.filter.try_into()),
                    Params::EventsV2(e) => (e.decode, e.filter.try_into()),
                    Params::None => {
                        return Err(errors::invalid_params(
                            "events",
                            "Expected a filter object.",
                        ));
                    }
                };

                match filter {
                    Ok(filter) => Ok(self.subscribe_events(filter, decode)),
                    Err(err) => Err(err),
                }
            }
            (pubsub::Kind::Events, _) => Err(errors::invalid_params(
                "events",
                "Expected a filter object.",
            )),
            (pubsub::Kind::NewMintBlock, _) => Ok(self.subscribe_mint_block(ctx)),
        }
    }
}

#[derive(Debug)]
struct Unsubscribe(LocalSubscriptionId);

impl ServiceRequest for Unsubscribe {
    type Response = bool;
}

impl ServiceHandler<Self, Unsubscribe> for PubSubService {
    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut ServiceContext<Self>) -> bool {
        let mut removed = false;
        removed = self.new_header_subscribers.remove(&msg.0).is_some() || removed;
        removed = self.new_event_subscribers.remove(&msg.0).is_some() || removed;

        let mint_removed = self.mint_block_subscribers.remove(&msg.0).is_some();
        removed = mint_removed || removed;
        if mint_removed {
            self.miner_service.do_send(UpdateSubscriberNumRequest {
                number: Some(self.mint_block_subscribers.len() as u32),
            });
        }

        if let Some(h) = self.new_pending_txn_tasks.write().remove(&msg.0) {
            h.abort();
            removed = true;
        }

        removed
    }
}

fn send_to_all<T: Clone>(
    subscriptions: &mut HashMap<LocalSubscriptionId, mpsc::UnboundedSender<T>>,
    msg: T,
) {
    let mut remove_outdated = vec![];

    for (id, ch) in subscriptions.iter() {
        if let Err(err) = ch.unbounded_send(msg.clone()) {
            if err.is_disconnected() {
                remove_outdated.push(*id);
            } else if err.is_full() {
                log::error!(
                    "subscription {:?} failed to send new message: channel is full",
                    id
                );
            }
        }
    }

    for id in remove_outdated {
        subscriptions.remove(&id);
    }
}

trait EventHandler<M> {
    fn handle(&self, msg: M) -> Vec<anyhow::Result<pubsub::Result>>;
}

#[derive(Copy, Clone, Debug)]
pub struct TxnEventHandler;

impl EventHandler<Arc<[HashValue]>> for TxnEventHandler {
    fn handle(&self, msg: Arc<[HashValue]>) -> Vec<anyhow::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::TransactionHash(msg.to_vec()))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewHeadHandler;

impl EventHandler<Notification<ThinBlock>> for NewHeadHandler {
    fn handle(&self, msg: Notification<ThinBlock>) -> Vec<anyhow::Result<pubsub::Result>> {
        let Notification(block) = msg;
        vec![Ok(pubsub::Result::Block(Box::new(BlockView {
            header: block.header.into(),
            body: block.body.into(),
            uncles: vec![],
            raw: None,
        })))]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NewMintBlockHandler;

impl EventHandler<MintBlockEvent> for NewMintBlockHandler {
    fn handle(&self, msg: MintBlockEvent) -> Vec<anyhow::Result<pubsub::Result>> {
        vec![Ok(pubsub::Result::MintBlock(Box::new(msg)))]
    }
}

#[derive(Clone, Debug)]
pub struct ContractEventHandler {
    filter: Filter,
    decode: bool,
    storage: Arc<Storage>,
    storage2: Arc<Storage2>,
}

impl EventHandler<ContractEventNotification> for ContractEventHandler {
    fn handle(&self, msg: ContractEventNotification) -> Vec<anyhow::Result<pubsub::Result>> {
        let Notification((state_root, events, state_root2, events2)) = msg;
        let filtered = events
            .as_ref()
            .iter()
            .map(|e| (Some(e), None))
            .chain(events2.iter().map(|e| (None, Some(e))))
            .filter(|(e1, e2)| {
                let (block_number, e) = match (e1, e2) {
                    (Some(e), None) => (
                        e.block_number,
                        StcContractEvent::V1(e.contract_event.clone()),
                    ),
                    (None, Some(e)) => (
                        e.block_number,
                        StcContractEvent::V2(e.contract_event.clone()),
                    ),
                    _ => panic!("This should not happen!"),
                };
                self.filter.matching(block_number, &e)
            });

        let filtered_events: Vec<_> = match self.filter.limit {
            None => filtered.collect(),
            Some(l) => {
                let mut evts: Vec<_> = filtered.rev().take(l).collect();
                evts.reverse();
                evts
            }
        };

        let (state, state2) = if self.decode {
            (
                Some(ChainStateDB::new(self.storage.clone(), Some(state_root))),
                Some(ChainStateDB2::new(self.storage2.clone(), Some(state_root2))),
            )
        } else {
            (None, None)
        };

        filtered_events
            .into_iter()
            .map(|(e1, e2)| match (e1, e2) {
                (Some(e), None) => {
                    let decoded_data = match &state {
                        Some(s) => {
                            let abi = ABIResolver::new(s)
                                .resolve_type_tag(e.contract_event.type_tag())?;
                            Some(decode_move_value(&abi, e.contract_event.event_data())?)
                        }
                        None => None,
                    };

                    let d = TransactionEventResponse {
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
                    };
                    Ok(pubsub::Result::Event(Box::new(d)))
                }
                (None, Some(e)) => {
                    let decoded_data = match &state2 {
                        Some(s) => {
                            let abi = ABIResolver2::new(s)
                                .resolve_type_tag(e.contract_event.type_tag())?;
                            Some(decode_move_value2(&abi, e.contract_event.event_data())?)
                        }
                        None => None,
                    };
                    let d = TransactionEventResponse2 {
                        event: TransactionEventView2::new(
                            Some(e.block_hash),
                            Some(e.block_number),
                            Some(e.transaction_hash),
                            e.transaction_index,
                            e.transaction_global_index,
                            e.event_index,
                            &e.contract_event,
                        ),
                        decode_event_data: decoded_data,
                    };
                    Ok(pubsub::Result::EventV2(Box::new(d)))
                }
                _ => panic!("This should not happen!"),
            })
            .collect()
    }
}
