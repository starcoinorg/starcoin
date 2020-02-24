// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{GetCounterMessage, PeerMessage};
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::{NetworkConfig, NodeConfig};
use futures_03::TryFutureExt;
use libp2p::{
    identity,
    ping::{Ping, PingConfig, PingEvent},
    PeerId, Swarm,
};
use std::sync::Arc;
use traits::TxPoolAsyncService;
use txpool::{AddTransaction, TxPoolActor};
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

pub struct NetworkActor<P>
where
    P: TxPoolAsyncService,
    P: 'static,
{
    network_config: NetworkConfig,
    bus: Addr<BusActor>,
    txpool: P,
    //just for test, remove later.
    counter: u64,
}

impl<P> NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        txpool: P,
    ) -> Result<Addr<NetworkActor<P>>> {
        //TODO read from config
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Network peer id: {:?}", peer_id);

        let transport = libp2p::build_development_transport(id_keys)?;

        let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        let network_config = node_config.network.clone();

        Swarm::listen_on(&mut swarm, (&network_config).listen_address.clone())?;
        Ok(NetworkActor::create(
            move |ctx: &mut Context<NetworkActor<P>>| {
                ctx.add_stream(swarm);
                NetworkActor {
                    network_config,
                    bus,
                    txpool,
                    counter: 0,
                }
            },
        ))
    }
}

impl<P> Actor for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!(
            "Network actor started with config: {:?}",
            self.network_config
        );
    }
}

impl<P> StreamHandler<PingEvent> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    fn handle(&mut self, item: PingEvent, _ctx: &mut Self::Context) {
        println!("receive event {:?}", item);
        self.counter += 1;
    }
}

impl<P> Handler<GetCounterMessage> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Result = u64;

    fn handle(&mut self, _msg: GetCounterMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("GetCounterMessage {}", self.counter);
        self.counter
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
                //TODO
                println!("Broadcast transaction {:?} to peers", txn);
            }
            SystemEvents::NewHeadBlock(block) => {
                //TODO broadcast block to peers.
            }
            _ => {}
        }
    }
}

/// Handler for receive broadcast from other peer.
impl<P> Handler<PeerMessage> for NetworkActor<P>
where
    P: TxPoolAsyncService,
{
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: PeerMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PeerMessage::UserTransaction(txn) => {
                let f = self.txpool.clone().add(txn).and_then(|new_txn| async move {
                    println!("add tx success, is new tx: {}", new_txn);
                    Ok(())
                });
                let f = actix::fut::wrap_future(f);
                Box::new(f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::time::{delay_for, Duration};
    use txpool::TxPoolActor;

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

    #[actix_rt::test]
    async fn test_network_with_mock() {
        let node_config = NodeConfig::default();
        let bus = BusActor::launch();
        let txpool = traits::mock::MockTxPoolService::new();
        let network = NetworkActor::launch(&node_config, bus, txpool.clone()).unwrap();
        network
            .send(PeerMessage::UserTransaction(SignedUserTransaction::mock()))
            .await
            .unwrap();

        let txns = txpool.get_pending_txns().await.unwrap();
        assert_eq!(1, txns.len());
    }
}
