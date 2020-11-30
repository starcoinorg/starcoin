// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::net::{build_network_service, SNetworkService, RPC_PROTOCOL_PREFIX};
use crate::network_metrics::NetworkMetrics;
use crate::{NetworkMessage, PeerEvent, PeerMessage};
use anyhow::Result;
use async_trait::async_trait;
use config::NodeConfig;
use crypto::HashValue;
use futures::future::BoxFuture;
use futures::lock::Mutex;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use futures::{FutureExt, TryFutureExt};
use lru::LruCache;
use network_api::messages::NotificationMessage;
use network_api::{
    messages::TransactionsMessage, NetworkService, PeerMessageHandler, PeerProvider,
};
use network_p2p::Multiaddr;
use network_p2p_types::PeerId;
use network_rpc_core::RawRpcClient;
use starcoin_network_rpc::NetworkRpcService;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_txpool_api::PropagateNewTransactions;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use types::peer_info::{PeerInfo, RpcInfo};
use types::startup_info::{ChainInfo, ChainStatus};

const LRU_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct NetworkAsyncService {
    /// TODO: tx is unused?
    tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_id: PeerId,
    inner: Arc<Inner>,
}

struct Inner {
    network_service: SNetworkService,
    bus: ServiceRef<BusService>,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
    connected_tx: mpsc::Sender<PeerEvent>,
    need_send_event: AtomicBool,
    peer_message_handler: Arc<dyn PeerMessageHandler>,
    metrics: Option<NetworkMetrics>,
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
    async fn send_peer_message(&self, msg: PeerMessage) -> Result<()> {
        self.inner.send_peer_message(msg).await
    }

    async fn broadcast(&self, notification: NotificationMessage) {
        let inner = self.inner.clone();
        async_std::task::spawn(async move {
            let protocol = notification.protocol_name();
            if let Err(e) = inner.broadcast(notification).await {
                warn!(
                    "[network-broadcast] Broadcast {} message error: {:?}",
                    protocol, e
                );
            }
        });
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
        peer_id: network_api::PeerId,
        rpc_path: String,
        message: Vec<u8>,
        _timeout: Duration,
    ) -> BoxFuture<Result<Vec<u8>>> {
        let protocol = format!("{}{}", RPC_PROTOCOL_PREFIX, rpc_path);
        self.inner
            .network_service
            .request(peer_id, protocol, message)
            .map_err(|e| e.into())
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

    pub fn add_peer(&self, peer: String) -> Result<()> {
        self.inner.network_service.add_peer(peer)
    }

    pub async fn connected_peers(&self) -> Vec<types::peer_info::PeerId> {
        self.inner
            .network_service
            .connected_peers()
            .await
            .into_iter()
            .map(|peer_id| peer_id.into())
            .collect()
    }

    pub async fn get_address(&self, peer_id: types::peer_info::PeerId) -> Vec<Multiaddr> {
        self.inner.network_service.get_address(peer_id.into()).await
    }

    pub fn start<H>(
        node_config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        bus: ServiceRef<BusService>,
        rpc_info: RpcInfo,
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

        let (service, tx, rx, event_rx, tx_command) = build_network_service(
            chain_info,
            &config,
            NotificationMessage::protocols(),
            Some((rpc_info, network_rpc_service)),
        );
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
            network_service: service,
            bus,
            peers,
            connected_tx,
            need_send_event,
            peer_message_handler: Arc::new(peer_message_handler),
            metrics,
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

        Ok(NetworkAsyncService { tx, peer_id, inner })
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
        if let Err(e) = inner.handle_network_message(network_msg).await {
            error!("Handle_network_message error: {:?}", e);
        }
    }

    async fn handle_network_message(&self, network_msg: NetworkMessage) -> Result<()> {
        let peer_id = network_msg.peer_id;
        if let Some(peer_info) = self.peers.lock().await.get_mut(&peer_id) {
            let notification = NotificationMessage::decode_notification(
                network_msg.protocol_name,
                network_msg.data.as_slice(),
            )?;
            match &notification {
                NotificationMessage::Transactions(peer_transactions) => {
                    for txn in &peer_transactions.txns {
                        let id = txn.id();
                        peer_info.known_transactions.put(id, ());
                    }
                }
                NotificationMessage::CompactBlock(compact_block_message) => {
                    let block_header = compact_block_message.compact_block.header.clone();
                    let total_difficulty = compact_block_message.total_difficulty;
                    let block_id = block_header.id();
                    debug!(
                        "Receive new compact block from {:?} with hash {:?}",
                        peer_id, block_id
                    );
                    debug!(
                        "total_difficulty is {},peer_info is {:?}",
                        total_difficulty, peer_info
                    );
                    peer_info.known_blocks.put(block_id, ());
                    peer_info
                        .peer_info
                        .update_chain_status(ChainStatus::new(block_header, total_difficulty));
                }
            }

            let peer_message = PeerMessage::new(peer_id.into(), notification);
            self.peer_message_handler.handle_message(peer_message);
        } else {
            error!(
                "Receive NetworkMessage from unknown peer {}, protocol: {}",
                peer_id, network_msg.protocol_name
            )
        }
        Ok(())
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

    async fn send_peer_message(&self, msg: PeerMessage) -> Result<()> {
        let peer_id = msg.peer_id;
        let (protocol_name, data) = msg.notification.encode_notification()?;
        self.network_service
            .send_message(peer_id.into(), protocol_name, data)
            .await?;
        Ok(())
    }

    async fn broadcast(&self, notification: NotificationMessage) -> Result<()> {
        let protocol_name = notification.protocol_name();
        let _timer = self.metrics.as_ref().map(|metrics| {
            metrics
                .broadcast_duration
                .with_label_values(&[protocol_name.as_ref()])
                .start_timer()
        });
        let self_id = self.network_service.identify();
        match &notification {
            NotificationMessage::CompactBlock(msg) => {
                let id = msg.compact_block.header.id();
                debug!("broadcast new compact block message {:?}", id);
                let block_header = msg.compact_block.header.clone();
                let total_difficulty = msg.total_difficulty;
                if let Some(peer_info) = self.peers.lock().await.get_mut(self_id) {
                    debug!(
                        "total_difficulty is {}, peer_info is {:?}",
                        total_difficulty, peer_info
                    );

                    let chain_status = ChainStatus::new(block_header.clone(), total_difficulty);
                    peer_info
                        .peer_info
                        .update_chain_status(chain_status.clone());
                    self.network_service.update_chain_status(chain_status);
                } else {
                    error!("Can not find self peer info {:?}", self_id);
                }

                let send_futures = self.peers.lock().await.iter_mut().filter_map(|(peer_id, peer_info)|{
                    if peer_id == self_id {
                        return None
                    }
                    if peer_info.known_blocks.contains(&id)
                        || peer_info.peer_info.total_difficulty() >= total_difficulty
                    {
                        debug!("peer({:?})'s total_difficulty is > block({:?})'s total_difficulty or it know this block, so do not broadcast. ", peer_id, id);
                        None
                    }else{
                        peer_info.known_blocks.put(id, ());
                        Some(self
                            .send_peer_message(
                                PeerMessage::new(peer_id.clone().into(), notification.clone())
                            ))
                    }
                }).collect::<Vec<_>>();
                futures::future::join_all(send_futures)
                    .await
                    .into_iter()
                    .collect::<Result<_>>()?;
                Ok(())
            }
            NotificationMessage::Transactions(msg) => {
                let send_futures = self
                    .peers
                    .lock()
                    .await
                    .iter_mut()
                    .filter_map(|(peer_id, peer_info)| {
                        if peer_id == self_id {
                            return None;
                        }
                        let mut txns_unhandled = Vec::new();
                        for txn in &msg.txns {
                            let id = txn.id();
                            if !peer_info.known_transactions.contains(&id) {
                                peer_info.known_transactions.put(id, ());
                                txns_unhandled.push(txn.clone());
                            }
                        }
                        if txns_unhandled.is_empty() {
                            debug!(
                                "{} known all transactions, ignore broadcast message.",
                                peer_id
                            );
                            return None;
                        }
                        Some(self.send_peer_message(PeerMessage::new_transactions(
                            peer_id.clone().into(),
                            TransactionsMessage::new(txns_unhandled),
                        )))
                    })
                    .collect::<Vec<_>>();
                futures::future::join_all(send_futures)
                    .await
                    .into_iter()
                    .collect::<Result<_>>()?;
                Ok(())
            }
        }
    }
}

// TODO: figure out a better place for the actor.
/// Used to manage broadcast new txn to other network peers.
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
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<PropagateNewTransactions>();
        Ok(())
    }
}

// handle txn relayer
impl EventHandler<Self, PropagateNewTransactions> for PeerMsgBroadcasterService {
    fn handle_event(
        &mut self,
        msg: PropagateNewTransactions,
        ctx: &mut ServiceContext<PeerMsgBroadcasterService>,
    ) {
        let txns = msg.propagate_transaction();
        if txns.is_empty() {
            error!("broadcast PropagateNewTransactions is empty.");
            return;
        }
        debug!("propagate new txns, len: {}", txns.len());

        let network_service = self.network.clone();
        ctx.spawn(async move {
            network_service
                .broadcast(NotificationMessage::Transactions(TransactionsMessage::new(
                    txns,
                )))
                .await;
        });
    }
}
