use crate::download::DownloadActor;
use crate::process::ProcessActor;
use crate::sync::SyncActor;
use actix::Addr;
use atomic_refcell::AtomicRefCell;
use bus::BusActor;
use chain::{message::ChainRequest, ChainActor, ChainActorRef};
use config::NodeConfig;
use config::{gen_keypair, get_available_port};
use consensus::{dummy::DummyConsensus, Consensus};
use crypto::hash::{CryptoHash, HashValue};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use futures_timer::Delay;
use miner::MinerActor;
use network::{
    network::NetworkAsyncService,
    sync_messages::{GetHashByNumberMsg, ProcessMessage, SyncMessage},
    NetworkActor, RPCRequest, RPCResponse,
};
use starcoin_genesis::Genesis;
use std::{sync::Arc, time::Duration};
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use traits::mock::MockTxPoolService;
use traits::AsyncChain;
use txpool::{CachedSeqNumberClient, TxPool, TxPoolRef};
use types::account_address::AccountAddress;
use types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
};

fn genesis_block_for_test() -> Block {
    Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test())
}

// fn gen_chain_actor(times: u64) -> (PeerInfo, ChainActorRef<ChainActor>) {
//     let peer = PeerInfo::random();
//     let config = Arc::new(NodeConfig::default());
//     let repo = Arc::new(MemoryStorage::new());
//     let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
//
//     let mut chain = ChainActor::launch(config, storage, None).unwrap();
//     if times > 0 {
//         chain
//             .clone()
//             .address
//             .do_send(ChainRequest::CreateBlock(times));
//     }
//     (peer, chain)
// }
//
// pub fn gen_chain_actors(
//     times: u64,
// ) -> (
//     PeerInfo,
//     ChainActorRef<ChainActor>,
//     PeerInfo,
//     ChainActorRef<ChainActor>,
// ) {
//     let (first_peer, first_chain) = gen_chain_actor(times);
//     let (second_peer, second_chain) = gen_chain_actor(0);
//     (first_peer, first_chain, second_peer, second_chain)
// }

// #[actix_rt::test]
// async fn test_process_actor_new_peer() {
//     let new_peer = PeerInfo::random();
//     let (my_peer, chain) = gen_chain_actor(5);
//
//     let mut process_actor = ProcessActor::launch(Arc::new(my_peer), chain).unwrap();
//     process_actor
//         .send(ProcessMessage::NewPeerMsg(None, new_peer))
//         .await;
//     Delay::new(Duration::from_millis(50)).await;
// }
//
// #[actix_rt::test]
// async fn test_actors() {
//     let (first_peer, first_chain, second_peer, second_chain) = gen_chain_actors(5);
//     let first_p = Arc::new(first_peer.clone());
//     let second_p = Arc::new(second_peer.clone());
//
//     let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), first_chain.clone()).unwrap();
//     let first_d_actor =
//         DownloadActor::launch(first_p, first_chain).expect("launch DownloadActor failed.");
//     let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), second_chain.clone())
//         .expect("launch ProcessActor failed.");
//     let second_d_actor =
//         DownloadActor::launch(second_p, second_chain).expect("launch DownloadActor failed.");
//
//     //connect
//     first_p_actor.do_send(ProcessMessage::NewPeerMsg(
//         Some(second_d_actor),
//         second_peer,
//     ));
//     second_p_actor.do_send(ProcessMessage::NewPeerMsg(Some(first_d_actor), first_peer));
//
//     Delay::new(Duration::from_millis(50)).await;
// }
//
// #[actix_rt::test]
// async fn test_sync_actors() {
//     let (first_peer, first_chain, second_peer, second_chain) = gen_chain_actors(5);
//     let first_p = Arc::new(first_peer.clone());
//     let second_p = Arc::new(second_peer.clone());
//
//     let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), first_chain.clone())
//         .expect("launch ProcessActor failed.");
//     let first_d_actor =
//         DownloadActor::launch(first_p, first_chain.clone()).expect("launch DownloadActor failed.");
//     let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), second_chain.clone())
//         .expect("launch ProcessActor failed.");
//     let second_d_actor = DownloadActor::launch(second_p, second_chain.clone())
//         .expect("launch DownloadActor failed.");
//
//     let first_bus = BusActor::launch();
//     let second_bus = BusActor::launch();
//
//     let first_sync_actor =
//         SyncActor::launch(first_bus, first_p_actor, first_d_actor.clone()).unwrap();
//     let second_sync_actor =
//         SyncActor::launch(second_bus, second_p_actor, second_d_actor.clone()).unwrap();
//
//     //connect
//     first_sync_actor.do_send(SyncMessage::ProcessMessage(ProcessMessage::NewPeerMsg(
//         Some(second_d_actor),
//         second_peer,
//     )));
//     second_sync_actor.do_send(SyncMessage::ProcessMessage(ProcessMessage::NewPeerMsg(
//         Some(first_d_actor),
//         first_peer,
//     )));
//
//     Delay::new(Duration::from_millis(50)).await;
//
//     let block_1 = first_chain.head_block().await.unwrap();
//     let block_2 = second_chain.head_block().await.unwrap();
//     println!(
//         "{}:{}",
//         block_1.header().number(),
//         block_2.header().number()
//     );
//     assert_eq!(block_1.crypto_hash(), block_2.crypto_hash());
// }

fn gen_network(
    node_config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: TxPoolRef,
) -> (NetworkAsyncService<TxPoolRef>, AccountAddress) {
    let key_pair = gen_keypair();
    let addr = AccountAddress::from_public_key(&key_pair.public_key);
    let network = NetworkActor::launch(node_config.clone(), bus, txpool, key_pair);
    (network, addr)
}

#[actix_rt::test]
async fn test_network_actor() {
    //bus
    let bus_1 = BusActor::launch();
    let bus_2 = BusActor::launch();

    //storage
    let storage_1 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());
    let storage_2 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());

    //txpool
    let txpool_1 = TxPool::start(CachedSeqNumberClient::new(storage_1.clone()));
    let txpool_2 = TxPool::start(CachedSeqNumberClient::new(storage_2.clone()));

    //network actor
    let mut config_1 = NodeConfig::default();
    config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
    let node_config_1 = Arc::new(config_1);
    let (network_1, addr_1) = gen_network(node_config_1.clone(), bus_1.clone(), txpool_1.clone());

    let mut config_2 = NodeConfig::default();
    let addr_1_hex = hex::encode(addr_1);
    let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
    config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
    config_2.network.seeds = vec![seed];
    let mut node_config_2 = Arc::new(config_2);
    let (network_2, addr_2) = gen_network(node_config_2.clone(), bus_2.clone(), txpool_2.clone());

    //genesis
    let genesis_1 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_1.clone(), storage_1.clone())
            .unwrap();
    let genesis_2 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_2.clone(), storage_2.clone())
            .unwrap();

    //chain actor
    let first_chain = ChainActor::launch(
        node_config_1.clone(),
        genesis_1.startup_info().clone(),
        storage_1.clone(),
        Some(network_1.clone()),
        bus_1.clone(),
        txpool_1.clone(),
    )
    .unwrap();
    let second_chain = ChainActor::launch(
        node_config_2.clone(),
        genesis_2.startup_info().clone(),
        storage_2.clone(),
        Some(network_2.clone()),
        bus_2.clone(),
        txpool_2.clone(),
    )
    .unwrap();

    //sync
    let first_p = Arc::new(PeerInfo::new(addr_1));
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

    let second_p = Arc::new(PeerInfo::new(addr_2));
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

    //miner
    let _miner_1 = MinerActor::<
        DummyConsensus,
        MockExecutor,
        TxPoolRef,
        ChainActorRef<ChainActor>,
        StarcoinStorage,
    >::launch(
        node_config_1.clone(),
        bus_1.clone(),
        storage_1.clone(),
        txpool_1.clone(),
        first_chain.clone(),
        None,
    );

    Delay::new(Duration::from_secs(5)).await;

    let block_1 = first_chain.head_block().await.unwrap();
    let block_2 = second_chain.head_block().await.unwrap();

    println!(
        "block number:{}:{}",
        block_1.header().number(),
        block_2.header().number()
    );
    assert!(block_2.header().number() > 0);
}

#[actix_rt::test]
async fn test_network_actor_rpc() {
    //first chain
    //bus
    let bus_1 = BusActor::launch();
    //storage
    let storage_1 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());
    //txpool
    let txpool_1 = TxPool::start(CachedSeqNumberClient::new(storage_1.clone()));
    //node config
    let mut config_1 = NodeConfig::default();
    config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
    let node_config_1 = Arc::new(config_1);
    //network
    let (network_1, addr_1) = gen_network(node_config_1.clone(), bus_1.clone(), txpool_1.clone());
    println!("addr_1 : {:?}", addr_1);

    //genesis
    let genesis_1 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_1.clone(), storage_1.clone())
            .unwrap();

    //chain
    let first_chain = ChainActor::launch(
        node_config_1.clone(),
        genesis_1.startup_info().clone(),
        storage_1.clone(),
        Some(network_1.clone()),
        bus_1.clone(),
        txpool_1.clone(),
    )
    .unwrap();
    //sync
    let first_p = Arc::new(PeerInfo::new(addr_1));
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
    //miner
    let _miner_1 = MinerActor::<
        DummyConsensus,
        MockExecutor,
        TxPoolRef,
        ChainActorRef<ChainActor>,
        StarcoinStorage,
    >::launch(
        node_config_1.clone(),
        bus_1.clone(),
        storage_1.clone(),
        txpool_1.clone(),
        first_chain.clone(),
        None,
    );
    Delay::new(Duration::from_secs(5)).await;
    let block_1 = first_chain.clone().head_block().await.unwrap();
    let number = block_1.header().number();
    println!("first chain :{:?}", number);
    assert!(number > 0);

    ////////////////////////
    //second chain
    //bus
    let bus_2 = BusActor::launch();
    //storage
    let storage_2 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());
    //txpool
    let txpool_2 = TxPool::start(CachedSeqNumberClient::new(storage_2.clone()));
    //node config
    let mut config_2 = NodeConfig::default();
    let addr_1_hex = hex::encode(addr_1);
    let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
    config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
    config_2.network.seeds = vec![seed];
    let mut node_config_2 = Arc::new(config_2);
    //network
    let (network_2, addr_2) = gen_network(node_config_2.clone(), bus_2.clone(), txpool_2.clone());
    println!("addr_2 : {:?}", addr_2);
    Delay::new(Duration::from_secs(1)).await;

    let genesis_2 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_2.clone(), storage_2.clone())
            .unwrap();

    //chain
    let second_chain = ChainActor::launch(
        node_config_2.clone(),
        genesis_2.startup_info().clone(),
        storage_2.clone(),
        Some(network_2.clone()),
        bus_2.clone(),
        txpool_2.clone(),
    )
    .unwrap();
    //sync
    let second_p = Arc::new(PeerInfo::new(addr_2));
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
        SyncActor::launch(bus_2, second_p_actor, second_d_actor.clone()).unwrap();

    Delay::new(Duration::from_secs(10)).await;

    for i in 0..5 as usize {
        Delay::new(Duration::from_secs(5)).await;
        let block_1 = first_chain.clone().head_block().await.unwrap();
        let number_1 = block_1.header().number();
        println!("index : {}, first chain number is {}", i, number_1);

        let block_2 = second_chain.clone().head_block().await.unwrap();
        let number_2 = block_2.header().number();
        println!("index : {}, second chain number is {}", i, number_2);

        assert!(number_2 > 0);
        Delay::new(Duration::from_secs(2)).await;
    }
}

#[actix_rt::test]
async fn test_network_actor_rpc_2() {
    //first chain
    //bus
    let bus_1 = BusActor::launch();
    //storage
    let storage_1 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());
    //txpool
    let txpool_1 = TxPool::start(CachedSeqNumberClient::new(storage_1.clone()));
    //node config
    let mut config_1 = NodeConfig::default();
    config_1.network.listen = format!("/ip4/127.0.0.1/tcp/{}", get_available_port());
    let node_config_1 = Arc::new(config_1);
    //network
    let (network_1, addr_1) = gen_network(node_config_1.clone(), bus_1.clone(), txpool_1.clone());
    println!("addr_1 : {:?}", addr_1);
    let genesis_1 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_1.clone(), storage_1.clone())
            .unwrap();
    //chain
    let first_chain = ChainActor::launch(
        node_config_1.clone(),
        genesis_1.startup_info().clone(),
        storage_1.clone(),
        Some(network_1.clone()),
        bus_1.clone(),
        txpool_1.clone(),
    )
    .unwrap();
    //sync
    let first_p = Arc::new(PeerInfo::new(addr_1.clone()));
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
    let block_1 = first_chain.clone().head_block().await.unwrap();
    let number = block_1.header().number();
    println!("first chain :{:?} : {:?}", number, block_1.header().id());

    ////////////////////////
    //second chain
    //bus
    let bus_2 = BusActor::launch();
    //storage
    let storage_2 = Arc::new(StarcoinStorage::new(Arc::new(MemoryStorage::new())).unwrap());
    //txpool
    let txpool_2 = TxPool::start(CachedSeqNumberClient::new(storage_2.clone()));
    //node config
    let mut config_2 = NodeConfig::default();
    let addr_1_hex = hex::encode(addr_1.clone());
    let seed = format!("{}/p2p/{}", &node_config_1.network.listen, addr_1_hex);
    config_2.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
    config_2.network.seeds = vec![seed];
    let mut node_config_2 = Arc::new(config_2);
    //network
    let (network_2, addr_2) = gen_network(node_config_2.clone(), bus_2.clone(), txpool_2.clone());
    Delay::new(Duration::from_secs(1)).await;
    println!("addr_2 : {:?}", addr_2);
    let genesis_2 =
        Genesis::new::<MockExecutor, StarcoinStorage>(node_config_2.clone(), storage_2.clone())
            .unwrap();
    //chain
    let second_chain = ChainActor::launch(
        node_config_2.clone(),
        genesis_2.startup_info().clone(),
        storage_2.clone(),
        Some(network_2.clone()),
        bus_2.clone(),
        txpool_2.clone(),
    )
    .unwrap();
    //sync
    let second_p = Arc::new(PeerInfo::new(addr_2.clone()));
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
        SyncActor::launch(bus_2, second_p_actor, second_d_actor.clone()).unwrap();

    let block_2 = second_chain.clone().head_block().await.unwrap();
    let number = block_2.header().number();
    println!("second chain :{:?} : {:?}", number, block_2.header().id());

    let mut numbers = Vec::new();
    numbers.push(0);
    let get_hash_by_number_msg = GetHashByNumberMsg { numbers };
    let req =
        RPCRequest::GetHashByNumberMsg(ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg));
    let resp = network_1
        .clone()
        .send_request(addr_2.clone(), req.clone(), Duration::from_secs(1))
        .await
        .unwrap();

    assert!(match resp {
        RPCResponse::BatchHashByNumberMsg(_) => true,
        _ => false,
    });

    Delay::new(Duration::from_secs(2)).await;
}
