use crate::MinerActor;
use actix_rt::{Runtime, System};
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::{dummy::DummyConsensus, Consensus};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use logger::prelude::*;
use network::network::NetworkActor;
use starcoin_genesis::Genesis;
use std::{fmt, sync::Arc, thread};
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use sync::{DownloadActor, ProcessActor, SyncActor};
use tokio::time::{delay_for, Duration};
use traits::{AsyncChain, TxPoolAsyncService};
use txpool::TxPoolRef;
use types::{
    account_address::AccountAddress, peer_info::PeerInfo, transaction::SignedUserTransaction,
};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[actix_rt::test]
async fn test_miner_with_schedule_pacemaker() {
    ::logger::init_for_test();

    let peer_info = Arc::new(PeerInfo::random());
    let config = Arc::new(NodeConfig::default());
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let key_pair = config::gen_keypair();
    let _address = AccountAddress::from_public_key(&key_pair.public_key);
    let genesis =
        Genesis::new::<MockExecutor, StarcoinStorage>(config.clone(), storage.clone()).unwrap();
    let txpool = {
        let best_block_id = genesis.startup_info().head.head_block;
        TxPoolRef::start(storage.clone(), best_block_id, bus.clone())
    };
    let network = NetworkActor::launch(config.clone(), bus.clone(), txpool.clone(), key_pair);
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
        ChainActorRef<ChainActor>,
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

    delay_for(Duration::from_millis(10 * 1000)).await;
    let number = chain.clone().current_header().await.unwrap().number();
    info!("current block number: {}", number);
    assert!(number > 4);
}

#[actix_rt::test]
async fn test_miner_with_ondemand_pacemaker() {
    ::logger::init_for_test();

    let peer_info = Arc::new(PeerInfo::random());
    let mut conf = NodeConfig::default();
    conf.miner.pacemaker_strategy = PacemakerStrategy::Ondemand;
    let config = Arc::new(conf);
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());

    let key_pair = config::gen_keypair();
    let _address = AccountAddress::from_public_key(&key_pair.public_key);
    let genesis =
        Genesis::new::<MockExecutor, StarcoinStorage>(config.clone(), storage.clone()).unwrap();
    let txpool = {
        let best_block_id = genesis.startup_info().head.head_block;
        TxPoolRef::start(storage.clone(), best_block_id, bus.clone())
    };

    let network = NetworkActor::launch(config.clone(), bus.clone(), txpool.clone(), key_pair);
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
        ChainActorRef<ChainActor>,
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

    delay_for(Duration::from_millis(5 * 1000)).await;

    let number = chain.clone().current_header().await.unwrap().number();
    info!("{}", number);
    assert!(number > 0);

    delay_for(Duration::from_millis(1000)).await;
}
