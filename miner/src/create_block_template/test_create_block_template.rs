// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService, Inner};
use bus::BusActor;
use chain::BlockChain;
use consensus::Consensus;
use starcoin_account_api::AccountInfo;
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_txpool::TxPoolService;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

#[stest::test]
fn test_create_block_template() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, startup_info, genesis_id) =
        StarcoinGenesis::init_storage_for_test(node_config.net())
            .expect("init storage by genesis fail.");

    //TODO mock txpool after refactor txpool by service reigstry.
    let chain_header = storage
        .get_block_header_by_hash(startup_info.master)
        .unwrap()
        .unwrap();
    //TODO mock txpool after refactor txpool by service reigstry.
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);
    let miner_account = AccountInfo::random();
    let inner = Inner::new(
        node_config.net(),
        storage,
        genesis_id,
        txpool,
        None,
        miner_account,
    )
    .unwrap();

    let block_template = inner.create_block_template().unwrap();
    assert_eq!(block_template.parent_hash, genesis_id);
    assert_eq!(block_template.parent_hash, *startup_info.get_master());
    assert_eq!(block_template.number, 1);
}

#[stest::test]
fn test_do_uncles() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let times = 2;

    let miner_account = AccountInfo::random();
    // master
    let mut head_id = genesis_id;
    let mut master_inner = None;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);

    for _i in 0..times {
        let mut master =
            BlockChain::new(node_config.net().consensus(), head_id, storage.clone()).unwrap();

        let tmp_inner = Inner::new(
            node_config.net(),
            storage.clone(),
            head_id,
            txpool.clone(),
            None,
            miner_account.clone(),
        )
        .unwrap();

        let block_template = tmp_inner.create_block_template().unwrap();

        let block = node_config
            .net()
            .consensus()
            .create_block(&master, block_template)
            .unwrap();
        head_id = block.id();
        master.apply(block).unwrap();
        master_inner = Some(tmp_inner);
    }

    // branch
    for _i in 0..times {
        let mut branch =
            BlockChain::new(node_config.net().consensus(), genesis_id, storage.clone()).unwrap();
        let inner = Inner::new(
            node_config.net(),
            storage.clone(),
            genesis_id,
            txpool.clone(),
            None,
            miner_account.clone(),
        )
        .unwrap();

        let block_template = inner.create_block_template().unwrap();
        let uncle_block = node_config
            .net()
            .consensus()
            .create_block(&branch, block_template)
            .unwrap();
        let uncle_block_header = uncle_block.header().clone();
        branch.apply(uncle_block).unwrap();

        master_inner
            .as_mut()
            .unwrap()
            .insert_uncle(uncle_block_header);
    }

    // uncles
    {
        let master = BlockChain::new(node_config.net().consensus(), head_id, storage).unwrap();
        let block_template = master_inner
            .as_ref()
            .unwrap()
            .create_block_template()
            .unwrap();
        let block = node_config
            .net()
            .consensus()
            .create_block(&master, block_template)
            .unwrap();
        assert_eq!(block.uncles().unwrap().len(), times);
    }
}

#[stest::test]
fn test_new_head() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let times = 10;

    let miner_account = AccountInfo::random();
    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);

    let mut master_inner = Inner::new(
        node_config.net(),
        storage,
        genesis_id,
        txpool,
        None,
        miner_account,
    )
    .unwrap();

    for i in 0..times {
        let block_template = master_inner.create_block_template().unwrap();
        let block = node_config
            .net()
            .consensus()
            .create_block(&master_inner.chain, block_template)
            .unwrap();
        (&mut master_inner.chain).apply(block.clone()).unwrap();
        if i % 2 == 0 {
            master_inner.update_chain(block).unwrap();
        }
        assert_eq!(master_inner.chain.current_header().number(), i + 1);
    }
}

#[stest::test]
fn test_new_branch() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let times = 5;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);

    let miner_account = AccountInfo::random();
    // master

    let mut master_inner = Inner::new(
        node_config.net(),
        storage.clone(),
        genesis_id,
        txpool.clone(),
        None,
        miner_account.clone(),
    )
    .unwrap();
    for _i in 0..times {
        let block_template = master_inner.create_block_template().unwrap();
        let block = node_config
            .net()
            .consensus()
            .create_block(&master_inner.chain, block_template)
            .unwrap();
        (&mut master_inner.chain).apply(block.clone()).unwrap();
    }

    // branch
    let mut new_head_id = genesis_id;
    for i in 0..(times * 2) {
        let mut branch =
            BlockChain::new(node_config.net().consensus(), new_head_id, storage.clone()).unwrap();
        let inner = Inner::new(
            node_config.net(),
            storage.clone(),
            new_head_id,
            txpool.clone(),
            None,
            miner_account.clone(),
        )
        .unwrap();
        let block_template = inner.create_block_template().unwrap();
        let new_block = node_config
            .net()
            .consensus()
            .create_block(&branch, block_template)
            .unwrap();
        new_head_id = new_block.id();
        branch.apply(new_block.clone()).unwrap();

        if i > times {
            master_inner.update_chain(new_block).unwrap();
            assert_eq!(master_inner.chain.current_header().number(), i + 1);
        }
    }
}

#[stest::test]
async fn test_create_block_template_actor() {
    let bus = BusActor::launch();
    let node_config = Arc::new(NodeConfig::random_for_test());
    let registry = RegistryService::launch();
    registry.put_shared(bus).await.unwrap();
    registry.put_shared(node_config.clone()).await.unwrap();

    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    //TODO mock txpool.
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);
    registry.put_shared(txpool).await.unwrap();

    registry.put_shared(storage).await.unwrap();
    registry
        .registry_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let create_block_template_service = registry
        .registry::<CreateBlockTemplateService>()
        .await
        .unwrap();
    let response = create_block_template_service
        .send(CreateBlockTemplateRequest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.number, 1);
}
