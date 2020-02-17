// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{GetCounterMessage, PeerMessage, StopMessage};
use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::{NetworkConfig, NodeConfig};
use libp2p::{
    identity,
    ping::{Ping, PingConfig, PingEvent},
    PeerId, Swarm,
};
use txpool::{SubmitTransactionMessage, TxPoolActor};
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

pub struct NetworkActor {
    network_config: NetworkConfig,
    bus: Addr<BusActor>,
    txpool: Addr<TxPoolActor>,
    //just for test, remove later.
    counter: u64,
}

impl NetworkActor {
    pub fn launch(
        node_config: &NodeConfig,
        bus: Addr<BusActor>,
        txpool: Addr<TxPoolActor>,
    ) -> Result<Addr<NetworkActor>> {
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
            move |ctx: &mut Context<NetworkActor>| {
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

impl Actor for NetworkActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!(
            "Network actor started with config: {:?}",
            self.network_config
        );
    }
}

impl StreamHandler<PingEvent> for NetworkActor {
    fn handle(&mut self, item: PingEvent, _ctx: &mut Self::Context) {
        println!("receive event {:?}", item);
        self.counter += 1;
    }
}

impl Handler<GetCounterMessage> for NetworkActor {
    type Result = u64;

    fn handle(&mut self, _msg: GetCounterMessage, _ctx: &mut Self::Context) -> Self::Result {
        println!("GetCounterMessage {}", self.counter);
        self.counter
    }
}

impl Handler<StopMessage> for NetworkActor {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("Stop network actor.");
        ctx.stop()
    }
}

/// handler system events.
impl Handler<SystemEvents> for NetworkActor {
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
impl Handler<PeerMessage> for NetworkActor {
    type Result = ();

    fn handle(&mut self, msg: PeerMessage, ctx: &mut Self::Context) {
        match msg {
            PeerMessage::UserTransaction(txn) => {
                self.txpool
                    .send(SubmitTransactionMessage { tx: txn })
                    .into_actor(self)
                    .then(|_result, act, _ctx| async {}.into_actor(act))
                    .wait(ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{delay_for, Duration};

    #[actix_rt::test]
    async fn test_network() {
        let node_config = NodeConfig::default();
        let bus = BusActor::launch();
        let txpool = TxPoolActor::launch(&node_config, bus.clone());
        let network = NetworkActor::launch(&node_config, bus, txpool).unwrap();

        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {:?}", peer_id);

        let transport = libp2p::build_development_transport(id_keys).unwrap();

        let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        let remote = node_config.network.advertised_address;
        Swarm::dial_addr(&mut swarm, remote.clone()).unwrap();
        println!("Dialed {}", remote);
        delay_for(Duration::from_millis(200)).await;
        let count = network.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(count, 2);
    }
}
