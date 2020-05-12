use actix::Actor;
use actix_rt::System;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{ConsensusStrategy, NodeConfig, PacemakerStrategy};
use consensus::dev::DevConsensus;
use logger::prelude::*;
use network::network::NetworkActor;
use starcoin_genesis::Genesis;
use starcoin_miner::MinerActor;
use starcoin_miner::MinerClientActor;
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_wallet_api::WalletAccount;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use sync::SyncActor;
use tokio::time::{delay_for, Duration};
use traits::ChainAsyncService;
use txpool::TxPoolRef;
use types::{
    account_address,
    peer_info::{PeerId, PeerInfo},
};

#[test]
fn test_miner_with_schedule_pacemaker() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let mut system = System::new("test");

    let fut = async move {
        let peer_id = Arc::new(PeerId::random());
        let mut config = NodeConfig::random_for_test();
        config.miner.pacemaker_strategy = PacemakerStrategy::Schedule;
        config.miner.consensus_strategy = ConsensusStrategy::Dummy(1);
        let config = Arc::new(config);
        let bus = BusActor::launch();
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
        );
        let key_pair = config.network.network_keypair();
        let _address = account_address::from_public_key(&key_pair.public_key);
        let genesis = Genesis::build(config.net()).unwrap();
        let genesis_hash = genesis.block().header().id();
        let startup_info = genesis.execute(storage.clone()).unwrap();
        let txpool = {
            let best_block_id = startup_info.master.get_head();
            TxPoolRef::start(
                config.tx_pool.clone(),
                storage.clone(),
                best_block_id,
                bus.clone(),
            )
        };
        let network = NetworkActor::launch(
            config.clone(),
            bus.clone(),
            handle.clone(),
            genesis_hash,
            PeerInfo::default(),
        );
        let sync_metadata = SyncMetadata::new(config.clone(), bus.clone());
        let chain = ChainActor::launch(
            config.clone(),
            startup_info.clone(),
            storage.clone(),
            Some(network.clone()),
            bus.clone(),
            txpool.clone(),
            sync_metadata.clone(),
        )
        .unwrap();
        let miner_account = WalletAccount::random();
        let _miner =
            MinerActor::<DevConsensus, TxPoolRef, ChainActorRef<DevConsensus>, Storage>::launch(
                config.clone(),
                bus.clone(),
                storage.clone(),
                txpool.clone(),
                chain.clone(),
                None,
                miner_account,
            );
        MinerClientActor::new(config.miner.clone()).start();
        let _sync = SyncActor::launch(
            config.clone(),
            bus,
            peer_id,
            chain.clone(),
            txpool.clone(),
            network.clone(),
            storage.clone(),
            sync_metadata.clone(),
        )
        .unwrap();

        delay_for(Duration::from_millis(6 * 1000)).await;
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
        let peer_id = Arc::new(PeerId::random());
        let mut conf = NodeConfig::random_for_test();
        conf.miner.pacemaker_strategy = PacemakerStrategy::Ondemand;
        conf.miner.consensus_strategy = ConsensusStrategy::Dummy(1);
        let config = Arc::new(conf);
        let bus = BusActor::launch();
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap(),
        );

        let key_pair = config.network.network_keypair();
        let _address = account_address::from_public_key(&key_pair.public_key);

        let genesis = Genesis::build(config.net()).unwrap();
        let genesis_hash = genesis.block().header().id();
        let startup_info = genesis.execute(storage.clone()).unwrap();
        let txpool = {
            let best_block_id = startup_info.master.get_head();
            TxPoolRef::start(
                config.tx_pool.clone(),
                storage.clone(),
                best_block_id,
                bus.clone(),
            )
        };
        let network = NetworkActor::launch(
            config.clone(),
            bus.clone(),
            handle.clone(),
            genesis_hash,
            PeerInfo::default(),
        );
        let sync_metadata = SyncMetadata::new(config.clone(), bus.clone());
        let chain = ChainActor::launch(
            config.clone(),
            startup_info.clone(),
            storage.clone(),
            Some(network.clone()),
            bus.clone(),
            txpool.clone(),
            sync_metadata.clone(),
        )
        .unwrap();
        let receiver = txpool.clone().subscribe_txns().await.unwrap();
        let miner_account = WalletAccount::random();
        let _miner =
            MinerActor::<DevConsensus, TxPoolRef, ChainActorRef<DevConsensus>, Storage>::launch(
                config.clone(),
                bus.clone(),
                storage.clone(),
                txpool.clone(),
                chain.clone(),
                Some(receiver),
                miner_account,
            );
        MinerClientActor::new(config.miner.clone()).start();
        let _sync = SyncActor::launch(
            config.clone(),
            bus,
            peer_id,
            chain.clone(),
            txpool.clone(),
            network.clone(),
            storage.clone(),
            sync_metadata.clone(),
        )
        .unwrap();

        delay_for(Duration::from_millis(6 * 10 * 1000)).await;

        let number = chain.clone().master_head_header().await.unwrap().number();
        info!("{}", number);
        assert!(number > 0);

        delay_for(Duration::from_millis(1000)).await;
    };
    system.block_on(fut);
    drop(rt);
}
