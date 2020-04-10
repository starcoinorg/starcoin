// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message_processor::{MessageFuture, MessageProcessor};
use crate::net::{build_network_service, SNetworkService};
use crate::{NetworkMessage, PeerEvent, PeerMessage, RPCRequest, RPCResponse, RpcRequestMessage};
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, Bus, BusActor};
use config::NodeConfig;
use futures::lock::Mutex;
use futures::{channel::mpsc, sink::SinkExt, stream::StreamExt};
use libp2p::PeerId;
use network_p2p_api::sync_messages::{DownloadMessage, SyncMessage};
use scs::SCSCodec;
use std::sync::Arc;
use tx_relay::*;
use types::peer_info::PeerInfo;
use types::system_events::SystemEvents;

use crate::helper::get_unix_ts;
use futures_timer::Delay;
use lru::LruCache;
use std::time::Duration;
use tokio::runtime::Handle;

use bitflags::_core::sync::atomic::Ordering;
use crypto::{hash::CryptoHash, HashValue};
use network_p2p_api::messages::RawRpcRequestMessage;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use types::transaction::SignedUserTransaction;

const LRU_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct NetworkAsyncService {
    addr: Addr<NetworkActor>,
    message_processor: MessageProcessor<u128, RPCResponse>,
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_id: PeerId,
    handle: Handle,
    inner: Arc<Inner>,
}

struct Inner {
    network_service: SNetworkService,
    bus: Addr<BusActor>,
    message_processor: MessageProcessor<u128, RPCResponse>,
    raw_message_processor: MessageProcessor<u128, Vec<u8>>,
    handle: Handle,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
    connected_tx: mpsc::Sender<PeerEvent>,
    need_send_event: AtomicBool,
}

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

impl NetworkAsyncService {
    pub async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()> {
        let data = msg.encode()?;
        let network_message = NetworkMessage {
            peer_id: peer_id.into(),
            data,
        };
        self.tx.unbounded_send(network_message)?;

        Ok(())
    }

    pub async fn broadcast_system_event(&self, event: SystemEvents) -> Result<()> {
        self.addr.send(event).await?;
        Ok(())
    }

    pub async fn send_request(
        &self,
        peer_id: PeerId,
        message: RPCRequest,
        time_out: Duration,
    ) -> Result<RPCResponse> {
        let request_id = get_unix_ts();
        let peer_msg = PeerMessage::RPCRequest(request_id, message);
        let data = peer_msg.encode()?;
        let network_message = NetworkMessage {
            peer_id: peer_id.clone().into(),
            data,
        };
        self.tx.unbounded_send(network_message)?;
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let message_future = MessageFuture::new(rx);
        self.message_processor.add_future(request_id, tx).await;
        info!("send request to {} with id {}", peer_id, request_id);
        let processor = self.message_processor.clone();
        let peer_id_clone = peer_id.clone();
        let task = async move {
            Delay::new(time_out).await;
            processor.remove_future(request_id).await;
            warn!(
                "send request to {} with id {} timeout",
                peer_id_clone, request_id
            );
        };

        self.handle.spawn(task);
        let response = message_future.await;
        info!("receive response from {} with id {}", peer_id, request_id);
        response
    }

    pub fn identify(&self) -> &PeerId {
        &self.peer_id
    }

    pub async fn send_request_bytes(
        &self,
        peer_id: PeerId,
        message: Vec<u8>,
        time_out: Duration,
    ) -> Result<Vec<u8>> {
        let request_id = get_unix_ts();
        let peer_msg = PeerMessage::RawRPCRequest(request_id, message);
        let data = peer_msg.encode()?;
        let network_message = NetworkMessage {
            peer_id: peer_id.clone().into(),
            data,
        };
        self.tx.unbounded_send(network_message)?;
        let (tx, rx) = futures::channel::mpsc::channel(1);
        let message_future = MessageFuture::new(rx);
        self.raw_message_processor.add_future(request_id, tx).await;
        info!("send request to {} with id {}", peer_id, request_id);
        let processor = self.message_processor.clone();
        let peer_id_clone = peer_id.clone();
        let task = async move {
            Delay::new(time_out).await;
            processor.remove_future(request_id).await;
            warn!(
                "send request to {} with id {} timeout",
                peer_id_clone, request_id
            );
        };

        self.handle.spawn(task);
        let response = message_future.await;
        info!("receive response from {} with id {}", peer_id, request_id);
        response
    }

    pub async fn peer_set(&self) -> Result<HashSet<PeerInfo>> {
        let mut result = HashSet::new();

        for (_, peer) in self.inner.peers.lock().await.iter() {
            result.insert(peer.peer_info.clone());
        }
        Ok(result)
    }

    pub async fn get_peer(&self, peer_id: &PeerId) -> Result<Option<PeerInfo>> {
        match self.inner.peers.lock().await.get(peer_id) {
            Some(peer) => Ok(Some(peer.peer_info.clone())),
            None => Ok(None),
        }
    }

    pub async fn best_peer(&self) -> Result<Option<PeerInfo>> {
        let size = self.inner.peers.lock().await.len();
        if size == 0 {
            return Ok(None);
        }
        let mut info = PeerInfo::default();
        for (_, peer) in self.inner.peers.lock().await.iter() {
            if peer.peer_info.total_difficult > info.total_difficult {
                info = peer.peer_info.clone();
            }
        }
        Ok(Some(info))
    }

    pub async fn get_peer_set_size(&self) -> Result<usize> {
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
                .fold(String::new(), |acc, arg| acc + arg.as_str()),
            service.identify()
        );
        let message_processor = MessageProcessor::new();
        let message_processor_clone = message_processor.clone();

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
        if node_config.network.seeds.len() > 0 {
            need_send_event.swap(true, Ordering::Acquire);
        }
        let inner = Inner {
            network_service: service,
            bus,
            handle: handle.clone(),
            message_processor: message_processor_clone,
            raw_message_processor: raw_message_processor_clone,
            peers,
            connected_tx,
            need_send_event,
        };
        let inner = Arc::new(inner);
        handle.spawn(Self::start(
            handle.clone(),
            inner.clone(),
            rx,
            event_rx,
            tx_command,
        ));

        if node_config.network.seeds.len() > 0 {
            futures::executor::block_on(async move {
                let event = connected_rx.next().await.unwrap();
                info!("receive event {:?}", event);
            });
        }

        NetworkAsyncService {
            addr,
            message_processor,
            raw_message_processor,
            tx,
            peer_id,
            inner,
            handle,
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
                    .handle_network_message(network_msg.peer_id.into(), msg)
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
                    if total_difficulty > peer_info.peer_info.total_difficult {
                        peer_info.peer_info.block_number = block_number;
                        peer_info.peer_info.block_id = block_hash;
                        peer_info.peer_info.total_difficult = total_difficulty;
                    }
                }
                self.bus
                    .send(Broadcast {
                        msg: SyncMessage::DownloadMessage(DownloadMessage::NewHeadBlock(
                            peer_id.into(),
                            block.get_block().clone(),
                        )),
                    })
                    .await?;
            }
            PeerMessage::RPCRequest(id, request) => {
                info!("do request.");
                let (tx, mut rx) = mpsc::channel(1);
                self.bus
                    .send(Broadcast {
                        msg: RpcRequestMessage {
                            responder: tx,
                            request,
                        },
                    })
                    .await?;
                let network_service = self.network_service.clone();
                let task = async move {
                    let response = rx.next().await.unwrap();
                    let peer_msg = PeerMessage::RPCResponse(id, response);
                    let data = peer_msg.encode().unwrap();
                    network_service.send_message(peer_id, data).await.unwrap();
                };
                self.handle.spawn(task);
                info!("receive rpc request");
            }
            PeerMessage::RPCResponse(id, response) => {
                info!("do response.");
                self.message_processor.send_response(id, response).await?;
            }
            PeerMessage::RawRPCRequest(id, request) => {
                info!("do request.");
                let (tx, mut rx) = mpsc::channel(1);
                self.bus
                    .send(Broadcast {
                        msg: RawRpcRequestMessage {
                            responder: tx,
                            request,
                        },
                    })
                    .await?;
                let network_service = self.network_service.clone();
                let task = async move {
                    let response = rx.next().await.unwrap();
                    let peer_msg = PeerMessage::RawRPCResponse(id, response);
                    let data = peer_msg.encode().unwrap();
                    network_service.send_message(peer_id, data).await.unwrap();
                };
                self.handle.spawn(task);
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

    async fn handle_event_receive(inner: Arc<Inner>, event: PeerEvent) -> Result<()> {
        info!("event is {:?}", event);
        match event.clone() {
            PeerEvent::Open(peer_id, peer_info) => {
                inner.on_peer_connected(peer_id.into(), peer_info).await;
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

    async fn on_peer_connected(&self, peer_id: PeerId, peer_info: PeerInfo) {
        self.peers
            .lock()
            .await
            .entry(peer_id.clone())
            .or_insert(PeerInfoNet::new(peer_info));
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
impl Handler<SystemEvents> for NetworkActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewHeadBlock(block) => {
                info!("broadcast a new block {:?}", block.header().id());

                let id = block.header().id();
                let peers = self.peers.clone();

                let network_service = self.network_service.clone();

                let block_hash = block.header().id();
                let block_number = block.header().number();

                let total_difficulty = block.get_total_difficulty();
                let msg = PeerMessage::Block(block.clone());
                let bytes = msg.encode().unwrap();

                let self_info = PeerInfo::new(
                    self.peer_id.clone().into(),
                    block_number,
                    total_difficulty,
                    block_hash,
                );

                Arbiter::spawn(async move {
                    for (peer_id, mut peer_info) in peers.lock().await.iter_mut() {
                        info!("send block to peer {}", peer_id);
                        peer_info.peer_info.block_number = block_number;
                        peer_info.peer_info.block_id = block_hash;
                        peer_info.peer_info.total_difficult = total_difficulty;

                        if !peer_info.known_blocks.contains(&id) {
                            peer_info.known_blocks.put(id.clone(), ());
                        } else {
                            continue;
                        }

                        network_service
                            .send_message(peer_id.clone(), bytes.clone())
                            .await
                            .unwrap();
                    }
                });

                self.network_service.update_self_info(self_info);

                ()
            }
            _ => (),
        };

        ()
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
        Arbiter::spawn(async move {
            for (peer_id, peer_info) in peers.lock().await.iter_mut() {
                let mut txns_unhandled = Vec::new();
                for (id, txn) in &txn_map {
                    if !peer_info.known_transactions.contains(id) {
                        peer_info.known_transactions.put(id.clone(), ());
                        txns_unhandled.push(txn.clone());
                    }
                }

                let msg = PeerMessage::UserTransactions(txns_unhandled);

                let bytes = msg.encode().unwrap();
                network_service
                    .send_message(peer_id.clone(), bytes)
                    .await
                    .unwrap();
            }
        });

        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RpcRequestMessage, TestRequest, TestResponse};
    use bus::Subscription;
    use futures::sink::SinkExt;
    use futures_timer::Delay;
    use tokio::runtime::{Handle, Runtime};
    use tokio::task;
    use types::account_address::AccountAddress;
    use types::transaction::SignedUserTransaction;

    #[test]
    fn test_peer_info() {
        let mut peer_info = PeerInfo::default();
        peer_info.block_number = 1;
        let data = peer_info.encode().unwrap();
        let peer_info_decode = PeerInfo::decode(&data).unwrap();
        assert_eq!(peer_info, peer_info_decode);
    }

    #[test]
    fn test_network_with_mock() {
        use std::time::Duration;

        ::logger::init_for_test();

        let mut rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let local = task::LocalSet::new();
        let future = System::run_in_tokio("test", &local);

        let mut node_config1 = NodeConfig::random_for_test();
        node_config1.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        let node_config1 = Arc::new(node_config1);

        let (network1, _addr1, _bus1) = build_network(node_config1.clone(), handle.clone());

        let mut node_config2 = NodeConfig::random_for_test();
        let addr1_hex = network1.peer_id.to_base58();
        let seed = format!("{}/p2p/{}", &node_config1.network.listen, addr1_hex);
        node_config2.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        node_config2.network.seeds = vec![seed];
        let node_config2 = Arc::new(node_config2);

        let (network2, _addr2, bus2) = build_network(node_config2.clone(), handle.clone());

        Arbiter::spawn(async move {
            let network_clone2 = network2.clone();

            let (tx, mut rx) = mpsc::unbounded();
            let response_actor = TestResponseActor::create(network_clone2, tx);
            let addr = response_actor.start();

            let recipient = addr.clone().recipient::<RpcRequestMessage>();
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

            let request = RPCRequest::TestRequest(TestRequest {
                data: HashValue::random(),
            });
            info!("req :{:?}", request);
            let resp = network1
                .send_request(network2.identify().clone(), request, Duration::from_secs(1))
                .await;
            info!("resp :{:?}", resp);

            _delay(Duration::from_millis(100)).await;

            System::current().stop();

            ()
        });

        local.block_on(&mut rt, future).unwrap();
    }

    async fn _delay(duration: Duration) {
        Delay::new(duration).await;
    }

    fn build_network(
        node_config: Arc<NodeConfig>,
        handle: Handle,
    ) -> (NetworkAsyncService, AccountAddress, Addr<BusActor>) {
        let bus = BusActor::launch();
        let addr =
            AccountAddress::from_public_key(&node_config.network.network_keypair().public_key);
        let network = NetworkActor::launch(
            node_config.clone(),
            bus.clone(),
            handle,
            HashValue::default(),
            PeerInfo::default(),
        );
        (network, addr, bus)
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
            let instance = Self {
                _network_service: network_service,
                peer_txns: vec![],
                event_tx,
            };
            instance
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

    impl Handler<RpcRequestMessage> for TestResponseActor {
        type Result = Result<()>;

        fn handle(&mut self, msg: RpcRequestMessage, ctx: &mut Self::Context) -> Self::Result {
            let mut responder = msg.responder.clone();
            match msg.request {
                RPCRequest::TestRequest(_r) => {
                    info!("request is {:?}", _r);
                    let response = TestResponse {
                        len: 1,
                        id: _r.data,
                    };
                    let f = async move {
                        responder
                            .send(RPCResponse::TestResponse(response))
                            .await
                            .unwrap();
                    };
                    let f = actix::fut::wrap_future(f);
                    ctx.spawn(Box::new(f));
                    Ok(())
                }
                _ => Ok(()),
            }
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
