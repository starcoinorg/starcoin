use crate::MinerActor;
use actix_rt::System;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::dummy::DummyConsensus;
use executor::mock_executor::MockExecutor;
use logger::prelude::*;
use network::network::NetworkActor;
use starcoin_genesis::Genesis;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::StarcoinStorage;
use sync::{DownloadActor, ProcessActor, SyncActor};
use tokio::time::{delay_for, Duration};
use traits::{ChainAsyncService, TxPoolAsyncService};
use txpool::TxPoolRef;
use types::{account_address::AccountAddress, peer_info::PeerInfo};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_miner_with_schedule_pacemaker() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let mut system = System::new("test");

    let fut = async move {
        let peer_info = Arc::new(PeerInfo::random());
        let config = Arc::new(NodeConfig::random_for_test());
        let bus = BusActor::launch();
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());
        let key_pair = config.network.network_keypair();
        let _address = AccountAddress::from_public_key(&key_pair.public_key);
        let genesis = Genesis::new::<MockExecutor, DummyConsensus, StarcoinStorage>(
            config.clone(),
            storage.clone(),
        )
        .unwrap();
        let txpool = {
            let best_block_id = genesis.startup_info().head.get_head();
            TxPoolRef::start(
                config.tx_pool.clone(),
                storage.clone(),
                best_block_id,
                bus.clone(),
            )
        };
        let network = NetworkActor::launch(config.clone(), bus.clone(), handle);
        let chain = ChainActor::launch(
            config.clone(),
            genesis.startup_info().clone(),
            storage.clone(),
            Some(network.clone()),
            bus.clone(),
            txpool.clone(),
        )
        .unwrap();
        let _miner = MinerActor::<
            DummyConsensus,
            MockExecutor,
            TxPoolRef,
            ChainActorRef,
            StarcoinStorage,
        >::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool.clone(),
            chain.clone(),
            None,
        );

        let process_actor = ProcessActor::launch(
            Arc::clone(&peer_info),
            chain.clone(),
            network.clone(),
            bus.clone(),
        )
        .unwrap();
        let download_actor =
            DownloadActor::launch(peer_info, chain.clone(), network.clone(), bus.clone())
                .expect("launch DownloadActor failed.");
        let _sync = SyncActor::launch(bus.clone(), process_actor, download_actor).unwrap();

        delay_for(Duration::from_millis(6 * 10 * 1000)).await;
        let number = chain.clone().master_head_header().await.unwrap().number();
        info!("current block number: {}", number);
        assert!(number > 1);
    };
    system.block_on(fut);
    drop(rt);
}

#[ignore]
#[test]
fn test_miner_with_ondemand_pacemaker() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let mut system = System::new("test");

    let fut = async move {
        let peer_info = Arc::new(PeerInfo::random());
        let mut conf = NodeConfig::random_for_test();
        conf.miner.pacemaker_strategy = PacemakerStrategy::Ondemand;
        let config = Arc::new(conf);
        let bus = BusActor::launch();
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());

        let key_pair = config.network.network_keypair();
        let _address = AccountAddress::from_public_key(&key_pair.public_key);
        let genesis = Genesis::new::<MockExecutor, DummyConsensus, StarcoinStorage>(
            config.clone(),
            storage.clone(),
        )
        .unwrap();
        let txpool = {
            let best_block_id = genesis.startup_info().head.get_head();
            TxPoolRef::start(
                config.tx_pool.clone(),
                storage.clone(),
                best_block_id,
                bus.clone(),
            )
        };
        let network = NetworkActor::launch(config.clone(), bus.clone(), handle);
        let chain = ChainActor::launch(
            config.clone(),
            genesis.startup_info().clone(),
            storage.clone(),
            Some(network.clone()),
            bus.clone(),
            txpool.clone(),
        )
        .unwrap();
        let receiver = txpool.clone().subscribe_txns().await.unwrap();

        let _miner = MinerActor::<
            DummyConsensus,
            MockExecutor,
            TxPoolRef,
            ChainActorRef,
            StarcoinStorage,
        >::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool.clone(),
            chain.clone(),
            Some(receiver),
        );

        let process_actor = ProcessActor::launch(
            Arc::clone(&peer_info),
            chain.clone(),
            network.clone(),
            bus.clone(),
        )
        .unwrap();
        let download_actor =
            DownloadActor::launch(peer_info, chain.clone(), network.clone(), bus.clone())
                .expect("launch DownloadActor failed.");
        let _sync = SyncActor::launch(bus.clone(), process_actor, download_actor).unwrap();

        delay_for(Duration::from_millis(6 * 10 * 1000)).await;

        let number = chain.clone().master_head_header().await.unwrap().number();
        info!("{}", number);
        assert!(number > 0);

        delay_for(Duration::from_millis(1000)).await;
    };
    system.block_on(fut);
    drop(rt);
}
