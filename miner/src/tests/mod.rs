use crate::MinerActor;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::{dummy::DummyConsensus, Consensus};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use sync::{DownloadActor, ProcessActor, SyncActor};
use tokio::time::{delay_for, Duration};
use traits::{AsyncChain, TxPoolAsyncService};
use txpool::{CachedSeqNumberClient, TxPool, TxPoolActor, TxPoolRef, SubscribeTxns};
use types::{peer_info::PeerInfo, transaction::SignedUserTransaction, account_address::AccountAddress};
use network::network::NetworkActor;
use actix_rt::{System, Runtime};
use std::{fmt, thread};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

#[actix_rt::test]
async fn test_miner_with_schedule_pacemaker() {
    let peer_info = Arc::new(PeerInfo::random());
    let config = Arc::new(NodeConfig::default());
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let seq_number_client = CachedSeqNumberClient::new(storage.clone());
    let txpool = TxPool::start(seq_number_client);
    let key_pair = config::gen_keypair();
    let _address = AccountAddress::from_public_key(&key_pair.public_key);
    let network = NetworkActor::launch(config.clone(), bus.clone(), txpool.clone(), key_pair);
    let chain = ChainActor::launch(config.clone(), storage.clone(), Some(network.clone())).unwrap();
    let _miner =
        MinerActor::<DummyConsensus, MockExecutor, TxPoolRef, ChainActorRef<ChainActor>>::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool.clone(),
            chain.clone(),
        );

    let process_actor = ProcessActor::launch(Arc::clone(&peer_info), chain.clone(), network.clone(), bus.clone()).unwrap();
    let download_actor =
        DownloadActor::launch(peer_info, chain.clone(), network.clone(), bus.clone()).expect("launch DownloadActor failed.");
    let _sync = SyncActor::launch(bus.clone(), process_actor, download_actor).unwrap();

    for _i in 0..5 as usize {
        txpool
            .clone()
            .add(SignedUserTransaction::mock())
            .await
            .unwrap();
        delay_for(Duration::from_millis(1000)).await;
    }
    let number = chain.clone().current_header().await.unwrap().number();
    println!("{}", number);
    assert!(number > 4);
}

#[actix_rt::test]
async fn test_miner_with_ondemand_pacemaker() {
    let peer_info = Arc::new(PeerInfo::random());
    let mut conf = NodeConfig::default();
    conf.miner.pacemaker_strategy = PacemakerStrategy::Ondemand;
    let config = Arc::new(conf);
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let seq_number_client = CachedSeqNumberClient::new(storage.clone());
    let txpool = TxPool::start(seq_number_client);
    let key_pair = config::gen_keypair();
    let _address = AccountAddress::from_public_key(&key_pair.public_key);
    let network = NetworkActor::launch(config.clone(), bus.clone(), txpool.clone(), key_pair);
    let chain = ChainActor::launch(config.clone(), storage.clone(), Some(network.clone())).unwrap();

    let tmp = txpool.clone();
    let handle = thread::Builder::new()
        .spawn(move || {
            println!("begin subscribe_txns");
            let fut = async move {
                println!("do subscribe_txns");
                let t = tmp.subscribe_txns().await.unwrap();
                println!("done subscribe_txns");
                t
            };
            //let tx = System::builder().build().block_on(fut);

            let mut rt = Runtime::new().expect("Can not create Runtime");
            let tx = rt.block_on(fut);
            println!("end subscribe_txns");
            tx
        });
    let a = handle.unwrap().join().unwrap();

    let _miner =
        MinerActor::<DummyConsensus, MockExecutor, TxPoolRef, ChainActorRef<ChainActor>>::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool.clone(),
            chain.clone(),
        );

    // let process_actor = ProcessActor::launch(Arc::clone(&peer_info), chain.clone(), network.clone(), bus.clone()).unwrap();
    // let download_actor =
    //     DownloadActor::launch(peer_info, chain.clone(), network.clone(), bus.clone()).expect("launch DownloadActor failed.");
    // let _sync = SyncActor::launch(bus.clone(), process_actor, download_actor).unwrap();
    //
    // for _i in 0..1 as usize {
    //     txpool
    //         .clone()
    //         .add(SignedUserTransaction::mock())
    //         .await
    //         .unwrap();
    //     delay_for(Duration::from_millis(1000)).await;
    // }
    //
    // let number = chain.clone().current_header().await.unwrap().number();
    // println!("{}", number);
}
