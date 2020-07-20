use crate::GetBlockHeadersByNumber;
use crate::NetworkRpcImpl;
use crate::{gen_client, gen_server::NetworkRpc};
use actix::{Actor, Addr, System};
use block_relayer::BlockRelayer;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::*;
use consensus::dev::DevConsensus;
use crypto::HashValue;
use futures::channel::mpsc::UnboundedReceiver;
use futures_timer::Delay;
use genesis::Genesis;
use miner::{MinerActor, MinerClientActor};
use network::{NetworkActor, NetworkAsyncService};
use network_api::messages::RawRpcRequestMessage;
use network_api::{Multiaddr, NetworkService};
use network_rpc_core::server::NetworkRpcServer;
use std::sync::Arc;
use std::time::Duration;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use txpool::{TxPool, TxPoolService};
use types::{
    block::BlockHeader,
    peer_info::{PeerId, PeerInfo},
};
use wallet_api::WalletAccount;

#[test]
fn test_network_rpc() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut system = System::new("test");
    let (.., network_1, net_addr_1) = {
        let config_1 = NodeConfig::random_for_test();
        gen_chain_env(config_1)
    };
    let (.., network_2, _) = {
        let mut config_2 = NodeConfig::random_for_test();
        config_2.network.seeds = vec![net_addr_1];
        gen_chain_env(config_2)
    };

    // network rpc client for chain 1
    let peer_id_2 = network_2.identify().clone();
    let client = gen_client::NetworkRpcClient::new(network_1);
    let fut = async move {
        Delay::new(Duration::from_secs(15)).await;
        let req = GetBlockHeadersByNumber::new(1, 1, 1);
        let resp: Vec<BlockHeader> = client
            .get_headers_by_number(peer_id_2.into(), req)
            .await
            .unwrap();
        assert!(!resp.is_empty());
    };
    system.block_on(fut);
    drop(rt);
}

fn gen_chain_env(
    mut config: NodeConfig,
) -> (
    ChainActorRef<DevConsensus>,
    Arc<Storage>,
    TxPoolService,
    NetworkAsyncService,
    Multiaddr,
) {
    let bus = BusActor::launch();
    config.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port_from(1024))
        .parse()
        .unwrap();
    let node_config = Arc::new(config);
    let genesis = Genesis::load(node_config.net()).unwrap();
    let genesis_hash = genesis.block().header().id();
    // network
    let (network, rpc_rx, net_addr) = gen_network(node_config.clone(), bus.clone(), genesis_hash);

    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let startup_info = genesis
        .execute_genesis_block(node_config.net(), storage.clone())
        .unwrap();
    let txpool = {
        let best_block_id = *startup_info.get_master();
        TxPool::start(
            node_config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let tx_pool_service = txpool.get_service();
    BlockRelayer::new(bus.clone(), txpool.get_service(), network.clone()).unwrap();
    let chain = ChainActor::<DevConsensus>::launch(
        node_config.clone(),
        startup_info,
        storage.clone(),
        bus.clone(),
        tx_pool_service.clone(),
    )
    .unwrap();

    let miner_account = WalletAccount::random();
    MinerClientActor::new(node_config.miner.clone()).start();
    MinerActor::<DevConsensus, TxPoolService, ChainActorRef<DevConsensus>, Storage>::launch(
        node_config,
        bus,
        storage.clone(),
        tx_pool_service.clone(),
        chain.clone(),
        miner_account,
    )
    .unwrap();
    let rpc_impl = NetworkRpcImpl::new(chain.clone(), tx_pool_service.clone(), storage.clone());
    NetworkRpcServer::start(rpc_rx, rpc_impl.to_delegate()).unwrap();
    (chain, storage, tx_pool_service, network, net_addr)
}

fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    genesis_hash: HashValue,
) -> (
    NetworkAsyncService,
    UnboundedReceiver<RawRpcRequestMessage>,
    Multiaddr,
) {
    let key_pair = node_config.network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let rpc_proto_info = Vec::new();
    let (network, rpc_rx) = NetworkActor::launch(
        node_config.clone(),
        bus,
        genesis_hash,
        PeerInfo::new_for_test(addr, rpc_proto_info),
    );
    let addr_hex = network.identify().to_base58();
    let net_addr: Multiaddr = format!("{}/p2p/{}", &node_config.network.listen, addr_hex)
        .parse()
        .unwrap();
    (network, rpc_rx, net_addr)
}
