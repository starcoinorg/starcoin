// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::{get_unix_ts, is_global};
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
use libp2p::multiaddr::Protocol;
use libp2p::PeerId;
use lru::LruCache;
use network_api::{messages::RawRpcRequestMessage, NetworkService, PeerProvider};
use network_p2p::Multiaddr;
use network_rpc_core::RawRpcClient;
use scs::SCSCodec;
use starcoin_block_relayer_api::{NetCmpctBlockMessage, PeerCmpctBlockEvent};
use starcoin_network_rpc::NetworkRpcService;
use starcoin_network_rpc_api::gen_client::get_rpc_info;
use starcoin_network_rpc_api::CHAIN_PROTOCOL_NAME;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::{BlockStore, Storage};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tx_relay::*;
use types::peer_info::{PeerInfo, RpcInfo};
use types::system_events::NewHeadBlock;
use types::transaction::SignedUserTransaction;
use types::{BLOCK_PROTOCOL_NAME, TXN_PROTOCOL_NAME};

const LRU_CACHE_SIZE: usize = 1024;
const PEERS_FILE_NAME: &str = "peers.json";

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
    node_config: Arc<NodeConfig>,
    peer_id: PeerId,
    network_rpc_service: ServiceRef<NetworkRpcService>,
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

    pub fn register_rpc_proto(&mut self, rpc_info: RpcInfo) {
        self.peer_info.register_rpc_proto(rpc_info)
    }

    pub fn get_peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

#[async_trait]
impl NetworkService for NetworkAsyncService {
    async fn send_peer_message(
        &self,
        protocol_name: Cow<'static, [u8]>,
        peer_id: types::peer_info::PeerId,
        msg: PeerMessage,
    ) -> Result<()> {
        let data = msg.encode()?;
        self.network_service
            .send_message(peer_id.into(), protocol_name, data)
            .await?;

        Ok(())
    }
    async fn broadcast_new_head_block(
        &self,
        _protocol_name: Cow<'static, [u8]>,
        _event: NewHeadBlock,
    ) -> Result<()> {
        unimplemented!()
    }

    async fn register_rpc_proto(&self, rpc_info: RpcInfo) -> Result<()> {
        if let Some(peer_info) = self.inner.peers.lock().await.get_mut(&self.peer_id) {
            peer_info.register_rpc_proto(rpc_info);
            self.inner
                .network_service
                .update_self_info(peer_info.get_peer_info().clone());
        }
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
            None => {
                self.best_peer()
                    .await?
                    .ok_or_else(|| format_err!("No connected peers to request for {:?}", peer_msg))?
                    .peer_id
            }
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

    pub fn start(
        node_config: Arc<NodeConfig>,
        genesis_hash: HashValue,
        bus: ServiceRef<BusService>,
        storage: Arc<Storage>,
        network_rpc_service: ServiceRef<NetworkRpcService>,
    ) -> Result<NetworkAsyncService> {
        let peer_id = node_config.network.self_peer_id()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Can not find startup info."))?;
        let head_block_hash = startup_info.master;
        let head_block = storage
            .get_block(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block by hash {}", head_block_hash))?;
        let head_block_info = storage
            .get_block_info(head_block_hash)?
            .ok_or_else(|| format_err!("can't get block info by hash {}", head_block_hash))?;

        let mut rpc_proto_info = Vec::new();
        let chain_rpc_proto_info = get_rpc_info();
        rpc_proto_info.push(RpcInfo::new(chain_rpc_proto_info));
        let self_info = PeerInfo::new_with_proto(
            peer_id,
            head_block_info.get_total_difficulty(),
            head_block.header().clone(),
            rpc_proto_info,
        );

        // merge seeds from chain config
        let mut config = node_config.network.clone();
        if !node_config.network.disable_seed {
            let seeds = node_config.net().boot_nodes().to_vec();
            config.seeds.extend(seeds);
        }
        let has_seed = !config.seeds.is_empty();

        let (service, tx, rx, event_rx, tx_command) =
            build_network_service(&config, genesis_hash, self_info.clone());
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
        peers.insert(
            self_info.peer_id.clone().into(),
            PeerInfoNet::new(self_info),
        );
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
            node_config,
            peer_id: peer_id.clone(),
            network_rpc_service,
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
                    debug!("receive net message");
                },
                event = event_rx.select_next_some()=>{
                    async_std::task::spawn(Inner::handle_event_receive(inner.clone(),event));
                    debug!("receive net event");
                },
                complete => {
                    close_tx.unbounded_send(()).unwrap();
                    debug!("all stream are complete");
                    break;
                }
            }
        }
    }

    async fn handle_network_receive(inner: Arc<Inner>, network_msg: NetworkMessage) -> Result<()> {
        debug!("receive network_message ");
        // TODO: we should decode msg based on protocol name.
        // when protocol upgrade, we can decoded data based on the new protocol.

        let message = PeerMessage::decode(&network_msg.data);
        match message {
            Ok(msg) => {
                inner
                    .handle_network_message(network_msg.peer_id, msg)
                    .await?;
            }
            Err(e) => {
                debug!("get error {:?}", e);
            }
        }

        Ok(())
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
                self.bus.broadcast(PeerTransactions::new(txns))?;
            }
            PeerMessage::CompactBlock(compact_block, total_diff) => {
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
                        total_diff, peer_info
                    );
                    if total_diff > peer_info.peer_info.total_difficulty {
                        peer_info.peer_info.latest_header = block_header;
                        peer_info.peer_info.total_difficulty = total_diff;
                    }
                }
                self.bus.broadcast(PeerCmpctBlockEvent {
                    peer_id: peer_id.into(),
                    compact_block,
                })?;
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

    async fn handle_event_receive(inner: Arc<Inner>, event: PeerEvent) -> Result<()> {
        debug!("handle_event_receive {:?}", event);
        match event.clone() {
            PeerEvent::Open(peer_id, peer_info) => {
                inner.on_peer_connected(peer_id.into(), *peer_info).await?;
                if inner.need_send_event.load(Ordering::Acquire) {
                    let mut connected_tx = inner.connected_tx.clone();
                    connected_tx.send(event.clone()).await?;
                    inner.need_send_event.swap(false, Ordering::Acquire);
                }
            }
            PeerEvent::Close(peer_id) => {
                inner.on_peer_disconnected(peer_id.into()).await;
            }
        }
        inner.bus.broadcast(event)?;
        Ok(())
    }

    async fn on_peer_connected(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<()> {
        self.peers
            .lock()
            .await
            .entry(peer_id.clone())
            .or_insert_with(|| PeerInfoNet::new(peer_info));

        let path = self.node_config.data_dir();
        let file = Path::new(PEERS_FILE_NAME);

        let path = path.join(file);

        let mut peers = HashSet::new();
        for peer in self.peers.lock().await.keys() {
            if !self.peer_id.eq(peer) {
                peers.insert(peer.clone());
            }
        }
        if path.exists() {
            std::fs::remove_file(path.clone())?;
        }
        let mut addrs_list = HashSet::new();
        let mut addrs_set = HashSet::new();
        for peer_id in peers {
            let addrs = self.network_service.get_address(peer_id.clone()).await;
            for addr in addrs {
                if Self::check_ip(&addr, &mut addrs_set) {
                    let new_addr = addr.with(Protocol::P2p(peer_id.clone().into()));
                    addrs_list.insert(new_addr);
                }
            }
        }
        let mut file = std::fs::File::create(path)?;
        let content = serde_json::to_vec(&addrs_list)?;
        file.write_all(&content)?;

        Ok(())
    }

    fn check_ip(addr: &Multiaddr, addrs_set: &mut HashSet<Multiaddr>) -> bool {
        if addrs_set.contains(addr) {
            return false;
        }
        let components = addr.iter().collect::<Vec<_>>();
        for protocol in components {
            match protocol {
                Protocol::Ip4(ip) => {
                    if !is_global(ip) {
                        return false;
                    }
                }
                Protocol::Ip6(_ip) => {
                    return false;
                }
                _ => {}
            }
        }
        addrs_set.insert(addr.clone());
        true
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
                    "total_difficulty is {},peer_info is {:?}",
                    total_difficulty, peer_info
                );
                if total_difficulty > peer_info.peer_info.total_difficulty {
                    peer_info.peer_info.latest_header = block_header;
                    peer_info.peer_info.total_difficulty = total_difficulty;
                }

                // update self peer info
                let self_info = PeerInfo::new_with_peer_info(
                    self_id.clone().into(),
                    peer_info.peer_info.total_difficulty,
                    peer_info.peer_info.latest_header.clone(),
                    peer_info.get_peer_info(),
                );
                network.update_self_info(self_info);
            }

            for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                if peer_info.known_blocks.contains(&id)
                    || peer_info.peer_info.get_total_difficulty() >= total_difficulty
                {
                    continue;
                }

                peer_info.known_blocks.put(id, ());
                network
                    .send_peer_message(
                        BLOCK_PROTOCOL_NAME.into(),
                        peer_id.clone().into(),
                        msg.clone(),
                    )
                    .await
                    .expect("send message failed ,check network service please");
            }
        })
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
        ctx.spawn(async move {
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
                    .await
                    .expect("check network service");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::block::BlockHeader;

    #[test]
    fn test_peer_info() {
        let mut peer_info = PeerInfo::random();
        peer_info.latest_header = BlockHeader::random();
        let data = peer_info.encode().unwrap();
        let peer_info_decode = PeerInfo::decode(&data).unwrap();
        assert_eq!(peer_info, peer_info_decode);
    }
}
