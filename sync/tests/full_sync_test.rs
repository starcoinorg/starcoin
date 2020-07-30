mod gen_network;

use actix::Actor;
use actix_rt::System;
use bus::{Broadcast, BusActor};
use chain::{ChainActor, ChainActorRef};
use config::{get_random_available_port, NodeConfig};
use futures_timer::Delay;
use gen_network::gen_network;
use libp2p::multiaddr::Multiaddr;
use logger::prelude::*;
use miner::{MinerActor, MinerClientActor};
use network_api::NetworkService;
use starcoin_block_relayer::BlockRelayer;
use starcoin_genesis::Genesis;
use starcoin_state_service::ChainStateActor;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_sync::SyncActor;
use starcoin_wallet_api::WalletAccount;
use std::{sync::Arc, time::Duration};
use traits::ChainAsyncService;
use txpool::{TxPool, TxPoolService};
use types::system_events::SyncBegin;

#[test]
fn test_network_actor_rpc() {
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
        let startup_info_1 = genesis_1.execute_genesis_block(storage_1.clone()).unwrap();
        let txpool_1 = {
            let best_block_id = *startup_info_1.get_master();
            TxPool::start(
                node_config_1.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };
        let tx_pool_service = txpool_1.get_service();

        // network
        let (network_1, addr_1, rx_1) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_1 : {:?}", addr_1);

        // chain
        let first_chain = ChainActor::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            bus_1.clone(),
            tx_pool_service.clone(),
            None,
        )
        .unwrap();

        // network rpc server
        let state_service_1 =
            ChainStateActor::launch(bus_1.clone(), storage_1.clone(), None).unwrap();

        starcoin_network_rpc::start_network_rpc_server(
            rx_1,
            first_chain.clone(),
            storage_1.clone(),
            state_service_1,
            tx_pool_service.clone(),
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
        Delay::new(Duration::from_secs(1)).await;
        if let Err(e) = bus_1.send(Broadcast { msg: SyncBegin }).await {
            error!("error: {:?}", e);
        }

        BlockRelayer::new(bus_1.clone(), txpool_1.get_service(), network_1.clone()).unwrap();

        let miner_account = WalletAccount::random();
        // miner
        let _miner_1 = MinerActor::<TxPoolService, ChainActorRef, Storage>::launch(
            node_config_1.clone(),
            bus_1.clone(),
            storage_1.clone(),
            tx_pool_service.clone(),
            first_chain.clone(),
            miner_account,
        );
        MinerClientActor::new(node_config_1.miner.clone(), node_config_1.net().consensus()).start();
        Delay::new(Duration::from_secs(60)).await;
        let block_1 = first_chain
            .clone()
            .master_head_block()
            .await
            .unwrap()
            .unwrap();
        let number = block_1.header().number();
        debug!("first chain :{:?}", number);
        assert!(number > 0, "assert first chain number >0 fail.");

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
        let startup_info_2 = genesis_2.execute_genesis_block(storage_2.clone()).unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = *startup_info_2.get_master();
            TxPool::start(
                node_config_2.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2, rx_2) = gen_network(
            node_config_2.clone(),
            bus_2.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_2 : {:?}", addr_2);

        // chain
        let second_chain = ChainActor::launch(
            node_config_2.clone(),
            startup_info_2.clone(),
            storage_2.clone(),
            bus_2.clone(),
            txpool_2.get_service(),
            None,
        )
        .unwrap();
        // network rpc server
        let state_service_2 =
            ChainStateActor::launch(bus_2.clone(), storage_2.clone(), None).unwrap();
        starcoin_network_rpc::start_network_rpc_server(
            rx_2,
            second_chain.clone(),
            storage_2.clone(),
            state_service_2,
            txpool_2.get_service(),
        )
        .unwrap();
        // sync
        let second_p = Arc::new(network_2.identify().clone().into());
        let _second_sync_actor = SyncActor::launch(
            node_config_2.clone(),
            bus_2.clone(),
            Arc::clone(&second_p),
            second_chain.clone(),
            txpool_2.get_service(),
            network_2.clone(),
            storage_2.clone(),
        )
        .unwrap();

        let _block_relayer =
            BlockRelayer::new(bus_2.clone(), txpool_2.get_service(), network_2.clone()).unwrap();

        Delay::new(Duration::from_secs(1)).await;
        if let Err(e) = bus_2.clone().send(Broadcast { msg: SyncBegin }).await {
            error!("error: {:?}", e);
        }

        Delay::new(Duration::from_secs(20)).await;

        for i in 0..5 as usize {
            Delay::new(Duration::from_secs(2)).await;
            let block_1 = first_chain
                .clone()
                .master_head_block()
                .await
                .unwrap()
                .unwrap();
            let number_1 = block_1.header().number();
            debug!("index : {}, first chain number is {}", i, number_1);

            let block_2 = second_chain
                .clone()
                .master_head_block()
                .await
                .unwrap()
                .unwrap();
            let number_2 = block_2.header().number();
            debug!("index : {}, second chain number is {}", i, number_2);

            assert!(number_2 > 0);
        }
    };

    system.block_on(fut);
    drop(rt);
}

#[ignore]
#[test]
fn test_network_actor_rpc_2() {
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
        let genesis_1 = Genesis::load(node_config_1.net()).unwrap();
        let genesis_hash = genesis_1.block().header().id();
        let startup_info_1 = genesis_1.execute_genesis_block(storage_1.clone()).unwrap();
        let txpool_1 = {
            let best_block_id = *startup_info_1.get_master();
            TxPool::start(
                node_config_1.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        // network
        let (network_1, addr_1, _rx_1) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        info!("addr_1 : {:?}", addr_1);
        BlockRelayer::new(bus_1.clone(), txpool_1.get_service(), network_1.clone()).unwrap();
        // chain
        let first_chain = ChainActor::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            bus_1.clone(),
            txpool_1.get_service(),
            None,
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
        Delay::new(Duration::from_secs(1)).await;
        if let Err(e) = bus_1.send(Broadcast { msg: SyncBegin }).await {
            error!("error: {:?}", e);
        }

        info!("here");
        let block_1 = first_chain
            .clone()
            .master_head_block()
            .await
            .unwrap()
            .unwrap();
        let number = block_1.header().number();
        info!("first chain :{:?} : {:?}", number, block_1.header().id());

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
        let startup_info_2 = genesis_2.execute_genesis_block(storage_2.clone()).unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = *startup_info_2.get_master();
            TxPool::start(
                node_config_2.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2, _rx_2) =
            gen_network(node_config_2.clone(), bus_2.clone(), handle, genesis_hash);
        debug!("addr_2 : {:?}", addr_2);
        BlockRelayer::new(bus_2.clone(), txpool_2.get_service(), network_2.clone()).unwrap();
        // chain
        let second_chain = ChainActor::launch(
            node_config_2.clone(),
            startup_info_2.clone(),
            storage_2.clone(),
            bus_2.clone(),
            txpool_2.get_service(),
            None,
        )
        .unwrap();
        // sync
        let second_p = Arc::new(network_2.identify().clone().into());
        let _second_sync_actor = SyncActor::launch(
            node_config_2.clone(),
            bus_2.clone(),
            Arc::clone(&second_p),
            second_chain.clone(),
            txpool_2.get_service(),
            network_2.clone(),
            storage_2.clone(),
        )
        .unwrap();
        Delay::new(Duration::from_secs(1)).await;
        if let Err(e) = bus_2.clone().send(Broadcast { msg: SyncBegin }).await {
            error!("error: {:?}", e);
        }

        let block_2 = second_chain
            .clone()
            .master_head_block()
            .await
            .unwrap()
            .unwrap();
        let number = block_2.header().number();
        debug!("second chain :{:?} : {:?}", number, block_2.header().id());

        Delay::new(Duration::from_secs(2)).await;
    };

    system.block_on(fut);
    drop(rt);
}
