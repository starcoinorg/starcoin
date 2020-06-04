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
    let mut rpc_proto_info = Vec::new();
    let sync_rpc_proto_info = starcoin_sync::helper::sync_rpc_info();
    rpc_proto_info.push((sync_rpc_proto_info.0.into(), sync_rpc_proto_info.1));
    let (network, rpc_rx) = NetworkActor::launch(
        node_config,
        bus,
        handle,
        genesis_hash,
        PeerInfo::new_for_test(addr.clone(), rpc_proto_info),
    );
    (network, addr, rpc_rx)
}
