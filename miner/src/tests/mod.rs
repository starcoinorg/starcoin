use crate::MinerActor;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::NodeConfig;
use consensus::{dummy::DummyConsensus, Consensus};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use sync::{DownloadActor, ProcessActor, SyncActor};
use tokio::time::{delay_for, Duration};
use traits::{AsyncChain, TxPoolAsyncService};
use txpool::{CachedSeqNumberClient, TxPool, TxPoolActor, TxPoolRef};
use types::{peer_info::PeerInfo, transaction::SignedUserTransaction};

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
    let chain = ChainActor::launch(config.clone(), storage.clone()).unwrap();
    let _miner =
        MinerActor::<DummyConsensus, MockExecutor, TxPoolRef, ChainActorRef<ChainActor>>::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool.clone(),
            chain.clone(),
        );

    let process_actor = ProcessActor::launch(Arc::clone(&peer_info), chain.clone()).unwrap();
    let download_actor =
        DownloadActor::launch(peer_info, chain.clone()).expect("launch DownloadActor failed.");
    let sync = SyncActor::launch(bus, process_actor, download_actor).unwrap();

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
