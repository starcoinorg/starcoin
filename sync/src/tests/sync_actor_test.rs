use crate::{download::DownloadActor, process::ProcessActor, sync::SyncActor};
use actix::Addr;
use actix_rt::System;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{get_available_port, NodeConfig};
use consensus::dummy::DummyConsensus;
use executor::executor::Executor;
use futures_timer::Delay;
use miner::MinerActor;
use network::{
    network::NetworkAsyncService,
    sync_messages::{GetHashByNumberMsg, ProcessMessage},
    NetworkActor, RPCRequest, RPCResponse,
};
use starcoin_genesis::Genesis;
use std::{sync::Arc, time::Duration};
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::StarcoinStorage;
use tokio::runtime::Handle;
use traits::ChainAsyncService;
use txpool::TxPoolRef;
use types::{
    block::{Block, BlockHeader},
    peer_info::{PeerId, PeerInfo},
};

fn _genesis_block_for_test() -> Block {
    Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test())
}

fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    handle: Handle,
) -> (NetworkAsyncService, PeerId) {
    let key_pair = node_config.network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let network = NetworkActor::launch(node_config.clone(), bus, handle);
    (network, addr)
}

#[test]
fn test_network_actor() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let mut system = System::new("test");

    let fut = async move {
        // bus
        let bus_1 = BusActor::launch();
        let bus_2 = BusActor::launch();

        // storage
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage_1 =
            Arc::new(StarcoinStorage::new(cache_storage.clone(), db_storage.clone()).unwrap());

        let cache_storage2 = Arc::new(CacheStorage::new());
        let tmpdir2 = libra_temppath::TempPath::new();
        let db_storage2 = Arc::new(DBStorage::new(tmpdir2.path()));
        let storage_2 =
            Arc::new(StarcoinStorage::new(cache_storage2.clone(), db_storage2.clone()).unwrap());
        // network actor
        let mut config_1 = NodeConfig::random_for_test();
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config_1 = Arc::new(config_1);
        // genesis
        let genesis_1 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_1.clone(),
            storage_1.clone(),
        )
        .unwrap();
        // txpool
        let txpool_1 = {
            let best_block_id = genesis_1.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        let (network_1, _addr_1) =
            gen_network(node_config_1.clone(), bus_1.clone(), handle.clone());
        Delay::new(Duration::from_secs(1)).await;

        // chain actor
        let first_chain = ChainActor::<Executor, DummyConsensus>::launch(
            node_config_1.clone(),
            genesis_1.startup_info().clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_1.clone(),
        )
        .unwrap();

        // sync
        let first_p = Arc::new(PeerInfo::new(network_1.identify().clone().into()));
        let first_p_actor = ProcessActor::launch(
            Arc::clone(&first_p),
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let first_d_actor = DownloadActor::launch(
            first_p,
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let _first_sync_actor =
            SyncActor::launch(bus_1.clone(), first_p_actor, first_d_actor.clone()).unwrap();

        // miner
        let _miner_1 = MinerActor::<
            DummyConsensus,
            Executor,
            TxPoolRef,
            ChainActorRef<Executor, DummyConsensus>,
            StarcoinStorage,
            consensus::dummy::DummyHeader,
        >::launch(
            node_config_1.clone(),
            bus_1.clone(),
            storage_1.clone(),
            txpool_1.clone(),
            first_chain.clone(),
            None,
        );

        Delay::new(Duration::from_secs(3 * 10)).await;

        let mut config_2 = NodeConfig::random_for_test();
        let addr_1_str = network_1.identify().to_base58();
        let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_str);
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);
        let genesis_2 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_2.clone(),
            storage_2.clone(),
        )
        .unwrap();
        let txpool_2 = {
            let best_block_id = genesis_2.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };

        let (network_2, _addr_2) =
            gen_network(node_config_2.clone(), bus_2.clone(), handle.clone());
        Delay::new(Duration::from_secs(1)).await;

        let second_chain = ChainActor::<Executor, DummyConsensus>::launch(
            node_config_2.clone(),
            genesis_2.startup_info().clone(),
            storage_2.clone(),
            Some(network_2.clone()),
            bus_2.clone(),
            txpool_2.clone(),
        )
        .unwrap();

        let second_p = Arc::new(PeerInfo::new(network_2.identify().clone().into()));
        let second_p_actor = ProcessActor::launch(
            Arc::clone(&second_p),
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let second_d_actor = DownloadActor::launch(
            second_p,
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let _second_sync_actor =
            SyncActor::launch(bus_2.clone(), second_p_actor, second_d_actor.clone()).unwrap();

        Delay::new(Duration::from_secs(3 * 10)).await;

        let block_1 = first_chain.master_head_block().await.unwrap();
        let block_2 = second_chain.master_head_block().await.unwrap();

        debug!(
            "block number:{}:{}",
            block_1.header().number(),
            block_2.header().number()
        );
        assert!(block_2.header().number() > 0);
    };
    system.block_on(fut);
    drop(rt);
}

#[ignore]
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
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage_1 = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());
        // node config
        let mut config_1 = NodeConfig::random_for_test();
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config_1 = Arc::new(config_1);

        // genesis
        let genesis_1 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_1.clone(),
            storage_1.clone(),
        )
        .unwrap();
        let txpool_1 = {
            let best_block_id = genesis_1.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        // network
        let (network_1, addr_1) = gen_network(node_config_1.clone(), bus_1.clone(), handle.clone());
        debug!("addr_1 : {:?}", addr_1);

        // chain
        let first_chain = ChainActor::launch(
            node_config_1.clone(),
            genesis_1.startup_info().clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_1.clone(),
        )
        .unwrap();
        // sync
        let first_p = Arc::new(PeerInfo::new(network_1.identify().clone().into()));
        let first_p_actor = ProcessActor::launch(
            Arc::clone(&first_p),
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let first_d_actor = DownloadActor::launch(
            first_p,
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let _first_sync_actor =
            SyncActor::launch(bus_1.clone(), first_p_actor, first_d_actor.clone()).unwrap();
        // miner
        let _miner_1 = MinerActor::<
            DummyConsensus,
            Executor,
            TxPoolRef,
            ChainActorRef<Executor, DummyConsensus>,
            StarcoinStorage,
            consensus::dummy::DummyHeader,
        >::launch(
            node_config_1.clone(),
            bus_1.clone(),
            storage_1.clone(),
            txpool_1.clone(),
            first_chain.clone(),
            None,
        );
        Delay::new(Duration::from_secs(1 * 60)).await;
        let block_1 = first_chain.clone().master_head_block().await.unwrap();
        let number = block_1.header().number();
        debug!("first chain :{:?}", number);
        assert!(number > 0);

        ////////////////////////
        // second chain
        // bus
        let bus_2 = BusActor::launch();
        // storage
        let cache_storage2 = Arc::new(CacheStorage::new());
        let tmpdir2 = libra_temppath::TempPath::new();
        let db_storage2 = Arc::new(DBStorage::new(tmpdir2.path()));
        let storage_2 = Arc::new(StarcoinStorage::new(cache_storage2, db_storage2).unwrap());

        // node config
        let mut config_2 = NodeConfig::random_for_test();
        let addr_1_hex = network_1.identify().to_base58();
        let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);

        let genesis_2 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_2.clone(),
            storage_2.clone(),
        )
        .unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = genesis_2.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2) = gen_network(node_config_2.clone(), bus_2.clone(), handle.clone());
        debug!("addr_2 : {:?}", addr_2);
        Delay::new(Duration::from_secs(1)).await;

        // chain
        let second_chain = ChainActor::<Executor, DummyConsensus>::launch(
            node_config_2.clone(),
            genesis_2.startup_info().clone(),
            storage_2.clone(),
            Some(network_2.clone()),
            bus_2.clone(),
            txpool_2.clone(),
        )
        .unwrap();
        // sync
        let second_p = Arc::new(PeerInfo::new(network_2.identify().clone().into()));
        let second_p_actor = ProcessActor::<Executor, DummyConsensus>::launch(
            Arc::clone(&second_p),
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let second_d_actor = DownloadActor::<Executor, DummyConsensus>::launch(
            second_p,
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let _second_sync_actor = SyncActor::<Executor, DummyConsensus>::launch(
            bus_2,
            second_p_actor,
            second_d_actor.clone(),
        )
        .unwrap();

        Delay::new(Duration::from_secs(1 * 60)).await;

        for i in 0..5 as usize {
            Delay::new(Duration::from_secs(5)).await;
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
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage_1 = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());
        // node config
        let mut config_1 = NodeConfig::random_for_test();
        config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
        let node_config_1 = Arc::new(config_1);
        let genesis_1 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_1.clone(),
            storage_1.clone(),
        )
        .unwrap();
        let txpool_1 = {
            let best_block_id = genesis_1.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_1.tx_pool.clone(),
                storage_1.clone(),
                best_block_id,
                bus_1.clone(),
            )
        };

        // network
        let (network_1, addr_1) = gen_network(node_config_1.clone(), bus_1.clone(), handle.clone());
        info!("addr_1 : {:?}", addr_1);

        // chain
        let first_chain = ChainActor::<Executor, DummyConsensus>::launch(
            node_config_1.clone(),
            genesis_1.startup_info().clone(),
            storage_1.clone(),
            Some(network_1.clone()),
            bus_1.clone(),
            txpool_1.clone(),
        )
        .unwrap();
        // sync
        let first_p = Arc::new(PeerInfo::new(network_1.identify().clone().into()));
        let first_p_actor = ProcessActor::<Executor, DummyConsensus>::launch(
            Arc::clone(&first_p),
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let first_d_actor = DownloadActor::<Executor, DummyConsensus>::launch(
            first_p,
            first_chain.clone(),
            network_1.clone(),
            bus_1.clone(),
        )
        .unwrap();
        let _first_sync_actor =
            SyncActor::launch(bus_1.clone(), first_p_actor, first_d_actor.clone()).unwrap();

        info!("here");
        let block_1 = first_chain.clone().master_head_block().await.unwrap();
        let number = block_1.header().number();
        info!("first chain :{:?} : {:?}", number, block_1.header().id());

        ////////////////////////
        // second chain
        // bus
        let bus_2 = BusActor::launch();
        // storage
        let cache_storage2 = Arc::new(CacheStorage::new());
        let tmpdir2 = libra_temppath::TempPath::new();
        let db_storage2 = Arc::new(DBStorage::new(tmpdir2.path()));
        let storage_2 = Arc::new(StarcoinStorage::new(cache_storage2, db_storage2).unwrap());
        // node config
        let mut config_2 = NodeConfig::random_for_test();
        let addr_1_hex = network_1.identify().to_base58();
        let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
        config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
        config_2.network.seeds = vec![seed];
        let node_config_2 = Arc::new(config_2);
        let genesis_2 = Genesis::new::<Executor, DummyConsensus, StarcoinStorage>(
            node_config_2.clone(),
            storage_2.clone(),
        )
        .unwrap();
        // txpool
        let txpool_2 = {
            let best_block_id = genesis_2.startup_info().head.get_head();
            TxPoolRef::start(
                node_config_2.tx_pool.clone(),
                storage_2.clone(),
                best_block_id,
                bus_2.clone(),
            )
        };
        // network
        let (network_2, addr_2) = gen_network(node_config_2.clone(), bus_2.clone(), handle);
        Delay::new(Duration::from_secs(1)).await;
        debug!("addr_2 : {:?}", addr_2);

        // chain
        let second_chain = ChainActor::launch(
            node_config_2.clone(),
            genesis_2.startup_info().clone(),
            storage_2.clone(),
            Some(network_2.clone()),
            bus_2.clone(),
            txpool_2.clone(),
        )
        .unwrap();
        // sync
        let second_p = Arc::new(PeerInfo::new(network_2.identify().clone().into()));
        let second_p_actor = ProcessActor::launch(
            Arc::clone(&second_p),
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let second_d_actor = DownloadActor::launch(
            second_p,
            second_chain.clone(),
            network_2.clone(),
            bus_2.clone(),
        )
        .unwrap();
        let _second_sync_actor = SyncActor::<Executor, DummyConsensus>::launch(
            bus_2,
            second_p_actor,
            second_d_actor.clone(),
        )
        .unwrap();

        let block_2 = second_chain.clone().master_head_block().await.unwrap();
        let number = block_2.header().number();
        debug!("second chain :{:?} : {:?}", number, block_2.header().id());

        let mut numbers = Vec::new();
        numbers.push(0);
        let get_hash_by_number_msg = GetHashByNumberMsg { numbers };
        let req = RPCRequest::GetHashByNumberMsg(ProcessMessage::GetHashByNumberMsg(
            get_hash_by_number_msg,
        ));
        let resp = network_1
            .clone()
            .send_request(
                network_2.identify().clone().into(),
                req.clone(),
                Duration::from_secs(1),
            )
            .await
            .unwrap();

        assert!(match resp {
            RPCResponse::BatchHashByNumberMsg(_) => true,
            _ => false,
        });

        Delay::new(Duration::from_secs(2)).await;
    };

    system.block_on(fut);
    drop(rt);
}
