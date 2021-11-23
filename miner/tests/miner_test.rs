// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus::Consensus;
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_miner::{
    BlockBuilderService, BlockHeaderExtra, BlockTemplateRequest, MinerService, SubmitSealRequest,
};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_txpool::TxPoolService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::delay_for;
use types::{system_events::GenerateBlockEvent, U256};

#[stest::test]
async fn test_miner_service() {
    let mut config = NodeConfig::random_for_test();
    config.miner.disable_mint_empty_block = Some(false);
    let registry = RegistryService::launch();
    let node_config = Arc::new(config.clone());
    registry.put_shared(node_config.clone()).await.unwrap();
    let (storage, _chain_info, genesis) = Genesis::init_storage_for_test(config.net()).unwrap();
    registry.put_shared(storage.clone()).await.unwrap();

    let genesis_hash = genesis.block().id();
    let chain_header = storage
        .get_block_header_by_hash(genesis_hash)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);
    registry.put_shared(txpool).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let template = registry.register::<BlockBuilderService>().await.unwrap();
    let response = template
        .send(BlockTemplateRequest)
        .await
        .unwrap()
        .unwrap()
        .template;
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
        .try_send(SubmitSealRequest::new(
            minting_blob,
            nonce,
            BlockHeaderExtra::new([0u8; 4]),
        ))
        .unwrap();

    registry.shutdown_system().await.unwrap();
}
