use actix::Addr;
use bus::BusActor;
use config::NodeConfig;
use crypto::hash::HashValue;
use network::{network::NetworkAsyncService, NetworkActor};
use network_api::messages::RawRpcRequestMessage;
use starcoin_network_rpc_api::gen_client::get_rpc_info;
use std::sync::Arc;
use tokio::runtime::Handle;
use types::peer_info::{PeerId, PeerInfo, RpcInfo};

pub fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    _handle: Handle,
    genesis_hash: HashValue,
) -> (
    NetworkAsyncService,
    PeerId,
    futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>,
) {
    let key_pair = node_config.network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let mut rpc_proto_info = Vec::new();
    let chain_rpc_proto_info = get_rpc_info();
    rpc_proto_info.push((
        chain_rpc_proto_info.0.into(),
        RpcInfo::new(chain_rpc_proto_info.1),
    ));
    let (network, rpc_rx) = NetworkActor::launch(
        node_config,
        bus,
        genesis_hash,
        PeerInfo::new_for_test(addr.clone(), rpc_proto_info),
    );
    (network, addr, rpc_rx)
}
