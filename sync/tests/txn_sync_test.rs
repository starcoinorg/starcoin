mod gen_network;

use actix_rt::System;
use bus::{Bus, BusActor};
use chain::ChainActor;
use config::{get_random_available_port, NodeConfig};
use consensus::dev::DevConsensus;
use crypto::{hash::PlainCryptoHash, keygen::KeyGen};
use futures_timer::Delay;
use gen_network::gen_network;
use libp2p::multiaddr::Multiaddr;
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_sync::SyncActor;
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_txpool_api::TxPoolSyncService;
use std::{sync::Arc, time::Duration};
use txpool::TxPool;
use types::{
    account_address,
    transaction::helpers::get_current_timestamp,
    transaction::{authenticator::AuthenticationKey, SignedUserTransaction},
};

#[test]
fn test_txn_sync_actor() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let mut system = System::new("test");

    let fut = async move {
        // first chain
        // bus
        let bus_1 = BusActor::launch();
        // storage
        let storage_1 = Arc::new(
            Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
        );
        // node config
        let mut config_1 = NodeConfig::random_for_test();
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_random_available_port())
            .parse()
            .unwrap();
        let node_config_1 = Arc::new(config_1);

        // genesis
        let genesis_1 = Genesis::load(node_config_1.net()).unwrap();
        let genesis_hash = genesis_1.block().header().id();
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

        // network
        let (network_1, addr_1, rpc_rx) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_1 : {:?}", addr_1);
        // chain
        let first_chain = ChainActor::<DevConsensus>::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            bus_1.clone(),
            txpool_1.get_service(),
        )
        .unwrap();

        // network rpc server
        network_rpc::start_network_rpc_server(
            rpc_rx,
            first_chain.clone(),
            storage_1.clone(),
            txpool_1.get_service(),
        )
        .unwrap();
        // sync
        let first_p = Arc::new(network_1.identify().clone().into());
        let _first_sync_actor = SyncActor::launch(
            node_config_1.clone(),
            bus_1.clone(),
            first_p,
            first_chain.clone(),
            txpool_1.get_service(),
            network_1.clone(),
            storage_1.clone(),
        )
        .unwrap();

        // add txn to node1
        let user_txn = gen_user_txn();
        let import_result = txpool_1
            .get_service()
            .add_txns(vec![user_txn.clone()])
            .pop();
        assert!(import_result.unwrap().is_ok());

        ////////////////////////
        // second chain
        // bus
        let bus_2 = BusActor::launch();
        // storage
        let storage_2 = Arc::new(
            Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
        );

        // node config
        let mut config_2 = NodeConfig::random_for_test();
        let addr_1_hex = network_1.identify().to_base58();
        let seed: Multiaddr = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex)
            .parse()
            .unwrap();
        config_2.network.listen =
            format!("/ip4/127.0.0.1/tcp/{}", config::get_random_available_port())
                .parse()
                .unwrap();
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);

        let genesis_2 = Genesis::load(node_config_2.net()).unwrap();
        let genesis_hash = genesis_2.block().header().id();
        let startup_info_2 = genesis_2
            .execute_genesis_block(node_config_2.net(), storage_2.clone())
            .unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = *startup_info_2.get_master();
            TxPool::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2, rpc_rx_2) = gen_network(
            node_config_2.clone(),
            bus_2.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_2 : {:?}", addr_2);

        // chain
        let second_chain = ChainActor::<DevConsensus>::launch(
            node_config_2.clone(),
            startup_info_2.clone(),
            storage_2.clone(),
            bus_2.clone(),
            txpool_2.get_service(),
        )
        .unwrap();

        // network rpc server
        network_rpc::start_network_rpc_server(
            rpc_rx_2,
            second_chain.clone(),
            storage_2.clone(),
            txpool_2.get_service(),
        )
        .unwrap();

        // sync
        let second_p = Arc::new(network_2.identify().clone().into());
        let _second_sync_actor = SyncActor::<DevConsensus>::launch(
            node_config_2.clone(),
            bus_2.clone(),
            Arc::clone(&second_p),
            second_chain.clone(),
            txpool_2.get_service(),
            network_2.clone(),
            storage_2.clone(),
        )
        .unwrap();

        Delay::new(Duration::from_secs(10)).await;

        // make node2 to sync txn
        bus_2.clone().broadcast(StartSyncTxnEvent).await.unwrap();
        // wait 10s to sync done
        Delay::new(Duration::from_secs(10)).await;

        // check txn
        let mut txns = txpool_2.get_service().get_pending_txns(None, None);
        assert!(txns.len() == 1);
        let txn = txns.pop().unwrap();
        assert_eq!(user_txn.crypto_hash(), txn.crypto_hash());
    };

    system.block_on(fut);
    drop(rt);
}

fn gen_user_txn() -> SignedUserTransaction {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
    let txn = executor::build_transfer_from_association(
        account_address,
        auth_prefix,
        0,
        10000,
        get_current_timestamp() + 40000,
    );
    txn.as_signed_user_txn().unwrap().clone()
}
