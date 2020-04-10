mod gen_network;

use actix_rt::System;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{get_available_port, NodeConfig};
use consensus::dummy::DummyConsensus;
use executor::executor::Executor;
use futures_timer::Delay;
use gen_network::gen_network;
use logger::prelude::*;
use miner::{miner_client::MinerClient, MinerActor};
use network_p2p_api::sync_messages::{GetHashByNumberMsg, ProcessMessage};
use network_p2p_api::sync_messages::{SyncRpcRequest, SyncRpcResponse};
use starcoin_genesis::Genesis;
use starcoin_sync::helper::send_sync_request;
use starcoin_sync::SyncActor;
use starcoin_sync_api::SyncMetadata;
use starcoin_wallet_api::WalletAccount;
use std::{sync::Arc, time::Duration};
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use traits::ChainAsyncService;
use txpool::TxPoolRef;

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
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config_1 = Arc::new(config_1);

        // genesis
        let genesis_1 = Genesis::build(node_config_1.net()).unwrap();
        let genesis_hash = genesis_1.block().header().id();
        let startup_info_1 = genesis_1.execute(storage_1.clone()).unwrap();
        let txpool_1 = {
            let best_block_id = startup_info_1.head.get_head();
            TxPoolRef::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        // network
        let (network_1, addr_1) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_1 : {:?}", addr_1);

        let sync_metadata_actor_1 = SyncMetadata::new(node_config_1.clone());
        // chain
        let first_chain = ChainActor::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_1.clone(),
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
            network_1.clone(),
            storage_1.clone(),
            sync_metadata_actor_1.clone(),
        )
        .unwrap();
        let miner_account = WalletAccount::random();
        // miner
        let _miner_1 = MinerActor::<
            DummyConsensus,
            Executor,
            TxPoolRef,
            ChainActorRef<Executor, DummyConsensus>,
            Storage,
            consensus::dummy::DummyHeader,
        >::launch(
            node_config_1.clone(),
            bus_1.clone(),
            storage_1.clone(),
            txpool_1.clone(),
            first_chain.clone(),
            None,
            miner_account,
        );
        handle.spawn(MinerClient::<DummyConsensus>::run(
            node_config_1.miner.stratum_server,
        ));
        Delay::new(Duration::from_secs(20)).await;
        let block_1 = first_chain.clone().master_head_block().await.unwrap();
        let number = block_1.header().number();
        debug!("first chain :{:?}", number);
        assert!(number > 0);

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
        let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);

        let genesis_2 = Genesis::build(node_config_2.net()).unwrap();
        let genesis_hash = genesis_2.block().header().id();
        let startup_info_2 = genesis_2.execute(storage_2.clone()).unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = startup_info_2.head.get_head();
            TxPoolRef::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2) = gen_network(
            node_config_2.clone(),
            bus_2.clone(),
            handle.clone(),
            genesis_hash,
        );
        debug!("addr_2 : {:?}", addr_2);

        let sync_metadata_actor_2 = SyncMetadata::new(node_config_2.clone());

        // chain
        let second_chain = ChainActor::<Executor, DummyConsensus>::launch(
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
        let _second_sync_actor = SyncActor::<Executor, DummyConsensus>::launch(
            node_config_2.clone(),
            bus_2,
            Arc::clone(&second_p),
            second_chain.clone(),
            network_2.clone(),
            storage_2.clone(),
            sync_metadata_actor_2.clone(),
        )
        .unwrap();

        Delay::new(Duration::from_secs(30)).await;

        for i in 0..5 as usize {
            Delay::new(Duration::from_secs(2)).await;
            let block_1 = first_chain.clone().master_head_block().await.unwrap();
            let number_1 = block_1.header().number();
            debug!("index : {}, first chain number is {}", i, number_1);

            let block_2 = second_chain.clone().master_head_block().await.unwrap();
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
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config_1 = Arc::new(config_1);
        let genesis_1 = Genesis::build(node_config_1.net()).unwrap();
        let genesis_hash = genesis_1.block().header().id();
        let startup_info_1 = genesis_1.execute(storage_1.clone()).unwrap();
        let txpool_1 = {
            let best_block_id = startup_info_1.head.get_head();
            TxPoolRef::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        // network
        let (network_1, addr_1) = gen_network(
            node_config_1.clone(),
            bus_1.clone(),
            handle.clone(),
            genesis_hash,
        );
        info!("addr_1 : {:?}", addr_1);

        let sync_metadata_actor_1 = SyncMetadata::new(node_config_1.clone());
        // chain
        let first_chain = ChainActor::<Executor, DummyConsensus>::launch(
            node_config_1.clone(),
            startup_info_1.clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_1.clone(),
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
            network_1.clone(),
            storage_1.clone(),
            sync_metadata_actor_1.clone(),
        )
        .unwrap();

        info!("here");
        let block_1 = first_chain.clone().master_head_block().await.unwrap();
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
        let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);
        let genesis_2 = Genesis::build(node_config_2.net()).unwrap();
        let genesis_hash = genesis_2.block().header().id();
        let startup_info_2 = genesis_2.execute(storage_2.clone()).unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = startup_info_2.head.get_head();
            TxPoolRef::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2) =
            gen_network(node_config_2.clone(), bus_2.clone(), handle, genesis_hash);
        debug!("addr_2 : {:?}", addr_2);

        let sync_metadata_actor_2 = SyncMetadata::new(node_config_2.clone());
        // chain
        let second_chain = ChainActor::launch(
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
        let _second_sync_actor = SyncActor::<Executor, DummyConsensus>::launch(
            node_config_2.clone(),
            bus_2,
            Arc::clone(&second_p),
            second_chain.clone(),
            network_2.clone(),
            storage_2.clone(),
            sync_metadata_actor_2.clone(),
        )
        .unwrap();

        let block_2 = second_chain.clone().master_head_block().await.unwrap();
        let number = block_2.header().number();
        debug!("second chain :{:?} : {:?}", number, block_2.header().id());

        let mut numbers = Vec::new();
        numbers.push(0);
        let get_hash_by_number_msg = GetHashByNumberMsg { numbers };
        let req = SyncRpcRequest::GetHashByNumberMsg(ProcessMessage::GetHashByNumberMsg(
            get_hash_by_number_msg,
        ));
        let resp = send_sync_request(&network_1, network_2.identify().clone().into(), req.clone())
            .await
            .unwrap();

        assert!(match resp {
            SyncRpcResponse::BatchHashByNumberMsg(_) => true,
            _ => false,
        });

        Delay::new(Duration::from_secs(2)).await;
    };

    system.block_on(fut);
    drop(rt);
}
