// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::{get_unix_ts, is_global};
use crate::message_processor::{MessageFuture, MessageProcessor};
use crate::net::{build_network_service, SNetworkService};
use crate::{NetworkMessage, PeerEvent, PeerMessage};
use actix::prelude::*;
use anyhow::{bail, Result};
use bitflags::_core::sync::atomic::Ordering;
use bus::{Broadcast, Bus, BusActor};
use config::NodeConfig;
use crypto::{hash::PlainCryptoHash, HashValue};
use futures::lock::Mutex;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use futures_timer::Delay;
use libp2p::multiaddr::Protocol;
use libp2p::PeerId;
use lru::LruCache;
use network_api::{messages::RawRpcRequestMessage, NetworkService};
use network_p2p::Multiaddr;

use crate::network_metrics::NetworkMetrics;
use async_trait::async_trait;
use scs::SCSCodec;
use starcoin_sync_api::sync_messages::PeerNewBlock;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;
use tx_relay::*;
use types::peer_info::PeerInfo;
use types::system_events::NewHeadBlock;
use types::transaction::SignedUserTransaction;

const LRU_CACHE_SIZE: usize = 1024;
const PEERS_FILE_NAME: &str = "peers.json";

#[derive(Clone)]
pub struct NetworkAsyncService {
    addr: Addr<NetworkActor>,
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_id: PeerId,
    handle: Handle,
    inner: Arc<Inner>,
    metrics: Option<NetworkMetrics>,
}

struct Inner {
    network_service: SNetworkService,
    bus: Addr<BusActor>,
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    handle: Handle,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
    connected_tx: mpsc::Sender<PeerEvent>,
    need_send_event: AtomicBool,
    node_config: Arc<NodeConfig>,
    peer_id: PeerId,
}

#[derive(Debug)]
struct PeerInfoNet {
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
}

#[async_trait]
impl NetworkService for NetworkAsyncService {
    async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()> {
        let data = msg.encode()?;
        let network_message = NetworkMessage { peer_id, data };
        self.tx.unbounded_send(network_message)?;

        Ok(())
    }
    async fn broadcast_new_head_block(&self, event: NewHeadBlock) -> Result<()> {
        self.addr.send(event).await?;
        Ok(())
    }

    fn identify(&self) -> &PeerId {
        &self.peer_id
    }

    async fn send_request_bytes(
        &self,
        peer_id: PeerId,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>> {
        let request_id = get_unix_ts();
        let peer_msg = PeerMessage::RawRPCRequest(request_id, message);
        let data = peer_msg.encode()?;
        let network_message = NetworkMessage {
            peer_id: peer_id.clone(),
            data,
        };
        self.tx.unbounded_send(network_message)?;
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let message_future = MessageFuture::new(rx);
        self.raw_message_processor.add_future(request_id, tx).await;
        info!("send request to {} with id {}", peer_id, request_id);
        let processor = self.raw_message_processor.clone();
        let peer_id_clone = peer_id.clone();

        if let Some(metrics) = &self.metrics {
            metrics.request_count.inc();
        }

        let metrics = self.metrics.clone();
        let task = async move {
            Delay::new(time_out).await;
            let timeout = processor.remove_future(request_id).await;
            if !timeout {
                return;
            }
            warn!(
                "send request to {} with id {} timeout",
                peer_id_clone, request_id
            );
            if let Some(metrics) = metrics {
                metrics.request_timeout_count.inc();
            }
        };

        self.handle.spawn(task);
        let response = message_future.await;
        info!("receive response from {} with id {}", peer_id, request_id);
        response
    }

    async fn peer_set(&self) -> Result<Vec<PeerInfo>> {
        let mut result = vec![];

        for (peer_id, peer) in self.inner.peers.lock().await.iter() {
            if self.peer_id.eq(peer_id) {
                continue;
            }
            info!("peer_id is {},peer_info is {:?}", peer_id, peer);
            result.push(peer.peer_info.clone());
        }
        info!("result is {:?}", result);
        Ok(result)
    }
    /// get all peers and sort by difficulty decreasely.
    async fn best_peer_set(&self) -> Result<Vec<PeerInfo>> {
        let mut peer_infos = self.peer_set().await?;
        peer_infos.sort_by_key(|p| p.total_difficulty);
        peer_infos.reverse();
        Ok(peer_infos)
    }

    async fn get_peer(&self, peer_id: &PeerId) -> Result<Option<PeerInfo>> {
        match self.inner.peers.lock().await.get(peer_id) {
            Some(peer) => Ok(Some(peer.peer_info.clone())),
            None => Ok(None),
        }
    }

    async fn get_self_peer(&self) -> Result<PeerInfo> {
        match self.inner.peers.lock().await.get(&self.peer_id) {
            Some(peer) => Ok(peer.peer_info.clone()),
            None => bail!("Can not find self peer info."),
        }
    }

    async fn best_peer(&self) -> Result<Option<PeerInfo>> {
        let self_peer_id = types::peer_info::PeerId::new(self.peer_id.clone());
        let best_peer_set = self.best_peer_set().await?;
        let best_peer = best_peer_set
            .iter()
            .find(|peer| self_peer_id != peer.get_peer_id());
        match best_peer {
            Some(peer) => Ok(Some(peer.clone())),
            None => Ok(None),
        }
    }

    async fn get_peer_set_size(&self) -> Result<usize> {
        let size = self.inner.peers.lock().await.len();
        Ok(size)
    }
}

impl NetworkAsyncService {
    #[cfg(test)]
    pub fn network_actor_addr(&self) -> Addr<NetworkActor> {
        self.addr.clone()
    }
}

pub struct NetworkActor {
    network_service: SNetworkService,
    bus: Addr<BusActor>,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
    peer_id: PeerId,
}

impl NetworkActor {
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        handle: Handle,
        genesis_hash: HashValue,
        self_info: PeerInfo,
    ) -> NetworkAsyncService {
        let has_seed = !node_config.network.seeds.is_empty();

        let (service, tx, rx, event_rx, tx_command) = build_network_service(
            &node_config.network,
            handle.clone(),
            genesis_hash,
            self_info.clone(),
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

        let raw_message_processor = MessageProcessor::new();
        let raw_message_processor_clone = raw_message_processor.clone();

        let peer_id = service.identify().clone();
        let peer_id_clone = peer_id.clone();

        let service_clone = service.clone();
        let bus_clone = bus.clone();
        let mut peers = HashMap::new();
        peers.insert(
            self_info.peer_id.clone().into(),
            PeerInfoNet::new(self_info),
        );
        let peers = Arc::new(Mutex::new(peers));
        let peers_clone = peers.clone();
        let addr = NetworkActor::create(move |_ctx: &mut Context<NetworkActor>| NetworkActor {
            network_service: service_clone,
            bus: bus_clone,
            peers: peers_clone,
            peer_id: peer_id_clone,
        });
        let (connected_tx, mut connected_rx) = futures::channel::mpsc::channel(1);
        let need_send_event = AtomicBool::new(false);

        if has_seed {
            need_send_event.swap(true, Ordering::Acquire);
        }

        let metrics = NetworkMetrics::register().ok();
        let inner = Inner {
            network_service: service,
            bus,
            handle: handle.clone(),
            raw_message_processor: raw_message_processor_clone,
            peers,
            connected_tx,
            need_send_event,
            node_config,
            peer_id: peer_id.clone(),
        };
        let inner = Arc::new(inner);
        handle.spawn(Self::start(
            handle.clone(),
            inner.clone(),
            rx,
            event_rx,
            tx_command,
        ));

        if has_seed {
            futures::executor::block_on(async move {
                let event = connected_rx.next().await.unwrap();
                info!("receive event {:?}", event);
            });
        }

        NetworkAsyncService {
            addr,
            raw_message_processor,
            tx,
            peer_id,
            inner,
            handle,
            metrics,
        }
    }

    async fn start(
        handle: Handle,
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
                    handle.spawn(Inner::handle_network_receive(inner.clone(),message));
                    info!("receive net message");
                },
                event = event_rx.select_next_some()=>{
                    handle.spawn(Inner::handle_event_receive(inner.clone(),event));
                    info!("receive net event");
                },
                complete => {
                    close_tx.unbounded_send(()).unwrap();
                    warn!("all stream are complete");
                    break;
                }
            }
        }
    }
}

impl Inner {
    async fn handle_network_receive(inner: Arc<Inner>, network_msg: NetworkMessage) -> Result<()> {
        info!("receive network_message ");
        let message = PeerMessage::decode(&network_msg.data);
        match message {
            Ok(msg) => {
                inner
                    .handle_network_message(network_msg.peer_id, msg)
                    .await?
            }
            Err(e) => {
                warn!("get error {:?}", e);
            }
        }
        Ok(())
    }

    async fn handle_network_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()> {
        match msg {
            PeerMessage::UserTransactions(txns) => {
                info!("receive new txn list from {:?} ", peer_id);
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
                self.bus
                    .clone()
                    .broadcast(PeerTransactions::new(txns))
                    .await?;
            }
            PeerMessage::Block(block) => {
                let block_hash = block.header().id();

                info!(
                    "receive new block from {:?} with hash {:?}",
                    peer_id, block_hash
                );
                let block_number = block.header().number();
                let total_difficulty = block.get_total_difficulty();

                if let Some(peer_info) = self.peers.lock().await.get_mut(&peer_id) {
                    debug!(
                        "total_difficulty is {},peer_info is {:?}",
                        total_difficulty, peer_info
                    );
                    if total_difficulty > peer_info.peer_info.total_difficulty {
                        peer_info.peer_info.block_number = block_number;
                        peer_info.peer_info.block_id = block_hash;
                        peer_info.peer_info.total_difficulty = total_difficulty;
                    }
                }

                self.bus
                    .send(Broadcast {
                        msg: PeerNewBlock::new(peer_id.into(), block.get_block().clone()),
                    })
                    .await?;
            }
            PeerMessage::RawRPCRequest(id, request) => {
                info!("do request.");
                let (tx, rx) = mpsc::channel(1);
                self.bus
                    .send(Broadcast {
                        msg: RawRpcRequestMessage {
                            responder: tx,
                            request,
                        },
                    })
                    .await?;
                let network_service = self.network_service.clone();
                self.handle
                    .spawn(Self::handle_response(id, peer_id, rx, network_service));
                info!("receive rpc request");
            }
            PeerMessage::RawRPCResponse(id, response) => {
                info!("do response.");
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
                network_service.send_message(peer_id, data).await?;
                Ok(())
            }
            None => {
                info!("can't get response by id {}", id);
                Ok(())
            }
        }
    }

    async fn handle_event_receive(inner: Arc<Inner>, event: PeerEvent) -> Result<()> {
        info!("event is {:?}", event);
        match event.clone() {
            PeerEvent::Open(peer_id, peer_info) => {
                inner.on_peer_connected(peer_id.into(), peer_info).await?;
                if inner.need_send_event.load(Ordering::Acquire) {
                    info!("send event");
                    let mut connected_tx = inner.connected_tx.clone();
                    connected_tx.send(event.clone()).await?;
                    inner.need_send_event.swap(false, Ordering::Acquire);
                }
            }
            PeerEvent::Close(peer_id) => {
                inner.on_peer_disconnected(peer_id.into()).await;
            }
        }
        inner.bus.send(Broadcast { msg: event }).await?;
        info!("already broadcast event");
        Ok(())
    }

    async fn on_peer_connected(&self, peer_id: PeerId, peer_info: PeerInfo) -> Result<()> {
        self.peers
            .lock()
            .await
            .entry(peer_id.clone())
            .or_insert_with(|| PeerInfoNet::new(peer_info));

        let path = self.node_config.base.data_dir();
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

impl Actor for NetworkActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let txn_propagate_recipient = ctx.address().recipient::<PropagateNewTransactions>();
        self.bus
            .clone()
            .subscribe(txn_propagate_recipient)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe txn propagate events, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
        info!("Network actor started ",);
    }
}

/// handler system events.
impl Handler<NewHeadBlock> for NetworkActor {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        let NewHeadBlock(block) = msg;
        info!("broadcast a new block {:?}", block.header().id());

        let id = block.header().id();
        let peers = self.peers.clone();

        let network_service = self.network_service.clone();

        let block_hash = block.header().id();
        let block_number = block.header().number();

        let total_difficulty = block.get_total_difficulty();
        let msg = PeerMessage::Block(block);
        let bytes = msg.encode().expect("should encode succ");

        let self_info = PeerInfo::new(
            self.peer_id.clone().into(),
            block_number,
            total_difficulty,
            block_hash,
        );
        let self_id = self.peer_id.clone();
        Arbiter::spawn(async move {
            if let Some(peer_info) = peers.lock().await.get_mut(&self_id) {
                debug!(
                    "total_difficulty is {},peer_info is {:?}",
                    total_difficulty, peer_info
                );
                if total_difficulty > peer_info.peer_info.total_difficulty {
                    peer_info.peer_info.block_number = block_number;
                    peer_info.peer_info.block_id = block_hash;
                    peer_info.peer_info.total_difficulty = total_difficulty;
                }
            }

            for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                if !peer_info.known_blocks.contains(&id) {
                    peer_info.known_blocks.put(id, ());
                } else {
                    continue;
                }

                network_service
                    .send_message(peer_id.clone(), bytes.clone())
                    .await
                    .expect("send message failed ,check network service please");
            }
        });

        self.network_service.update_self_info(self_info);
    }
}

/// handle txn relay
impl Handler<PropagateNewTransactions> for NetworkActor {
    type Result = <PropagateNewTransactions as Message>::Result;

    fn handle(&mut self, msg: PropagateNewTransactions, _ctx: &mut Self::Context) -> Self::Result {
        let txns = msg.transactions_to_propagate();

        // false positive
        if txns.is_empty() {
            return;
        }
        info!("propagate new txns, len: {}", txns.len());

        let peers = self.peers.clone();
        let network_service = self.network_service.clone();
        let mut txn_map: HashMap<HashValue, SignedUserTransaction> = HashMap::new();
        for txn in txns {
            txn_map.insert(txn.crypto_hash(), txn);
        }
        let self_peer_id = self.peer_id.clone();
        Arbiter::spawn(async move {
            for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                let mut txns_unhandled = Vec::new();
                for (id, txn) in &txn_map {
                    if !peer_info.known_transactions.contains(id) && !peer_id.eq(&self_peer_id) {
                        peer_info.known_transactions.put(*id, ());
                        txns_unhandled.push(txn.clone());
                    }
                }

                let msg = PeerMessage::UserTransactions(txns_unhandled);

                let bytes = msg.encode().expect("encode should succ");
                network_service
                    .send_message(peer_id.clone(), bytes)
                    .await
                    .expect("check network service");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bus::Subscription;
    use futures::sink::SinkExt;
    use futures_timer::Delay;
    use network_p2p::Multiaddr;
    use serde::{Deserialize, Serialize};
    use tokio::runtime::{Handle, Runtime};
    use tokio::task;
    use types::transaction::SignedUserTransaction;

    #[rtype(result = "Result<()>")]
    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Message, Clone)]
    pub struct TestRequest {
        pub data: HashValue,
    }

    #[test]
    fn test_peer_info() {
        let mut peer_info = PeerInfo::default();
        peer_info.block_number = 1;
        let data = peer_info.encode().unwrap();
        let peer_info_decode = PeerInfo::decode(&data).unwrap();
        assert_eq!(peer_info, peer_info_decode);
    }

    #[ignore]
    #[stest::test]
    fn test_network_with_mock() {
        use std::time::Duration;

        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let local = task::LocalSet::new();
        let future = System::run_in_tokio("test", &local);

        let mut node_config1 = NodeConfig::random_for_test();
        node_config1.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port())
                .parse()
                .unwrap();
        let node_config1 = Arc::new(node_config1);

        let (network1, _bus1) = build_network(node_config1.clone(), handle.clone());

        let mut node_config2 = NodeConfig::random_for_test();
        let addr1_hex = network1.peer_id.to_base58();
        let seed: Multiaddr = format!("{}/p2p/{}", &node_config1.network.listen, addr1_hex)
            .parse()
            .unwrap();
        node_config2.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port())
                .parse()
                .unwrap();
        node_config2.network.seeds = vec![seed];
        let node_config2 = Arc::new(node_config2);

        let (network2, bus2) = build_network(node_config2, handle);

        Arbiter::spawn(async move {
            let network_clone2 = network2.clone();

            let (tx, mut rx) = mpsc::unbounded();
            let response_actor = TestResponseActor::create(network_clone2, tx);
            let addr = response_actor.start();

            let recipient = addr.clone().recipient::<RawRpcRequestMessage>();
            bus2.send(Subscription { recipient }).await.unwrap();

            let recipient = addr.clone().recipient::<PeerEvent>();
            bus2.send(Subscription { recipient }).await.unwrap();

            // subscribe peer txns for network2
            bus2.send(Subscription {
                recipient: addr.clone().recipient::<PeerTransactions>(),
            })
            .await
            .unwrap();

            network1
                .network_actor_addr()
                .send(PropagateNewTransactions::from(vec![
                    SignedUserTransaction::mock(),
                ]))
                .await
                .unwrap();

            network2
                .network_actor_addr()
                .send(PropagateNewTransactions::from(vec![
                    SignedUserTransaction::mock(),
                ]))
                .await
                .unwrap();

            let _ = rx.next().await;
            let txns = addr.send(GetPeerTransactions).await.unwrap();
            assert_eq!(1, txns.len());

            let request = TestRequest {
                data: HashValue::random(),
            };
            info!("req :{:?}", request);
            let resp = network1
                .send_request_bytes(
                    network2.identify().clone(),
                    request.encode().unwrap(),
                    Duration::from_secs(1),
                )
                .await;
            info!("resp :{:?}", resp);

            _delay(Duration::from_millis(100)).await;

            System::current().stop();
        });

        local.block_on(&mut rt, future).unwrap();
    }

    async fn _delay(duration: Duration) {
        Delay::new(duration).await;
    }

    fn build_network(
        node_config: Arc<NodeConfig>,
        handle: Handle,
    ) -> (NetworkAsyncService, Addr<BusActor>) {
        let bus = BusActor::launch();
        let network = NetworkActor::launch(
            node_config,
            bus.clone(),
            handle,
            HashValue::default(),
            PeerInfo::default(),
        );
        (network, bus)
    }

    struct TestResponseActor {
        _network_service: NetworkAsyncService,
        peer_txns: Vec<PeerTransactions>,
        event_tx: mpsc::UnboundedSender<()>,
    }

    impl TestResponseActor {
        fn create(
            network_service: NetworkAsyncService,
            event_tx: mpsc::UnboundedSender<()>,
        ) -> TestResponseActor {
            Self {
                _network_service: network_service,
                peer_txns: vec![],
                event_tx,
            }
        }
    }

    impl Actor for TestResponseActor {
        type Context = Context<Self>;

        fn started(&mut self, _ctx: &mut Self::Context) {
            info!("Test actor started ",);
        }
    }

    impl Handler<PeerTransactions> for TestResponseActor {
        type Result = ();

        fn handle(&mut self, msg: PeerTransactions, _ctx: &mut Self::Context) -> Self::Result {
            self.peer_txns.push(msg);
            self.event_tx.unbounded_send(()).unwrap();
        }
    }

    struct GetPeerTransactions;

    impl Message for GetPeerTransactions {
        type Result = Vec<PeerTransactions>;
    }

    impl Handler<GetPeerTransactions> for TestResponseActor {
        type Result = MessageResult<GetPeerTransactions>;

        fn handle(&mut self, _msg: GetPeerTransactions, _ctx: &mut Self::Context) -> Self::Result {
            MessageResult(self.peer_txns.clone())
        }
    }

    impl Handler<RawRpcRequestMessage> for TestResponseActor {
        type Result = Result<()>;

        fn handle(&mut self, msg: RawRpcRequestMessage, ctx: &mut Self::Context) -> Self::Result {
            let mut responder = msg.responder.clone();
            let f = async move {
                responder.send(msg.request).await.unwrap();
            };
            let f = actix::fut::wrap_future(f);
            ctx.spawn(Box::new(f));
            Ok(())
        }
    }

    impl Handler<PeerEvent> for TestResponseActor {
        type Result = Result<()>;

        fn handle(&mut self, msg: PeerEvent, _ctx: &mut Self::Context) -> Self::Result {
            info!("Event is {:?}", msg);
            Ok(())
        }
    }
}
