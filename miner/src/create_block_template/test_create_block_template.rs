use crate::create_block_template::{CreateBlockTemplateActor, CreateBlockTemplateRequest, Inner};
use bus::BusActor;
use chain::BlockChain;
use config::NodeConfig;
use consensus::Consensus;
use starcoin_account_api::AccountInfo;
use starcoin_genesis::Genesis as StarcoinGenesis;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

#[stest::test]
fn test_create_block_template() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, startup_info, genesis_id) =
        StarcoinGenesis::init_storage_for_test(node_config.net())
            .expect("init storage by genesis fail.");
    let inner = Inner::new(genesis_id, storage, node_config.net()).unwrap();
    let miner_account = AccountInfo::random();
    let (block_template, _) = inner
        .create_block_template(
            1_000_000,
            *miner_account.address(),
            Some(miner_account.get_auth_key().prefix().to_vec()),
            Vec::new(),
        )
        .unwrap();
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
    for _i in 0..times {
        let mut master =
            BlockChain::new(node_config.net().consensus(), head_id, storage.clone()).unwrap();
        let tmp_inner = Inner::new(head_id, storage.clone(), node_config.net()).unwrap();
        let (block_template, _) = tmp_inner
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
            .unwrap();

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
        let inner = Inner::new(genesis_id, storage.clone(), node_config.net()).unwrap();
        let (block_template, _) = inner
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
            .unwrap();
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
        let (block_template, _) = master_inner
            .as_ref()
            .unwrap()
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
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
    let mut master_inner = Inner::new(genesis_id, storage, node_config.net()).unwrap();
    for i in 0..times {
        let (block_template, _) = master_inner
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
            .unwrap();
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

    let miner_account = AccountInfo::random();
    // master
    let mut master_inner = Inner::new(genesis_id, storage.clone(), node_config.net()).unwrap();
    for _i in 0..times {
        let (block_template, _) = master_inner
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
            .unwrap();
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
        let inner = Inner::new(new_head_id, storage.clone(), node_config.net()).unwrap();
        let (block_template, _) = inner
            .create_block_template(
                1_000_000,
                *miner_account.address(),
                Some(miner_account.get_auth_key().prefix().to_vec()),
                Vec::new(),
            )
            .unwrap();
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
    let (storage, _, genesis_id) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let miner_account = AccountInfo::random();
    let create_block_template_address =
        CreateBlockTemplateActor::launch(genesis_id, node_config.net(), bus, storage).unwrap();
    let response = create_block_template_address
        .send(CreateBlockTemplateRequest::new(
            1_000_000,
            *miner_account.address(),
            Some(miner_account.get_auth_key().prefix().to_vec()),
            Vec::new(),
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.block_template.number, 1);
}
