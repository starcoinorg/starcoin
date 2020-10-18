// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::create_block_template::{CreateBlockTemplateRequest, CreateBlockTemplateService, Inner};

use chain::BlockChain;
use consensus::Consensus;
use logger::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_account_service::AccountService;
use starcoin_config::{temp_path, NodeConfig, StarcoinOpt};
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_txpool::TxPoolService;
use starcoin_vm_types::genesis_config::ChainNetwork;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

#[stest::test]
fn test_create_block_template() {
    test_create_block_template_by_net(ChainNetwork::TEST);
    test_create_block_template_by_net(ChainNetwork::DEV);
    test_create_block_template_by_net(ChainNetwork::HALLEY);
    //test_create_block_template_by_net(ChainNetwork::PROXIMA);
}

fn test_create_block_template_by_net(net: ChainNetwork) {
    debug!("test_create_block_template_by_net {:?}", net);
    let mut opt = StarcoinOpt::default();
    let temp_path = temp_path();
    opt.net = Some(net);
    opt.data_dir = Some(temp_path.path().to_path_buf());

    let node_config = Arc::new(NodeConfig::load_with_opt(&opt).unwrap());
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

#[stest::test(timeout = 120)]
fn test_switch_master() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let times = 10;

    let miner_account = AccountInfo::random();
    // master
    let mut head_id = genesis_id;
    let mut master_inner = None;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header);

    let net = node_config.net();
    for i in 0..times {
        let mut master = BlockChain::new(net.time_service(), head_id, storage.clone()).unwrap();

        let mut tmp_inner = Inner::new(
            net,
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

        let block_header = block.header().clone();
        master.apply(block.clone()).unwrap();
        tmp_inner.update_chain(block).unwrap();
        master_inner = Some(tmp_inner);

        if i != (times - 1) {
            head_id = block_header.id();
        } else {
            master_inner
                .as_mut()
                .unwrap()
                .insert_uncle(block_header.clone());
        }
    }

    for i in 0..3 {
        let mut new_master = BlockChain::new(net.time_service(), head_id, storage.clone()).unwrap();

        let block_template = if i == 0 {
            let tmp = Inner::new(
                net,
                storage.clone(),
                head_id,
                txpool.clone(),
                None,
                miner_account.clone(),
            )
            .unwrap();

            tmp.create_block_template().unwrap()
        } else {
            master_inner
                .as_ref()
                .unwrap()
                .create_block_template()
                .unwrap()
        };

        let block = node_config
            .net()
            .consensus()
            .create_block(&new_master, block_template)
            .unwrap();

        new_master.apply(block.clone()).unwrap();

        head_id = block.id();
        if i == 0 {
            let block_header = block.header().clone();
            assert_eq!(master_inner.as_ref().unwrap().uncles.len(), 1);
            master_inner.as_mut().unwrap().update_chain(block).unwrap();
            master_inner.as_mut().unwrap().insert_uncle(block_header);
        } else if i == 1 {
            assert_eq!(master_inner.as_ref().unwrap().uncles.len(), 2);
            assert!(block.body.uncles.is_some());
            assert_eq!(block.body.uncles.as_ref().unwrap().len(), 1);
            master_inner.as_mut().unwrap().update_chain(block).unwrap();
        } else if i == 2 {
            assert_eq!(master_inner.as_ref().unwrap().uncles.len(), 2);
            assert!(block.body.uncles.is_none());
        }
    }
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

    let net = node_config.net();
    for _i in 0..times {
        let mut master = BlockChain::new(net.time_service(), head_id, storage.clone()).unwrap();

        let mut tmp_inner = Inner::new(
            net,
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
        master.apply(block.clone()).unwrap();
        tmp_inner.update_chain(block).unwrap();
        master_inner = Some(tmp_inner);
    }

    // branch
    for _i in 0..times {
        let mut branch = BlockChain::new(net.time_service(), genesis_id, storage.clone()).unwrap();
        let inner = Inner::new(
            net,
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
    for i in 0..times {
        let mut master = BlockChain::new(net.time_service(), head_id, storage.clone()).unwrap();

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
        if i == 0 {
            assert_eq!(block.uncles().unwrap().len(), times);
        } else {
            assert!(block.uncles().is_none());
        }
        head_id = block.id();
        master.apply(block.clone()).unwrap();
        master_inner.as_mut().unwrap().update_chain(block).unwrap();
    }
}

#[stest::test(timeout = 120)]
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

#[stest::test(timeout = 120)]
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
    let net = node_config.net();
    for i in 0..(times * 2) {
        let mut branch = BlockChain::new(net.time_service(), new_head_id, storage.clone()).unwrap();
        let inner = Inner::new(
            net,
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

#[stest::test(timeout = 480)]
async fn test_create_block_template_actor() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let registry = RegistryService::launch();
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
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let create_block_template_service = registry
        .register::<CreateBlockTemplateService>()
        .await
        .unwrap();
    let response = create_block_template_service
        .send(CreateBlockTemplateRequest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.number, 1);
}
