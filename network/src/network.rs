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
use futures::{channel::mpsc, stream::StreamExt};
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

use crypto::{hash::CryptoHash, HashValue};
use std::collections::HashMap;
use types::transaction::SignedUserTransaction;

const LRU_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct NetworkAsyncService {
    addr: Addr<NetworkActor>,
    message_processor: MessageProcessor<u128, RPCResponse>,
    tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_id: PeerId,
    handle: Handle,
    inner: Arc<Inner>,
}

struct Inner {
    network_service: SNetworkService,
    bus: Addr<BusActor>,
    message_processor: MessageProcessor<u128, RPCResponse>,
    handle: Handle,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
}

struct PeerInfoNet {
    pub protocol_version: u32,
    pub best_number: u64,
    known_transactions: LruCache<HashValue, ()>,
    /// Holds a set of blocks known to this peer.
    known_blocks: LruCache<HashValue, ()>,
}

impl PeerInfoNet {
    fn new() -> Self {
        Self {
            protocol_version: 0,
            best_number: 0,
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

    #[cfg(test)]
    pub fn network_actor_addr(&self) -> Addr<NetworkActor> {
        self.addr.clone()
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
}

pub struct NetworkActor {
    network_service: SNetworkService,
    bus: Addr<BusActor>,
    peers: Arc<Mutex<HashMap<PeerId, PeerInfoNet>>>,
}

impl NetworkActor {
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        handle: Handle,
    ) -> NetworkAsyncService {
        let (service, tx, rx, event_rx, tx_command) =
            build_network_service(&node_config.network, handle.clone());
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
        let peer_id = service.identify().clone();

        let service_clone = service.clone();
        let bus_clone = bus.clone();
        let peers = Arc::new(Mutex::new(HashMap::new()));
        let peers_clone = peers.clone();
        let addr = NetworkActor::create(move |_ctx: &mut Context<NetworkActor>| NetworkActor {
            network_service: service_clone,
            bus: bus_clone,
            peers: peers_clone,
        });
        let inner = Inner {
            network_service: service,
            bus,
            handle: handle.clone(),
            message_processor: message_processor_clone,
            peers,
        };
        let inner = Arc::new(inner);
        handle.spawn(Self::start(inner.clone(), rx, event_rx, tx_command));
        NetworkAsyncService {
            addr,
            message_processor,
            tx,
            peer_id,
            inner,
            handle,
        }
    }

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
                    inner.handle_network_receive(message).await.unwrap();
                    info!("receive net message");
                },
                event = event_rx.select_next_some()=>{
                    inner.handle_event_receive(event).await.unwrap();
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
    async fn handle_network_receive(&self, network_msg: NetworkMessage) -> Result<()> {
        info!("receive network_message ");
        let message = PeerMessage::decode(&network_msg.data);
        match message {
            Ok(msg) => {
                self.handle_network_message(network_msg.peer_id.into(), msg)
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
                let _peer_info = PeerInfo::new(peer_id.into());
                self.bus
                    .clone()
                    .broadcast(PeerTransactions::new(txns))
                    .await?;
            }
            PeerMessage::Block(block) => {
                let peer_info = PeerInfo::new(peer_id.into());
                self.bus
                    .send(Broadcast {
                        msg: SyncMessage::DownloadMessage(DownloadMessage::NewHeadBlock(
                            peer_info, block,
                        )),
                    })
                    .await?;
            }
            PeerMessage::LatestStateMsg(state) => {
                info!("broadcast LatestStateMsg.");
                let peer_info = PeerInfo::new(peer_id.into());
                self.bus
                    .send(Broadcast {
                        msg: SyncMessage::DownloadMessage(DownloadMessage::LatestStateMsg(
                            peer_info, state,
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
        }
        Ok(())
    }

    async fn handle_event_receive(&self, event: PeerEvent) -> Result<()> {
        info!("event is {:?}", event);
        match event.clone() {
            PeerEvent::Open(peer_id) => {
                self.on_peer_connected(peer_id.into()).await;
            }
            PeerEvent::Close(peer_id) => {
                self.on_peer_disconnected(peer_id.into()).await;
            }
        }
        self.bus.send(Broadcast { msg: event }).await?;
        info!("already broadcast event");
        Ok(())
    }

    async fn on_peer_connected(&self, peer_id: PeerId) {
        let mut peers = self.peers.lock().await;
        if !peers.contains_key(&peer_id) {
            peers.insert(peer_id, PeerInfoNet::new());
        };
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
                let msg = PeerMessage::Block(block);
                let bytes = msg.encode().unwrap();

                Arbiter::spawn(async move {
                    for (peer_id, peer_info) in peers.lock().await.iter_mut() {
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
    fn test_network_with_mock() {
        use std::thread;
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

        thread::sleep(Duration::from_secs(1));

        let mut node_config2 = NodeConfig::random_for_test();
        let addr1_hex = network1.peer_id.to_base58();
        let seed = format!("{}/p2p/{}", &node_config1.network.listen, addr1_hex);
        node_config2.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        node_config2.network.seeds = vec![seed];
        let node_config2 = Arc::new(node_config2);

        let (network2, _addr2, bus2) = build_network(node_config2.clone(), handle.clone());

        thread::sleep(Duration::from_secs(1));

        Arbiter::spawn(async move {
            let network_clone2 = network2.clone();

            let response_actor = TestResponseActor::create(network_clone2);
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

            _delay(Duration::from_millis(100)).await;

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

            _delay(Duration::from_millis(100)).await;

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
        let network = NetworkActor::launch(node_config.clone(), bus.clone(), handle);
        (network, addr, bus)
    }

    struct TestResponseActor {
        _network_service: NetworkAsyncService,
        peer_txns: Vec<PeerTransactions>,
    }

    impl TestResponseActor {
        fn create(network_service: NetworkAsyncService) -> TestResponseActor {
            let instance = Self {
                _network_service: network_service,
                peer_txns: vec![],
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
