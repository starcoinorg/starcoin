// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message_processor::{MessageFuture, MessageProcessor};
use crate::net::{build_network_service, SNetworkService};
use crate::sync_messages::{DownloadMessage, SyncMessage};
use crate::{NetworkMessage, PeerEvent, PeerMessage, RPCRequest, RPCResponse, RpcRequestMessage};
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::hash::CryptoHash;
use futures::{channel::mpsc, stream::StreamExt};
use libp2p::PeerId;
use scs::SCSCodec;
use std::sync::Arc;
use traits::TxPoolAsyncService;
use types::peer_info::PeerInfo;
use types::system_events::SystemEvents;

use crate::helper::get_unix_ts;
use futures_timer::Delay;
use std::time::Duration;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct NetworkAsyncService<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
    addr: Addr<NetworkActor<P>>,
    message_processor: MessageProcessor<u128, RPCResponse>,
    tx: mpsc::UnboundedSender<NetworkMessage>,
    peer_id: PeerId,
    handle: Handle,
    inner: Inner<P>,
}

#[derive(Clone)]
struct Inner<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
    network_service: SNetworkService,
    addr: Addr<NetworkActor<P>>,
    bus: Addr<BusActor>,
    txpool: P,
    message_processor: MessageProcessor<u128, RPCResponse>,
    handle: Handle,
}

impl<P> NetworkAsyncService<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
    pub async fn send_peer_message(&self, peer_id: PeerId, msg: PeerMessage) -> Result<()> {
        let data = msg.encode().unwrap();
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
        let data = peer_msg.encode().unwrap();
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

pub struct NetworkActor<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
    network_service: SNetworkService,
    _txpool: P,
}

impl<P> NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        txpool: P,
        handle: Handle,
    ) -> NetworkAsyncService<P> {
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
        let txpool_clone = txpool.clone();
        let addr = NetworkActor::create(move |_ctx: &mut Context<NetworkActor<P>>| NetworkActor {
            network_service: service_clone,
            _txpool: txpool_clone,
        });
        let inner = Inner {
            addr: addr.clone(),
            network_service: service,
            bus,
            txpool,
            handle: handle.clone(),
            message_processor: message_processor_clone,
        };
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
        inner: Inner<P>,
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

impl<P> Inner<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
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
            PeerMessage::UserTransaction(txn) => {
                let txpool = self.txpool.clone();
                let new_txn = txpool.add(txn).await?;
                info!("add tx success, is new tx: {}", new_txn);
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
        self.bus.send(Broadcast { msg: event }).await?;
        info!("already broadcast event");
        Ok(())
    }
}

impl<P> Actor for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Network actor started ",);
    }
}

/// handler system events.
impl<P> Handler<SystemEvents> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        let peer_msg = match msg {
            SystemEvents::NewUserTransaction(txn) => {
                info!("new user transaction {:?}", txn.crypto_hash());
                Some(PeerMessage::UserTransaction(txn))
            }
            SystemEvents::NewHeadBlock(block) => {
                info!("broadcast a new block {:?}", block.header().id());
                Some(PeerMessage::Block(block))
            }
            _ => None,
        };

        if let Some(msg) = peer_msg {
            let bytes = msg.encode().unwrap();
            let mut network_service = self.network_service.clone();
            Arbiter::spawn(async move {
                network_service.broadcast_message(bytes).await;
            })
        };

        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RpcRequestMessage, TestRequest, TestResponse};
    use bus::Subscription;
    use crypto::HashValue;
    use crypto::HashValue;
    use futures::sink::SinkExt;
    use futures_timer::Delay;
    use tokio::runtime::{Handle, Runtime};
    use traits::mock::MockTxPoolService;
    use types::account_address::AccountAddress;
    use types::transaction::SignedUserTransaction;

    #[test]
    fn test_network_with_mock() {
        use std::thread;
        use std::time::Duration;

        ::logger::init_for_test();
        let system = System::new("test");
        let rt = Runtime::new().unwrap();
        let handle = rt.handle().clone();

        let mut node_config1 = NodeConfig::random_for_test();
        node_config1.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        let node_config1 = Arc::new(node_config1);

        let (txpool1, network1, _addr1, _bus1) =
            build_network(node_config1.clone(), handle.clone());

        thread::sleep(Duration::from_secs(1));

        let mut node_config2 = NodeConfig::random_for_test();
        let addr1_hex = network1.peer_id.to_base58();
        let seed = format!("{}/p2p/{}", &node_config1.network.listen, addr1_hex);
        node_config2.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        node_config2.network.seeds = vec![seed];
        let node_config2 = Arc::new(node_config2);

        let (txpool2, network2, _addr2, bus2) = build_network(node_config2.clone(), handle.clone());

        thread::sleep(Duration::from_secs(1));

        Arbiter::spawn(async move {
            network1
                .broadcast_system_event(SystemEvents::NewUserTransaction(
                    SignedUserTransaction::mock(),
                ))
                .await
                .unwrap();
            network2
                .broadcast_system_event(SystemEvents::NewUserTransaction(
                    SignedUserTransaction::mock(),
                ))
                .await
                .unwrap();

            let network_clone2 = network2.clone();

            let response_actor = TestResponseActor::create(network_clone2);
            let addr = response_actor.start();

            let recipient = addr.clone().recipient::<RpcRequestMessage>();
            bus2.send(Subscription { recipient }).await.unwrap();

            let recipient = addr.recipient::<PeerEvent>();
            bus2.send(Subscription { recipient }).await.unwrap();

            _delay(Duration::from_millis(100)).await;

            let txns = txpool1.get_pending_txns(None).await.unwrap();
            assert_eq!(1, txns.len());

            let txns = txpool2.get_pending_txns(None).await.unwrap();
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

        system.run().unwrap();
    }

    async fn _delay(duration: Duration) {
        Delay::new(duration).await;
    }

    fn build_network(
        node_config: Arc<NodeConfig>,
        handle: Handle,
    ) -> (
        MockTxPoolService,
        NetworkAsyncService<MockTxPoolService>,
        AccountAddress,
        Addr<BusActor>,
    ) {
        let bus = BusActor::launch();
        let addr =
            AccountAddress::from_public_key(&node_config.network.network_keypair().public_key);
        let txpool = traits::mock::MockTxPoolService::new();
        let network =
            NetworkActor::launch(node_config.clone(), bus.clone(), txpool.clone(), handle);
        (txpool, network, addr, bus)
    }

    struct TestResponseActor {
        _network_service: NetworkAsyncService<MockTxPoolService>,
    }

    impl TestResponseActor {
        fn create(network_service: NetworkAsyncService<MockTxPoolService>) -> TestResponseActor {
            let instance = Self {
                _network_service: network_service,
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
