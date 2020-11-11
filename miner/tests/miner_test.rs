// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus::Consensus;
use futures::executor::block_on;
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_miner::{CreateBlockTemplateRequest, CreateBlockTemplateService, MinerService};
use starcoin_service_registry::bus::Bus;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_txpool::TxPoolService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::delay_for;
use types::{
    block::BlockTemplate,
    system_events::{GenerateBlockEvent, MintBlockEvent, NewHeadBlock, SubmitSealEvent},
    U256,
};

#[stest::test]
fn test_miner() {
    let mut config = NodeConfig::random_for_test();
    config.miner.enable_miner_client = false;
    let config = Arc::new(config);
    let handle = test_helper::run_node_by_config(config.clone()).unwrap();
    let bus = handle.bus().unwrap();
    let time_service = config.net().time_service();
    let fut = async move {
        let new_block_receiver = bus.oneshot::<NewHeadBlock>().await.unwrap();
        bus.broadcast(GenerateBlockEvent::new(false)).unwrap();
        // mint client handle mint block event
        let mint_block_event = bus
            .oneshot::<MintBlockEvent>()
            .await
            .unwrap()
            .await
            .unwrap();
        let nonce = mint_block_event.strategy.solve_consensus_nonce(
            &mint_block_event.minting_blob,
            mint_block_event.difficulty,
            time_service.as_ref(),
        );
        // mint client submit seal
        bus.broadcast(SubmitSealEvent {
            nonce,
            minting_blob: mint_block_event.minting_blob.clone(),
        })
        .unwrap();
        let mined_block = new_block_receiver.await.unwrap().0.get_block().clone();
        assert_eq!(mined_block.header.nonce, nonce);
        let minting_blob =
            BlockTemplate::from_block(mined_block).as_pow_header_blob(mint_block_event.difficulty);
        assert_eq!(mint_block_event.minting_blob, minting_blob);
        handle.stop().unwrap();
    };
    block_on(fut);
}

#[stest::test]
async fn test_miner_service() {
    let mut config = NodeConfig::random_for_test();
    config.miner.enable_mint_empty_block = true;
    let registry = RegistryService::launch();
    let node_config = Arc::new(config.clone());
    registry.put_shared(node_config.clone()).await.unwrap();
    let (storage, _startup_info, genesis_hash) =
        Genesis::init_storage_for_test(config.net()).unwrap();
    registry.put_shared(storage.clone()).await.unwrap();

    let chain_header = storage
        .get_block_header_by_hash(genesis_hash)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);
    registry.put_shared(txpool).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let template = registry
        .register::<CreateBlockTemplateService>()
        .await
        .unwrap();
    let response = template
        .send(CreateBlockTemplateRequest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.number, 1);

    let miner = registry.register::<MinerService>().await;
    assert!(miner.is_ok());

    let miner = miner.unwrap();
    miner.notify(GenerateBlockEvent::new(false)).unwrap();

    delay_for(Duration::from_millis(200)).await;
    miner.notify(GenerateBlockEvent::new(true)).unwrap();
    delay_for(Duration::from_millis(200)).await;
    // Generate a event
    let diff = U256::from(1024);
    let minting_blob = vec![0u8; 76];

    let nonce = config
        .net()
        .genesis_config()
        .consensus()
        .solve_consensus_nonce(&minting_blob, diff, config.net().time_service().as_ref());
    miner
        .notify(SubmitSealEvent::new(minting_blob, nonce))
        .unwrap();

    registry.shutdown_system().await.unwrap();
}
