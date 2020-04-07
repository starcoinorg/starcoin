use crate::{
    to_block_chain_collection, BlockChain, ChainActor, ChainActorRef, ChainAsyncService,
    SyncMetadata,
};
use anyhow::Result;
use bus::BusActor;
use config::NodeConfig;
use consensus::dummy::DummyHeader;
use consensus::{difficult, dummy::DummyConsensus};
use executor::executor::Executor;
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_genesis::Genesis;
use starcoin_wallet_api::WalletAccount;
use std::{sync::Arc, time::Duration};
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use traits::Consensus;
use traits::{ChainReader, ChainWriter};
use txpool::TxPoolRef;

async fn gen_head_chain(
    times: u64,
    delay: bool,
) -> (ChainActorRef<Executor, DummyConsensus>, Arc<NodeConfig>) {
    let node_config = NodeConfig::random_for_test();
    let node_config = Arc::new(node_config);
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::build(node_config.net()).unwrap();
    let startup_info = genesis.execute(storage.clone()).unwrap();
    let bus = BusActor::launch();
    let txpool = {
        let best_block_id = startup_info.head.get_head();
        TxPoolRef::start(
            node_config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let sync_metadata = SyncMetadata::new(node_config.clone());
    let chain = ChainActor::<Executor, DummyConsensus>::launch(
        node_config.clone(),
        startup_info.clone(),
        storage.clone(),
        None,
        bus.clone(),
        txpool.clone(),
        sync_metadata,
    )
    .unwrap();
    let miner_account = WalletAccount::random();
    if times > 0 {
        for _i in 0..times {
            let block_template = chain
                .clone()
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.get_auth_key().prefix().to_vec()),
                    None,
                    Vec::new(),
                )
                .await
                .unwrap();
            let startup_info = chain.clone().master_startup_info().await.unwrap();
            let collection = to_block_chain_collection(
                node_config.clone(),
                startup_info,
                storage.clone(),
                txpool.clone(),
            )
            .unwrap();
            let block_chain = BlockChain::<Executor, DummyConsensus, Storage, TxPoolRef>::new(
                node_config.clone(),
                collection
                    .clone()
                    .get_master()
                    .borrow()
                    .get(0)
                    .unwrap()
                    .get_chain_info(),
                storage.clone(),
                txpool.clone(),
                collection,
            )
            .unwrap();
            let block =
                DummyConsensus::create_block(node_config.clone(), &block_chain, block_template)
                    .unwrap();
            chain.clone().try_connect(block).await.unwrap();
            if delay {
                Delay::new(Duration::from_millis(1000)).await;
            }
        }
    }

    (chain, node_config)
}

#[actix_rt::test]
async fn test_block_chain_head() {
    ::logger::init_for_test();
    let times = 10;
    let (chain, _) = gen_head_chain(times, false).await;
    assert_eq!(chain.master_head_header().await.unwrap().number(), times);
}

#[actix_rt::test]
async fn test_block_chain_forks() {
    ::logger::init_for_test();
    let times = 5;
    let (chain, _conf) = gen_head_chain(times, true).await;
    let mut parent_hash = chain
        .clone()
        .master_startup_info()
        .await
        .unwrap()
        .head
        .branch_id();
    let miner_account = WalletAccount::random();
    if times > 0 {
        for i in 0..(times + 1) {
            Delay::new(Duration::from_millis(1000)).await;
            let block = chain
                .clone()
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.get_auth_key().prefix().to_vec()),
                    Some(parent_hash),
                    Vec::new(),
                )
                .await
                .unwrap()
                .into_block(DummyHeader {});
            info!(
                "{}:{:?}:{:?}:{:?}",
                i,
                parent_hash,
                block.header().id(),
                block.header().parent_hash()
            );
            parent_hash = block.header().id();
            chain.clone().try_connect(block).await.unwrap();
        }
    }

    assert_eq!(
        chain.master_head_header().await.unwrap().number(),
        (times + 1)
    )
}

#[stest::test]
async fn test_chain_apply() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::build(config.net()).unwrap();
    let startup_info = genesis.execute(storage.clone())?;
    let bus = BusActor::launch();
    let txpool = {
        let best_block_id = startup_info.head.get_head();
        TxPoolRef::start(
            config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let collection = to_block_chain_collection(
        config.clone(),
        startup_info.clone(),
        storage.clone(),
        txpool.clone(),
    )?;
    let mut block_chain = BlockChain::<Executor, DummyConsensus, Storage, TxPoolRef>::new(
        config.clone(),
        startup_info.head.clone(),
        storage,
        txpool,
        collection,
    )?;
    let header = block_chain.current_header();
    debug!("genesis header: {:?}", header);
    let difficulty = difficult::get_next_work_required(&block_chain);
    let miner_account = WalletAccount::random();
    let block_template = block_chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        None,
        difficulty,
        vec![],
    )?;
    let new_block = DummyConsensus::create_block(config.clone(), &block_chain, block_template)?;
    block_chain.apply(new_block)?;
    let header1 = block_chain.current_header();
    debug!("block 1 header: {:?}", header1);
    assert_ne!(header.state_root(), header1.state_root());
    Ok(())
}
