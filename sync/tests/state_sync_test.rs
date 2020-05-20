mod gen_network;

use actix::Actor;
use actix_rt::System;
use bus::{Broadcast, BusActor};
use chain::{ChainActor, ChainActorRef};
use config::{get_available_port, NodeConfig};
use consensus::dev::DevConsensus;
use futures_timer::Delay;
use gen_network::gen_network;
use libp2p::multiaddr::Multiaddr;
use logger::prelude::*;
use miner::{MinerActor, MinerClientActor};
use network_api::NetworkService;
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_sync::SyncActor;
use starcoin_sync_api::SyncMetadata;
use starcoin_wallet_api::WalletAccount;
use std::{sync::Arc, time::Duration};
use traits::ChainAsyncService;
use txpool::{TxPool, TxPoolService};
use types::system_events::SyncBegin;

#[stest::test(timeout = 120)]
fn test_state_sync() {
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
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port())
            .parse()
            .unwrap();
        debug!("first peer : {:?}", config_1.network.self_peer_id);
        let node_config_1 = Arc::new(config_1);

        // genesis
        let genesis_1 = Genesis::build(node_config_1.net()).unwrap();
        let genesis_hash = genesis_1.block().header().id();
        let startup_info_1 = genesis_1.execute(storage_1.clone()).unwrap();

        let txpool_1 = {
            let best_block_id = *startup_info_1.get_master();
            TxPool::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };
        let txpool_ref_1 = txpool_1.get_async_service();

        // network
        let (network_1, addr_1, rx_1) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_1 : {:?}", addr_1);

        let sync_metadata_actor_1 = SyncMetadata::new(node_config_1.clone(), bus_1.clone());
        // chain
        let first_chain = ChainActor::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_ref_1.clone(),
            sync_metadata_actor_1.clone(),
        )
        .unwrap();
        // sync
        let first_p = Arc::new(network_1.identify().clone().into());
        let _first_sync_actor = SyncActor::launch(
            node_config_1.clone(),
            bus_1.clone(),
            first_p,
            first_chain.clone(),
            txpool_ref_1.clone(),
            network_1.clone(),
            storage_1.clone(),
            sync_metadata_actor_1.clone(),
            rx_1,
        )
        .unwrap();
        Delay::new(Duration::from_secs(1)).await;
        let _ = bus_1.clone().send(Broadcast { msg: SyncBegin }).await;
        let miner_account = WalletAccount::random();
        // miner
        let _miner_1 = MinerActor::<
            DevConsensus,
            TxPoolService,
            ChainActorRef<DevConsensus>,
            Storage,
        >::launch(
            node_config_1.clone(),
            bus_1.clone(),
            storage_1.clone(),
            txpool_1.get_service(),
            first_chain.clone(),
            miner_account,
        );
        MinerClientActor::new(node_config_1.miner.clone()).start();
        Delay::new(Duration::from_secs(30)).await;
        let mut block_1 = first_chain.clone().master_head_block().await.unwrap();
        while block_1.header().number() <= 11 {
            Delay::new(Duration::from_secs(5)).await;
            block_1 = first_chain.clone().master_head_block().await.unwrap();
        }

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
        config_2.sync.fast_sync_mode();
        let addr_1_hex = network_1.identify().to_base58();
        let seed: Multiaddr = format!(
            "{}/p2p/{}",
            &node_config_1.network.listen.to_string(),
            addr_1_hex
        )
        .parse()
        .unwrap();
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port())
            .parse()
            .unwrap();
        config_2.network.seeds = vec![seed];
        debug!("second peer : {:?}", config_2.network.self_peer_id);
        let node_config_2 = Arc::new(config_2);

        let genesis_2 = Genesis::build(node_config_2.net()).unwrap();
        let genesis_hash = genesis_2.block().header().id();
        let startup_info_2 = genesis_2.execute(storage_2.clone()).unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = *startup_info_2.get_master();
            TxPool::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
            .get_async_service()
        };
        // network
        let (network_2, addr_2, rx_2) = gen_network(
            node_config_2.clone(),
            bus_2.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_2 : {:?}", addr_2);

        let sync_metadata_actor_2 = SyncMetadata::new(node_config_2.clone(), bus_2.clone());
        assert!(
            sync_metadata_actor_2.state_syncing(),
            "state_syncing is false."
        );

        // chain
        let second_chain = ChainActor::<DevConsensus>::launch(
            node_config_2.clone(),
            startup_info_2.clone(),
            storage_2.clone(),
            Some(network_2.clone()),
            bus_2.clone(),
            txpool_2.clone(),
            sync_metadata_actor_2.clone(),
        )
        .unwrap();
        // sync
        let second_p = Arc::new(network_2.identify().clone().into());
        let _second_sync_actor = SyncActor::<DevConsensus>::launch(
            node_config_2.clone(),
            bus_2.clone(),
            Arc::clone(&second_p),
            second_chain.clone(),
            txpool_2.clone(),
            network_2.clone(),
            storage_2.clone(),
            sync_metadata_actor_2.clone(),
            rx_2,
        )
        .unwrap();
        Delay::new(Duration::from_secs(5)).await;
        let _ = bus_2.clone().send(Broadcast { msg: SyncBegin }).await;

        Delay::new(Duration::from_secs(30)).await;

        assert!(
            !sync_metadata_actor_2.state_syncing(),
            "state_syncing is true."
        );
    };

    system.block_on(fut);
    drop(rt);
}
