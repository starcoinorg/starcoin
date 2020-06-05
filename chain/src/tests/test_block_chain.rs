use crate::{test_helper, BlockChain, ChainActor, ChainActorRef, ChainAsyncService, SyncMetadata};
use anyhow::Result;
use bus::BusActor;
use config::NodeConfig;
use consensus::dev::{DevConsensus, DummyHeader};
use futures_timer::Delay;
use logger::prelude::*;
use starcoin_genesis::Genesis;
use starcoin_wallet_api::WalletAccount;
use std::{sync::Arc, time::Duration};
use storage::{cache_storage::CacheStorage, storage::StorageInstance, Storage};
use traits::{ChainReader, ChainWriter, Consensus};
use txpool::TxPool;

async fn gen_master_chain(
    times: u64,
    delay: bool,
) -> (ChainActorRef<DevConsensus>, Arc<NodeConfig>) {
    let node_config = NodeConfig::random_for_test();
    let node_config = Arc::new(node_config);
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    let genesis = Genesis::build(node_config.net()).unwrap();
    let startup_info = genesis.execute(storage.clone()).unwrap();
    let bus = BusActor::launch();
    let txpool_service = {
        let best_block_id = *startup_info.get_master();
        TxPool::start(
            node_config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
        .get_service()
    };
    let sync_metadata = SyncMetadata::new(node_config.clone(), bus.clone());
    let chain = ChainActor::<DevConsensus>::launch(
        node_config.clone(),
        startup_info.clone(),
        storage.clone(),
        None,
        bus.clone(),
        txpool_service,
        sync_metadata,
    )
    .unwrap();
    let miner_account = WalletAccount::random();
    if times > 0 {
        for _i in 0..times {
            let startup_info = chain.clone().master_startup_info().await.unwrap();
            let block_chain = BlockChain::<DevConsensus, Storage>::new(
                node_config.clone(),
                startup_info.master,
                storage.clone(),
            )
            .unwrap();
            let block_template = chain
                .clone()
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.get_auth_key().prefix().to_vec()),
                    None,
                    Vec::new(),
                )
                .await
                .unwrap()
                .unwrap();
            let block =
                DevConsensus::create_block(node_config.clone(), &block_chain, block_template)
                    .unwrap();
            let _ = chain.clone().try_connect(block).await.unwrap();
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
    let (chain, _) = gen_master_chain(times, false).await;
    assert_eq!(
        chain.master_head_header().await.unwrap().unwrap().number(),
        times
    );
}

#[actix_rt::test]
async fn test_block_chain_forks() {
    ::logger::init_for_test();
    let times = 5;
    let (chain, _conf) = gen_master_chain(times, true).await;
    let mut parent_hash = chain.clone().master_block_by_number(0).await.unwrap().id();
    let miner_account = WalletAccount::random();
    if times > 0 {
        for i in 0..(times + 1) {
            Delay::new(Duration::from_millis(1000)).await;
            //TODO optimize this logic, use a more clear method to simulate chain difficulty and fork.
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
                .unwrap()
                .into_block(DummyHeader {}, 10000.into());
            info!(
                "{}:{:?}:{:?}:{:?}",
                i,
                parent_hash,
                block.header().id(),
                block.header().parent_hash()
            );
            parent_hash = block.header().id();
            let _ = chain.clone().try_connect(block).await.unwrap();
        }
    }

    assert_eq!(
        chain.master_head_header().await.unwrap().unwrap().id(),
        parent_hash
    )
}

#[stest::test]
async fn test_chain_apply() -> Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test::<DevConsensus>(config.clone())?;
    let header = block_chain.current_header();
    debug!("genesis header: {:?}", header);

    let miner_account = WalletAccount::random();
    let (block_template, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        Some(header.id()),
        vec![],
    )?;

    let new_block = DevConsensus::create_block(config, &block_chain, block_template)?;

    // block_chain.txn_accumulator.append(&[HashValue::random()])?;
    // block_chain.txn_accumulator.flush()?;
    //
    // block_chain.block_accumulator.append(&[new_block.id()])?;
    // block_chain.block_accumulator.flush()?;
    block_chain.apply(new_block)?;
    // let header1 = block_chain.current_header();
    // debug!("block 1 header: {:?}", header1);
    // assert_ne!(header.state_root(), header1.state_root());
    Ok(())
}
