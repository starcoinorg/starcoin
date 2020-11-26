// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::get_unix_ts;
use crate::message_processor::{MessageFuture, MessageProcessor};
use crate::net::{build_network_service, SNetworkService};
use crate::network_metrics::NetworkMetrics;
use crate::{NetworkMessage, PeerEvent, PeerMessage};
use anyhow::{format_err, Result};
use async_trait::async_trait;
use bitflags::_core::ops::Deref;
use bitflags::_core::sync::atomic::Ordering;
use config::NodeConfig;
use crypto::{hash::PlainCryptoHash, HashValue};
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::FutureExt;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use futures_timer::Delay;
use libp2p::PeerId;
use lru::LruCache;
use network_api::{
    messages::RawRpcRequestMessage, NetworkService, PeerMessageHandler, PeerProvider,
};
use network_p2p::Multiaddr;
use network_rpc_core::RawRpcClient;
use scs::SCSCodec;
use starcoin_block_relayer_api::{NetCmpctBlockMessage, PeerCmpctBlockEvent};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_network_rpc_api::CHAIN_PROTOCOL_NAME;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tx_relay::*;
use types::peer_info::PeerInfo;
use types::startup_info::{ChainInfo, ChainStatus};
use types::transaction::SignedUserTransaction;
use types::{BLOCK_PROTOCOL_NAME, PROTOCOLS, TXN_PROTOCOL_NAME};

const LRU_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct NetworkAsyncService {
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    /// TODO: tx is unused?
    tx: mpsc::UnboundedSender<NetworkMessage>,
    network_service: SNetworkService,
    peer_id: PeerId,
    inner: Arc<Inner>,
    metrics: Option<NetworkMetrics>,
}

impl Deref for NetworkAsyncService {
    type Target = SNetworkService;

    fn deref(&self) -> &Self::Target {
        &self.network_service
    }
}

struct Inner {
    network_service: SNetworkService,
    bus: ServiceRef<BusService>,
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
    connected_tx: mpsc::Sender<PeerEvent>,
    need_send_event: AtomicBool,
    network_rpc_service: ServiceRef<NetworkRpcService>,
    peer_message_handler: Arc<dyn PeerMessageHandler>,
}

#[derive(Debug)]
pub struct PeerInfoNet {
    peer_info: PeerInfo,
    known_transactions: LruCache<HashValue, ()>,
    /// Holds a set of blocks known to this peer.
    known_blocks: LruCache<HashValue, ()>,
}

impl PeerInfoNet {
    fn new(peer_info: PeerInfo) -> Self {
        Self {
            peer_info,
            known_blocks: LruCache::new(LRU_CACHE_SIZE),
            known_transactions: LruCache::new(LRU_CACHE_SIZE),
        }
    }

    pub fn get_peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

#[async_trait]
impl NetworkService for NetworkAsyncService {
    async fn send_peer_message(
        &self,
        protocol_name: Cow<'static, str>,
        peer_id: types::peer_info::PeerId,
        msg: PeerMessage,
    ) -> Result<()> {
        let data = msg.encode()?;
        self.network_service
            .send_message(peer_id.into(), protocol_name, data)
            .await?;

        Ok(())
    }
}

impl PeerProvider for NetworkAsyncService {
    fn identify(&self) -> types::peer_info::PeerId {
        self.peer_id.clone().into()
    }

    fn peer_set(&self) -> BoxFuture<Result<Vec<PeerInfo>>> {
        self.get_peer_set().boxed()
    }

    fn get_peer(&self, peer_id: types::peer_info::PeerId) -> BoxFuture<Result<Option<PeerInfo>>> {
        async move { self.get_peer_by_id(&peer_id.into()).await }.boxed()
    }
}

impl RawRpcClient for NetworkAsyncService {
    fn send_raw_request(
        &self,
        peer_id: Option<network_api::PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        timeout: Duration,
    ) -> BoxFuture<Result<Vec<u8>>> {
        self.send_request_bytes(peer_id, rpc_path, message, timeout)
            .boxed()
    }
}

impl NetworkAsyncService {
    async fn get_peer_set(&self) -> Result<Vec<PeerInfo>> {
        let mut result = vec![];

        for (peer_id, peer) in self.inner.peers.lock().await.iter() {
            if self.peer_id.eq(peer_id) {
                continue;
            }
            result.push(peer.peer_info.clone());
        }
        Ok(result)
    }

    async fn get_peer_by_id(&self, peer_id: &PeerId) -> Result<Option<PeerInfo>> {
        match self.inner.peers.lock().await.get(peer_id) {
            Some(peer) => Ok(Some(peer.peer_info.clone())),
            None => Ok(None),
        }
    }

    pub fn peers(&self) -> Arc<Mutex<HashMap<PeerId, PeerInfoNet>>> {
        self.inner.peers.clone()
    }

    async fn send_request_bytes(
        &self,
        peer_id: Option<types::peer_info::PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>> {
        let request_id = get_unix_ts();
        let peer_msg = PeerMessage::RawRPCRequest(request_id, rpc_path, message);
        let data = peer_msg.encode()?;
        let peer_id = match peer_id {
            Some(peer_id) => peer_id,
            None => self
                .best_peer()
                .await?
                .ok_or_else(|| format_err!("No connected peers to request for {:?}", peer_msg))?
                .peer_id(),
        };
        debug!(
            "Send request to {} with id {} and msg: {:?}",
            peer_id, request_id, peer_msg
        );

        self.network_service
            .send_message(peer_id.clone().into(), CHAIN_PROTOCOL_NAME.into(), data)
            .await?;

        let (tx, rx) = futures::channel::mpsc::channel(1);
        let message_future = MessageFuture::new(rx);
        self.raw_message_processor
            .add_future(request_id, tx, peer_id.clone().into())
            .await;

        let processor = self.raw_message_processor.clone();

        if let Some(metrics) = &self.metrics {
            metrics.request_count.inc();
        }

        let metrics = self.metrics.clone();
        let peer_id_for_task = peer_id.clone();
        let task = async move {
            Delay::new(time_out).await;
            let timeout = processor.remove_future(request_id).await;
            if !timeout {
                return;
            }
            debug!(
                "send request to {} with id {} timeout",
                peer_id_for_task, request_id
            );
            if let Some(metrics) = metrics {
                metrics.request_timeout_count.inc();
            }
        };

        async_std::task::spawn(task);
        let response = message_future.await;
        debug!("receive response from {} with id {}", peer_id, request_id);
        response
    }

    pub fn add_peer(&self, peer: String) -> Result<()> {
        self.network_service.add_peer(peer)
    }

    pub async fn connected_peers(&self) -> Vec<types::peer_info::PeerId> {
        self.network_service
            .connected_peers()
            .await
            .into_iter()
            .map(|peer_id| peer_id.into())
            .collect()
    }

    pub async fn get_address(&self, peer_id: types::peer_info::PeerId) -> Vec<Multiaddr> {
        self.network_service.get_address(peer_id.into()).await
    }

    pub fn start<H>(
        node_config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        bus: ServiceRef<BusService>,
        network_rpc_service: ServiceRef<NetworkRpcService>,
        peer_message_handler: H,
    ) -> Result<NetworkAsyncService>
    where
        H: PeerMessageHandler + 'static,
    {
        let peer_id = node_config.network.self_peer_id()?;

        let self_info = PeerInfo::new(peer_id, chain_info.clone());

        // merge seeds from chain config
        let mut config = node_config.network.clone();
        if !node_config.network.disable_seed {
            let seeds = node_config.net().boot_nodes().to_vec();
            config.seeds.extend(seeds);
        }
        let has_seed = !config.seeds.is_empty();

        let (service, tx, rx, event_rx, tx_command) =
            build_network_service(chain_info, &config, PROTOCOLS.clone());
        info!(
            "network started at {} with seed {},network address is {}",
            &node_config.network.listen,
            &node_config
                .network
                .seeds
                .iter()
                .fold(String::new(), |acc, arg| acc + arg.to_string().as_str()),
            service.identify()
        );

        let raw_message_processor = MessageProcessor::new();
        let raw_message_processor_clone = raw_message_processor.clone();

        let peer_id = service.identify().clone();

        let mut peers = HashMap::new();
        peers.insert(self_info.peer_id().into(), PeerInfoNet::new(self_info));
        let peers = Arc::new(Mutex::new(peers));

        let (connected_tx, mut connected_rx) = futures::channel::mpsc::channel(1);
        let need_send_event = AtomicBool::new(false);

        if has_seed && !node_config.network.disable_seed {
            need_send_event.swap(true, Ordering::Acquire);
        }

        let metrics = NetworkMetrics::register().ok();

        let inner = Inner {
            network_service: service.clone(),
            bus,
            raw_message_processor: raw_message_processor_clone,
            peers,
            connected_tx,
            need_send_event,
            network_rpc_service,
            peer_message_handler: Arc::new(peer_message_handler),
        };
        let inner = Arc::new(inner);

        // TODO: unify all async runtimes into one.
        async_std::task::spawn(Inner::start(inner.clone(), rx, event_rx, tx_command));

        if has_seed {
            info!("Seed was in configuration and not ignored.So wait for connection open event.");
            futures::executor::block_on(async move {
                if let Some(event) = connected_rx.next().await {
                    info!("Receive event {:?}, network started.", event);
                } else {
                    error!("Wait peer event return None.");
                }
            });
        }

        Ok(NetworkAsyncService {
            raw_message_processor,
            network_service: service,
            tx,
            peer_id,
            inner,
            metrics,
        })
    }
}

impl Inner {
    async fn start(
        inner: Arc<Inner>,
        net_rx: mpsc::UnboundedReceiver<NetworkMessage>,
        event_rx: mpsc::UnboundedReceiver<PeerEvent>,
        close_tx: mpsc::UnboundedSender<()>,
    ) {
        let mut net_rx = net_rx.fuse();
        let mut event_rx = event_rx.fuse();

        loop {
            futures::select! {
                message = net_rx.select_next_some()=>{
                    async_std::task::spawn(Inner::handle_network_receive(inner.clone(),message));
                },
                event = event_rx.select_next_some()=>{
                    async_std::task::spawn(Inner::handle_event_receive(inner.clone(),event));
                },
                complete => {
                    close_tx.unbounded_send(()).unwrap();
                    debug!("all stream are complete");
                    break;
                }
            }
        }
    }

    async fn handle_network_receive(inner: Arc<Inner>, network_msg: NetworkMessage) {
        debug!(
            "Receive network_message from peer: {:?}",
            network_msg.peer_id,
        );
        // TODO: we should decode msg based on protocol name.
        // when protocol upgrade, we can decoded data based on the new protocol.

        let message = PeerMessage::decode(&network_msg.data);
        match message {
            Ok(msg) => {
                if let Err(e) = inner.handle_network_message(network_msg.peer_id, msg).await {
                    warn!("Handle_network_message error: {:?}", e);
                }
            }
            Err(e) => {
                warn!("Decode network message {:?} error {:?}", network_msg, e);
            }
        }
    }

    async fn handle_network_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()> {
        match msg {
            PeerMessage::NewTransactions(txns) => {
                debug!("receive new txn list from {:?} ", peer_id);
                if let Some(peer_info) = self.peers.lock().await.get_mut(&peer_id) {
                    for txn in &txns {
                        let id = txn.crypto_hash();
                        if !peer_info.known_transactions.contains(&id) {
                            peer_info.known_transactions.put(id, ());
                        } else {
                            return Ok(());
                        }
                    }
                }
                self.peer_message_handler
                    .handle_transaction(PeerTransactions::new(txns));
            }
            PeerMessage::CompactBlock(compact_block, total_difficulty) => {
                //TODO: Check total difficulty
                let block_header = compact_block.header.clone();
                debug!(
                    "Receive new compact block from {:?} with hash {:?}",
                    peer_id,
                    block_header.id()
                );

                if let Some(peer_info) = self.peers.lock().await.get_mut(&peer_id) {
                    debug!(
                        "total_difficulty is {},peer_info is {:?}",
                        total_difficulty, peer_info
                    );
                    if total_difficulty > peer_info.peer_info.total_difficulty() {
                        peer_info
                            .peer_info
                            .update_chain_status(ChainStatus::new(block_header, total_difficulty));
                    }
                } else {
                    error!(
                        "Receive compat block from {}, but can not find it peer info.",
                        peer_id
                    )
                }
                self.peer_message_handler.handle_block(PeerCmpctBlockEvent {
                    peer_id: peer_id.into(),
                    compact_block,
                });
            }

            PeerMessage::RawRPCRequest(id, rpc_path, request) => {
                debug!("do request {} from peer {}", id, peer_id);
                let (tx, rx) = mpsc::channel(1);
                self.network_rpc_service.try_send(RawRpcRequestMessage {
                    responder: tx,
                    request: (peer_id.clone().into(), rpc_path, request),
                })?;
                let network_service = self.network_service.clone();
                async_std::task::spawn(Self::handle_response(id, peer_id, rx, network_service));
            }
            PeerMessage::RawRPCResponse(id, response) => {
                debug!("do response {} from peer {}", id, peer_id);
                self.raw_message_processor
                    .send_response(id, response)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_response(
        id: u128,
        peer_id: PeerId,
        mut rx: mpsc::Receiver<Vec<u8>>,
        network_service: SNetworkService,
    ) -> Result<()> {
        let response = rx.next().await;
        match response {
            Some(response) => {
                let peer_msg = PeerMessage::RawRPCResponse(id, response);
                let data = peer_msg.encode()?;
                network_service
                    .send_message(peer_id, CHAIN_PROTOCOL_NAME.into(), data)
                    .await?;
                debug!("send response by id {} succ.", id);
                Ok(())
            }
            None => {
                debug!("can't get response by id {}", id);
                Ok(())
            }
        }
    }

    async fn handle_event_receive(inner: Arc<Inner>, event: PeerEvent) {
        if let Err(e) = inner.do_handle_event_receive(event).await {
            warn!("Handle peer event error: {}", e);
        }
    }

    async fn do_handle_event_receive(&self, event: PeerEvent) -> Result<()> {
        debug!("handle_event_receive {:?}", event);
        match event.clone() {
            PeerEvent::Open(peer_id, chain_info) => {
                self.on_peer_connected(peer_id.into(), *chain_info).await?;
                if self.need_send_event.load(Ordering::Acquire) {
                    let mut connected_tx = self.connected_tx.clone();
                    connected_tx.send(event.clone()).await?;
                    self.need_send_event.swap(false, Ordering::Acquire);
                }
            }
            PeerEvent::Close(peer_id) => {
                self.on_peer_disconnected(peer_id.into()).await;
            }
        }
        self.bus.broadcast(event)?;
        Ok(())
    }

    async fn on_peer_connected(&self, peer_id: PeerId, chain_info: ChainInfo) -> Result<()> {
        self.peers
            .lock()
            .await
            .entry(peer_id.clone())
            .or_insert_with(|| PeerInfoNet::new(PeerInfo::new(peer_id.into(), chain_info)));

        Ok(())
    }

    async fn on_peer_disconnected(&self, peer_id: PeerId) {
        self.peers.lock().await.remove(&peer_id);
    }
}

// TODO: figure out a better place for the actor.
/// Used to manage broadcast new txn and new block event to other network peers.
pub struct PeerMsgBroadcasterService {
    network: NetworkAsyncService,
}

impl PeerMsgBroadcasterService {
    pub fn new(network: NetworkAsyncService) -> Self {
        Self { network }
    }
}

impl ServiceFactory<Self> for PeerMsgBroadcasterService {
    fn create(
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) -> Result<PeerMsgBroadcasterService> {
        let network = ctx.get_shared::<NetworkAsyncService>()?;
        Ok(Self::new(network))
    }
}

impl ActorService for PeerMsgBroadcasterService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<PropagateNewTransactions>();
        ctx.subscribe::<NetCmpctBlockMessage>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PropagateNewTransactions>();
        ctx.unsubscribe::<NetCmpctBlockMessage>();
        Ok(())
    }
}

impl EventHandler<Self, NetCmpctBlockMessage> for PeerMsgBroadcasterService {
    fn handle_event(
        &mut self,
        msg: NetCmpctBlockMessage,
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) {
        let id = msg.compact_block.header.id();
        debug!("broadcast new compact block message {:?}", id);
        let network = self.network.clone();
        let block_header = msg.compact_block.header.clone();
        let total_difficulty = msg.total_difficulty;
        let msg = PeerMessage::CompactBlock(msg.compact_block, total_difficulty);
        let self_id: PeerId = self.network.identify().into();
        ctx.spawn(async move {
            let peers = network.peers();
            if let Some(peer_info) = peers.lock().await.get_mut(&self_id) {
                debug!(
                    "total_difficulty is {}, peer_info is {:?}",
                    total_difficulty, peer_info
                );

                let chain_status = ChainStatus::new(block_header.clone(), total_difficulty);
                peer_info.peer_info.update_chain_status(chain_status.clone());
                network.update_chain_status(chain_status);
            }else{
                error!("Can not find self peer info {:?}", &self_id);
            }

            for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                if peer_info.known_blocks.contains(&id)
                    || peer_info.peer_info.total_difficulty() >= total_difficulty
                {
                    debug!("peer({:?})'s total_difficulty is > block({:?})'s total_difficulty or it know this block, so do not broadcast. ", peer_id, id);
                    continue;
                }

                peer_info.known_blocks.put(id, ());
                network
                    .send_peer_message(
                        BLOCK_PROTOCOL_NAME.into(),
                        peer_id.clone().into(),
                        msg.clone(),
                    )
                    .await?;
            }
            Ok(())
        }.then(|result: Result<()>| async move{
            if let Err(e) = result{
                error!("[peer-message-broadcaster] Handle NetCmpctBlockMessage error: {:?}", e);
            }
        }))
    }
}

/// handle txn relayer
impl EventHandler<Self, PropagateNewTransactions> for PeerMsgBroadcasterService {
    fn handle_event(
        &mut self,
        msg: PropagateNewTransactions,
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) {
        let (protocol_name, txns) = {
            (TXN_PROTOCOL_NAME, msg.propagate_transaction())
            // new version of txn message can come here
        };
        // false positive
        if txns.is_empty() {
            return;
        }
        debug!("propagate new txns, len: {}", txns.len());

        let network_service = self.network.clone();
        let mut txn_map: HashMap<HashValue, SignedUserTransaction> = HashMap::new();
        for txn in txns {
            txn_map.insert(txn.crypto_hash(), txn);
        }
        let self_peer_id: PeerId = self.network.identify().into();
        ctx.spawn(
            async move {
                let peers = network_service.peers();
                for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                    let mut txns_unhandled = Vec::new();
                    for (id, txn) in &txn_map {
                        if !peer_info.known_transactions.contains(id)
                            && !peer_id.eq(&self_peer_id.clone())
                        {
                            peer_info.known_transactions.put(*id, ());
                            txns_unhandled.push(txn.clone());
                        }
                    }
                    network_service
                        .send_peer_message(
                            Cow::Borrowed(protocol_name),
                            peer_id.clone().into(),
                            PeerMessage::NewTransactions(txns_unhandled),
                        )
                        .await?;
                }
                Ok(())
            }
            .then(|result: Result<()>| async move {
                if let Err(e) = result {
                    error!(
                        "[peer-message-broadcaster] Handle PropagateNewTransactions error: {:?}",
                        e
                    );
                }
            }),
        );
    }
}
