// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::net::{build_network_service, NetworkService};
use crate::{GetCounterMessage, NetworkMessage, PeerMessage, RPCMessage, RPCRequest, RPCResponse, RpcRequestMessage};
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::{NetworkConfig, NodeConfig};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::{test_utils::KeyPair, Uniform};
use futures_03::{compat::{Stream01CompatExt,Future01CompatExt}, TryFutureExt};
use libp2p::{
    identity,
    ping::{Ping, PingConfig, PingEvent},
    PeerId, Swarm,
};
use scs::SCSCodec;
use std::sync::Arc;
use traits::TxPoolAsyncService;
use txpool::{AddTransaction, TxPoolActor};
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};
use crate::message_processor::{MessageProcessor, MessageFuture};
use types::account_address::AccountAddress;
use crypto::hash::HashValue;
use futures::{
    stream::Stream,
    sync::{mpsc, oneshot},
};
use std::time::Duration;
use actix::fut::wrap_future;

#[derive(Clone)]
pub struct NetworkAsyncService<P>
    where
        P: TxPoolAsyncService,
        P: 'static,
{
    addr: Addr<NetworkActor<P>>,
    message_processor:MessageProcessor<RPCResponse>,
    tx: mpsc::UnboundedSender<NetworkMessage>,
}

impl<P> NetworkAsyncService<P>
    where
        P: TxPoolAsyncService,
        P: 'static,
{
    pub async fn send_peer_message(&self,peer_id:AccountAddress, msg: PeerMessage) -> Result<()>{
        let data = msg.encode().unwrap();
        let network_message = NetworkMessage{
            peer_id,
            data
        };
        self.tx.unbounded_send(network_message)?;

        Ok(())
    }

    pub async fn broadcast_system_event(&self,event: SystemEvents) -> Result<()>{
        self.addr.send(event).await;
        Ok(())
    }

    pub async fn send_request(
        &self,
        peer_id:AccountAddress,
        message:RPCRequest,
        _time_out:Duration,
    ) -> Result<RPCResponse>{
        let request_id=message.get_id();
        let peer_msg = PeerMessage::RPCRequest(message);
        let data = peer_msg.encode().unwrap();
        let network_message = NetworkMessage{
            peer_id,
            data
        };
        self.tx.unbounded_send(network_message)?;
        let (tx, rx) = futures::sync::mpsc::channel(1);
        let message_future = MessageFuture::new(rx);
        self.message_processor.add_future(request_id,tx);
        info!("send request to {}",peer_id);
        message_future.compat().await
    }

    pub async fn response_for(&self,peer_id:AccountAddress,
                          id: HashValue,mut response:RPCResponse)->Result<()>{
        response.set_request_id(id);
        let peer_msg = PeerMessage::RPCResponse(response);
        let data = peer_msg.encode().unwrap();
        let network_message = NetworkMessage{
            peer_id,
            data
        };
        self.tx.unbounded_send(network_message)?;
        Ok(())
    }

}

pub struct NetworkActor<P>
    where
        P: TxPoolAsyncService,
        P: 'static,
{
    network_service: NetworkService,
    tx: mpsc::UnboundedSender<NetworkMessage>,
    tx_command: oneshot::Sender<()>,
    bus: Addr<BusActor>,
    txpool: P,
    message_processor:MessageProcessor<RPCResponse>,
}

impl<P> NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        txpool: P,
        key_pair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    ) -> NetworkAsyncService<P>{

        let (service, tx, rx, tx_command) = build_network_service(&node_config.network, key_pair);
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
        let tx_clone = tx.clone();
        let addr=NetworkActor::create(
            move |ctx: &mut Context<NetworkActor<P>>| {
                ctx.add_stream(rx.fuse().compat());
                NetworkActor {
                    network_service: service,
                    tx:tx_clone,
                    tx_command,
                    bus,
                    txpool,
                    message_processor:message_processor_clone,
                }
            },
        );
        NetworkAsyncService{addr,message_processor,tx}
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

impl<P> StreamHandler<Result<NetworkMessage, ()>> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    fn handle(&mut self, item: Result<NetworkMessage, ()>, ctx: &mut Self::Context) {
        match item {
            Ok(network_msg) => {
                info!("receive network_message {:?}", network_msg);
                let message = PeerMessage::decode(&network_msg.data);
                match message {
                    Ok(msg) => {
                        self.handle_network_message(network_msg.peer_id,msg,ctx);
                    }
                    Err(e) => {
                        warn!("get error {:?}", e);
                    }
                }
            }
            Err(e) => {
                warn!("get error {:?}", e);
            }
        }
    }
}

impl<P> NetworkActor<P>
    where
        P: TxPoolAsyncService,
{
    fn handle_network_message(&self,peer_id:AccountAddress,msg:PeerMessage, ctx: &mut Context<Self>){
        match msg {
            PeerMessage::UserTransaction(txn) => {
                let txpool=self.txpool.clone();
                let f = async move{
                    let new_txn=txpool.add(txn).await.unwrap();
                    info!("add tx success, is new tx: {}", new_txn);
                };
                let f = actix::fut::wrap_future(f);
                ctx.spawn(Box::new(f));
            },
            PeerMessage::RPCRequest(request)=>{
                let bus = self.bus.clone();
                let f= async move {
                    bus.send(Broadcast{msg:RpcRequestMessage{
                        peer_id,
                        request,
                    }}).await;
                    info!("receive rpc request");
                };
                let f = actix::fut::wrap_future(f);
                ctx.spawn(Box::new(f));
            },
            PeerMessage::RPCResponse(response)=>{
                let message_processor = self.message_processor.clone();
                let f = async move {
                    let id=response.get_id();
                    message_processor.send_response(id,response).unwrap();
                };
                let f = actix::fut::wrap_future(f);
                ctx.spawn(Box::new(f));
            },
        }
    }
}

/// handler system events.
impl<P> Handler<SystemEvents> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewUserTransaction(txn) => {
                debug!("new user transaction {:?}", txn);
                let peer_msg = PeerMessage::UserTransaction(txn);
                let bytes = peer_msg.encode().unwrap();
                self.network_service.broadcast_message(bytes);
            }
            SystemEvents::NewHeadBlock(block) => {
                //TODO broadcast block to peers.
            }
            _ => {}
        }
    }
}

// /// Handler for receive broadcast from other peer.
// impl<P> Handler<PeerMessage> for NetworkActor<P>
// where
//     P: TxPoolAsyncService,
// {
//     type Result = ResponseActFuture<Self, Result<()>>;
//
//     fn handle(&mut self, msg: PeerMessage, _ctx: &mut Self::Context) -> Self::Result {
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use traits::mock::MockTxPoolService;
    use txpool::TxPoolActor;
    use types::account_address::AccountAddress;
    use futures_timer::Delay;
    use log::{Level, Metadata, Record};
    use log::{LevelFilter, SetLoggerError};
    use std::time::Instant;
    use futures::future::IntoFuture;
    use crate::{TestRequest, RpcRequestMessage,TestResponse};
    use bus::{Subscription};

    struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Info
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                println!("{} - {}", record.level(), record.args());
            }
        }

        fn flush(&self) {}
    }

    static LOGGER: SimpleLogger = SimpleLogger;

    fn init_log() -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
    }
    // #[actix_rt::test]
    // async fn test_network() {
    //     let node_config = NodeConfig::default();
    //     let bus = BusActor::launch();
    //     let txpool = TxPoolActor::launch(&node_config, bus.clone()).unwrap();
    //     let network = NetworkActor::launch(&node_config, bus, txpool.clone()).unwrap();
    //     network
    //         .send(PeerMessage::UserTransaction(SignedUserTransaction::mock()))
    //         .await
    //         .unwrap();
    //
    //     let txns = txpool.get_pending_txns().await.unwrap();
    //     assert_eq!(1, txns.len());
    // }


    #[test]
    fn test_network_with_mock() {
        let mut system = System::new("test");

        init_log();

        let mut node_config1 = NodeConfig::default();
        node_config1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config1 = Arc::new(node_config1);

        let (txpool1, network1, addr1,bus1) = build_network(node_config1.clone());

        let mut node_config2 = NodeConfig::default();
        let addr1_hex = hex::encode(addr1);
        let seed = format!("{}/p2p/{}", &node_config1.network.listen, addr1_hex);
        node_config2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        node_config2.network.seeds = vec![seed];
        let node_config2 = Arc::new(node_config2);

        let (txpool2, network2, addr2,bus2) = build_network(node_config2.clone());

        use std::thread;
        use std::time::Duration;

        thread::sleep(Duration::from_millis(1000));

        Arbiter::spawn(async move {

            network1.broadcast_system_event(SystemEvents::NewUserTransaction(
                SignedUserTransaction::mock(),
            )).await;
            network2.broadcast_system_event(SystemEvents::NewUserTransaction(
                SignedUserTransaction::mock(),
            )).await;

            let network_clone2 = network2.clone();

            let response_actor = TestResponseActor::create(network_clone2);
            let addr=response_actor.start();

            let recipient = addr.recipient::<RpcRequestMessage>();
            bus2.send(Subscription { recipient }).await;

            _delay(Duration::from_millis(100)).await;

            let txns = txpool1.get_pending_txns(None).await.unwrap();
            //assert_eq!(1, txns.len());

            let txns = txpool2.get_pending_txns(None).await.unwrap();
            //assert_eq!(1, txns.len());

            let request= RPCRequest::TestRequest(TestRequest{data:HashValue::random()});
            network1.send_request(addr2,request,Duration::from_secs(1)).await;

            _delay(Duration::from_millis(100)).await;

            System::current().stop();
            ()
        });

        system.run();
    }

    async fn _delay(duration: Duration){
        Delay::new(duration).await;
    }

    fn build_network(
        node_config: Arc<NodeConfig>,
    ) -> (
        MockTxPoolService,
        NetworkAsyncService<MockTxPoolService>,
        AccountAddress,
        Addr<BusActor>,
    ) {
        let bus = BusActor::launch();
        let key_pair = gen_keypair();
        let addr = AccountAddress::from_public_key(&key_pair.public_key);
        let txpool = traits::mock::MockTxPoolService::new();
        let network =
            NetworkActor::launch(node_config.clone(), bus.clone(), txpool.clone(), key_pair);
        (txpool, network, addr,bus)
    }

    fn gen_keypair() -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        use rand::prelude::*;

        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng0: StdRng = SeedableRng::from_seed(seed_buf);
        let account_keypair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> =
            Arc::new(KeyPair::generate_for_testing(&mut rng0));
        account_keypair
    }

    fn get_available_port() -> u16 {
        const MAX_PORT_RETRIES: u32 = 1000;

        for _ in 0..MAX_PORT_RETRIES {
            if let Ok(port) = get_ephemeral_port() {
                return port;
            }
        }

        panic!("Error: could not find an available port");
    }

    fn get_ephemeral_port() -> ::std::io::Result<u16> {
        use std::net::{TcpListener, TcpStream};

        // Request a random available port from the OS
        let listener = TcpListener::bind(("localhost", 0))?;
        let addr = listener.local_addr()?;

        // Create and accept a connection (which we'll promptly drop) in order to force the port
        // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
        // amount of time (roughly 60s on some Linux systems)
        let _sender = TcpStream::connect(addr)?;
        let _incoming = listener.accept()?;

        Ok(addr.port())
    }

    struct TestResponseActor {
        network_service:NetworkAsyncService<MockTxPoolService>
    }

    impl TestResponseActor {

        fn create(network_service:NetworkAsyncService<MockTxPoolService>) -> TestResponseActor {
            let instance=Self{
                network_service
            };
            instance
        }
    }

    impl Actor for TestResponseActor
    {
        type Context = Context<Self>;

        fn started(&mut self, _ctx: &mut Self::Context) {
            info!("Test actor started ",);
        }
    }

    impl Handler<RpcRequestMessage> for TestResponseActor
    {
        type Result = Result<()>;

        fn handle(&mut self, msg:RpcRequestMessage, ctx: &mut Self::Context) -> Self::Result {
            let id= (&msg.request).get_id();
            let peer_id = (&msg).peer_id;
            match msg.request {
                RPCRequest::TestRequest(_r)=>{
                    info!("request is {:?}",_r);
                    let response=TestResponse{len:1,id:id.clone()};
                    let network_service=self.network_service.clone();
                    let f=async move {
                        network_service.response_for(peer_id,id,RPCResponse::TestResponse(response)).await;
                    };
                    let f = actix::fut::wrap_future(f);
                    ctx.spawn(Box::new(f));
                    Ok(())
                }
                _ => {
                    Ok(())
                }
            }
        }
    }


}
