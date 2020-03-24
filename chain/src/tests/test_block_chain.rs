use crate::{to_block_chain_collection, BlockChain, ChainActor, ChainActorRef, ChainAsyncService};
use anyhow::Result;
use bus::BusActor;
use config::NodeConfig;
use consensus::dummy::DummyHeader;
use consensus::{difficult, dummy::DummyConsensus, Consensus};
use executor::executor::mock_create_account_txn;
use executor::mock_executor::MockExecutor;
use futures::channel::oneshot;
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_genesis::Genesis;
use std::{sync::Arc, time::Duration};
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::StarcoinStorage;
use traits::{ChainReader, ChainWriter};
use txpool::TxPoolRef;
use types::account_address::AccountAddress;
use types::transaction::SignedUserTransaction;

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

fn gen_txs() -> Vec<SignedUserTransaction> {
    let tx = mock_create_account_txn()
        .as_signed_user_txn()
        .unwrap()
        .clone();
    let mut txs = Vec::new();
    txs.push(tx);
    txs
}

async fn gen_head_chain(times: u64, delay: bool) -> ChainActorRef<MockExecutor, DummyConsensus> {
    let node_config = NodeConfig::random_for_test();
    let conf = Arc::new(node_config);
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage =
        Arc::new(StarcoinStorage::new(cache_storage.clone(), db_storage.clone()).unwrap());
    let genesis = Genesis::new::<MockExecutor, DummyConsensus, StarcoinStorage>(
        conf.clone(),
        storage.clone(),
    )
    .unwrap();
    let bus = BusActor::launch();
    let txpool = {
        let best_block_id = genesis.startup_info().head.get_head();
        TxPoolRef::start(
            conf.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let chain = ChainActor::<MockExecutor, DummyConsensus>::launch(
        conf.clone(),
        genesis.startup_info().clone(),
        storage.clone(),
        None,
        bus.clone(),
        txpool.clone(),
    )
    .unwrap();
    if times > 0 {
        for _i in 0..times {
            let block_template = chain
                .clone()
                .create_block_template(None, gen_txs())
                .await
                .unwrap();
            let (_sender, receiver) = oneshot::channel();

            let startup_info = chain.clone().master_startup_info().await.unwrap();
            let collection = to_block_chain_collection(
                conf.clone(),
                startup_info,
                storage.clone(),
                txpool.clone(),
            )
            .unwrap();
            let block_chain =
                BlockChain::<MockExecutor, DummyConsensus, StarcoinStorage, TxPoolRef>::new(
                    conf.clone(),
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
                DummyConsensus::create_block(conf.clone(), &block_chain, block_template, receiver)
                    .unwrap();
            chain.clone().try_connect(block).await.unwrap();
            if delay {
                Delay::new(Duration::from_millis(1000)).await;
            }
        }
    }

    chain
}

#[actix_rt::test]
async fn test_block_chain_head() {
    let times = 5;
    let chain = gen_head_chain(times, false).await;
    assert_eq!(chain.master_head_header().await.unwrap().number(), times);
}

#[actix_rt::test]
async fn test_block_chain_forks() {
    let times = 5;
    let chain = gen_head_chain(times, true).await;
    let mut parent_hash = chain
        .clone()
        .master_startup_info()
        .await
        .unwrap()
        .head
        .branch_id();

    if times > 0 {
        for i in 0..(times + 1) {
            Delay::new(Duration::from_millis(1000)).await;
            let block = chain
                .clone()
                .create_block_template(Some(parent_hash), gen_txs())
                .await
                .unwrap()
                .into_block(DummyHeader {});
            println!(
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
    let node_config = NodeConfig::random_for_test();
    let config = Arc::new(node_config);
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage =
        Arc::new(StarcoinStorage::new(cache_storage.clone(), db_storage.clone()).unwrap());
    let genesis = Genesis::new::<MockExecutor, DummyConsensus, StarcoinStorage>(
        config.clone(),
        storage.clone(),
    )?;
    let bus = BusActor::launch();
    let txpool = {
        let best_block_id = genesis.startup_info().head.get_head();
        TxPoolRef::start(
            config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let collection = to_block_chain_collection(
        config.clone(),
        genesis.startup_info().clone(),
        storage.clone(),
        txpool.clone(),
    )?;
    let mut block_chain =
        BlockChain::<MockExecutor, DummyConsensus, StarcoinStorage, TxPoolRef>::new(
            config.clone(),
            genesis.startup_info().head.clone(),
            storage,
            txpool,
            collection,
        )?;
    let header = block_chain.current_header();
    debug!("genesis header: {:?}", header);
    let difficulty = difficult::get_next_work_required(&block_chain);
    let block_template = block_chain.create_block_template(None, difficulty, vec![])?;
    let (_sender, receiver) = futures::channel::oneshot::channel();
    let new_block =
        DummyConsensus::create_block(config.clone(), &block_chain, block_template, receiver)?;
    block_chain.apply(new_block)?;
    let header1 = block_chain.current_header();
    debug!("block 1 header: {:?}", header1);
    assert_ne!(header.state_root(), header1.state_root());
    Ok(())
}
