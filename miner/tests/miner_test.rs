// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_dag::service::pruning_point_service::PruningPointService;
use starcoin_genesis::Genesis;
use starcoin_miner::{
    BlockBuilderService, BlockHeaderExtra, BlockTemplateRequest, MinerService, SubmitSealRequest,
};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_txpool::TxPoolService;
use starcoin_types::{system_events::GenerateBlockEvent, U256};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[stest::test]
async fn test_miner_service() {
    let mut config = NodeConfig::random_for_dag_test();
    config.miner.disable_mint_empty_block = Some(false);
    let registry = RegistryService::launch();
    let node_config = Arc::new(config.clone());
    registry.put_shared(node_config.clone()).await.unwrap();
    let (storage, _chain_info, genesis, dag) =
        Genesis::init_storage_for_test(config.net()).unwrap();
    registry.put_shared(storage.clone()).await.unwrap();
    registry.put_shared(dag).await.unwrap();

    let genesis_hash = genesis.block().id();
    registry.put_shared(genesis).await.unwrap();
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

    registry.register::<PruningPointService>().await.unwrap();

    registry
        .register::<BlockConnectorService<TxPoolService>>()
        .await
        .unwrap();

    let template = registry.register::<BlockBuilderService>().await.unwrap();
    let response = template
        .send(BlockTemplateRequest)
        .await
        .unwrap()
        .await
        .unwrap()
        .template;
    assert_eq!(response.number, 1);

    let miner = registry.register::<MinerService>().await;
    assert!(miner.is_ok());

    let miner = miner.unwrap();
    miner.notify(GenerateBlockEvent::new_break(false)).unwrap();

    sleep(Duration::from_millis(200)).await;
    miner.notify(GenerateBlockEvent::new_break(true)).unwrap();
    sleep(Duration::from_millis(200)).await;
    // Generate a event
    let diff = U256::from(1024);
    let minting_blob = vec![0u8; 76];

    let (nonce, block_level) = config
        .net()
        .genesis_config()
        .consensus()
        .solve_consensus_nonce(&minting_blob, diff, config.net().time_service().as_ref());
    miner
        .try_send(SubmitSealRequest::new(
            minting_blob,
            nonce,
            BlockHeaderExtra::new([0u8; 4]),
            block_level,
        ))
        .unwrap();

    registry.shutdown_system().await.unwrap();
}
