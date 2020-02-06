// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use config::{NetworkConfig, NodeConfig};
use libp2p::{
    identity,
    ping::{Ping, PingConfig, PingEvent},
    PeerId, Swarm,
};

pub struct NetworkActor {
    network_config: NetworkConfig,
    //just for test, remove later.
    counter: u64,
}

#[derive(Message)]
#[rtype(result = "u64")]
pub struct GetCounterMessage {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopMessage {}

impl NetworkActor {
    pub fn launch(node_config: &NodeConfig) -> Result<Addr<NetworkActor>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{delay_for, Duration};

    #[actix_rt::test]
    async fn test_network() {
        let node_config = NodeConfig::default();
        let network = NetworkActor::launch(&node_config).unwrap();

        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {:?}", peer_id);

        let transport = libp2p::build_development_transport(id_keys).unwrap();

        let behaviour = Ping::new(PingConfig::new().with_keep_alive(true));

        let mut swarm = Swarm::new(transport, behaviour, peer_id);
        let remote = node_config.network.advertised_address;
        Swarm::dial_addr(&mut swarm, remote.clone()).unwrap();
        println!("Dialed {}", remote);
        delay_for(Duration::from_millis(100)).await;
        let count = network.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(count, 2);
    }
}
