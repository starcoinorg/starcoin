#[cfg(test)]
mod tests {
    use crate::rpc_impl::NetworkRpcImpl;
    use crate::GetBlockHeadersByNumber;
    use crate::{gen_client, gen_server::NetworkRpc};
    use actix::{Addr, System};
    use bus::BusActor;
    use chain::ChainActor;
    use config::*;
    use consensus::dev::DevConsensus;
    use crypto::HashValue;
    use genesis::Genesis;
    use network::{NetworkActor, NetworkAsyncService};
    use network_api::messages::RawRpcRequestMessage;
    use network_api::NetworkService;
    use network_rpc_core::server::NetworkRpcServer;
    use std::sync::Arc;
    use storage::cache_storage::CacheStorage;
    use storage::storage::StorageInstance;
    use storage::Storage;
    use txpool::TxPool;
    use types::{
        block::BlockHeader,
        peer_info::{PeerId, PeerInfo},
    };

    #[test]
    fn test_network_rpc() {
        ::logger::init_for_test();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut system = System::new("test");
        let fut = async move {
            let mut config_1 = NodeConfig::random_for_test();
            let bus_1 = BusActor::launch();
            config_1.network.listen =
                format!("/ip4/127.0.0.1/tcp/{}", get_available_port_from(1024))
                    .parse()
                    .unwrap();
            let node_config_1 = Arc::new(config_1);
            let genesis_1 = Genesis::load(node_config_1.net()).unwrap();
            let genesis_hash = genesis_1.block().header().id();
            // network
            let (network_1, _addr_1, _rx_1) =
                gen_network(node_config_1.clone(), bus_1.clone(), genesis_hash);
            let storage_1 = Arc::new(
                Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
            );
            let startup_info_1 = genesis_1
                .execute_genesis_block(node_config_1.net(), storage_1.clone())
                .unwrap();

            let txpool_1 = {
                let best_block_id = *startup_info_1.get_master();
                TxPool::start(
                    node_config_1.tx_pool.clone(),
                    storage_1.clone(),
                    best_block_id,
                    bus_1.clone(),
                )
            };
            let tx_pool_service = txpool_1.get_service();
            // chain
            let _chain_1 = ChainActor::<DevConsensus>::launch(
                node_config_1.clone(),
                startup_info_1.clone(),
                storage_1.clone(),
                bus_1.clone(),
                tx_pool_service.clone(),
            )
            .unwrap();

            let bus_2 = BusActor::launch();
            // node config
            let mut config_2 = NodeConfig::random_for_test();
            let addr_1_hex = network_1.identify().to_base58();
            let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex)
                .parse()
                .unwrap();
            config_2.network.listen = format!(
                "/ip4/127.0.0.1/tcp/{}",
                config::get_available_port_from(1025)
            )
            .parse()
            .unwrap();
            config_2.network.seeds = vec![seed];
            let node_config_2 = Arc::new(config_2);

            let genesis_2 = Genesis::load(node_config_2.net()).unwrap();
            let genesis_hash = genesis_2.block().header().id();
            // network
            let (_network_2, addr_2, rx_2) =
                gen_network(node_config_2.clone(), bus_2.clone(), genesis_hash);
            let storage_2 = Arc::new(
                Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
            );
            let startup_info_2 = genesis_2
                .execute_genesis_block(node_config_2.net(), storage_2.clone())
                .unwrap();

            let txpool_2 = {
                let best_block_id = *startup_info_2.get_master();
                TxPool::start(
                    node_config_2.tx_pool.clone(),
                    storage_2.clone(),
                    best_block_id,
                    bus_2.clone(),
                )
            };

            let tx_pool_service = txpool_2.get_service();
            // chain
            let chain_2 = ChainActor::<DevConsensus>::launch(
                node_config_2.clone(),
                startup_info_2.clone(),
                storage_2.clone(),
                bus_2.clone(),
                tx_pool_service.clone(),
            )
            .unwrap();
            // server
            let rpc_impl = NetworkRpcImpl::new(chain_2, tx_pool_service, storage_2);
            let _ = NetworkRpcServer::start(rx_2, rpc_impl.to_delegate());
            // client
            let client = gen_client::NetworkRpcClient::new(network_1);
            let req = GetBlockHeadersByNumber::new(1, 2, 3);
            let resp = client
                .get_headers_by_number(addr_2, req.clone())
                .await
                .unwrap();
            assert_eq!(Vec::<BlockHeader>::new(), resp);
        };
        system.block_on(fut);
        drop(rt);
    }

    pub fn gen_network(
        node_config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        genesis_hash: HashValue,
    ) -> (
        NetworkAsyncService,
        PeerId,
        futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>,
    ) {
        let key_pair = node_config.network.network_keypair();
        let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
        let rpc_proto_info = Vec::new();
        let (network, rpc_rx) = NetworkActor::launch(
            node_config,
            bus,
            genesis_hash,
            PeerInfo::new_for_test(addr.clone(), rpc_proto_info),
        );
        (network, addr, rpc_rx)
    }
}
