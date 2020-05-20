use actix::Addr;
use bus::BusActor;
use config::NodeConfig;
use crypto::hash::HashValue;
use network::{network::NetworkAsyncService, NetworkActor};
use network_api::messages::RawRpcRequestMessage;
use std::sync::Arc;
use tokio::runtime::Handle;
use types::peer_info::{PeerId, PeerInfo};

pub fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    handle: Handle,
    genesis_hash: HashValue,
) -> (
    NetworkAsyncService,
    PeerId,
    futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>,
) {
    let key_pair = node_config.network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let (network, rpc_rx) = NetworkActor::launch(
        node_config,
        bus,
        handle,
        genesis_hash,
        PeerInfo::new_for_test(addr.clone()),
    );
    (network, addr, rpc_rx)
}
