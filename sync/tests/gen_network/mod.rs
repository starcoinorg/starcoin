use actix::Addr;
use bus::BusActor;
use config::NodeConfig;
use crypto::hash::HashValue;
use network::{network::NetworkAsyncService, NetworkActor};
use std::sync::Arc;
use tokio::runtime::Handle;
use types::peer_info::PeerId;

pub fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    handle: Handle,
    genesis_hash: HashValue,
) -> (NetworkAsyncService, PeerId) {
    let key_pair = node_config.network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let network = NetworkActor::launch(node_config.clone(), bus, handle, genesis_hash);
    (network, addr)
}
