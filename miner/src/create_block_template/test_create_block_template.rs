// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::create_block_template::{
    load_block_permit_private_key, BlockBuilderService, BlockTemplateRequest, EmptyProvider, Inner,
};
use anyhow::Result;
use starcoin_account_api::{AccountInfo, AccountPublicKey};
use starcoin_account_service::AccountService;
use starcoin_chain::verifier::{BasicVerifier, NoneVerifier};
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_config::ChainNetworkID;
use starcoin_config::{temp_dir, NodeConfig, StarcoinOpt};
use starcoin_consensus::Consensus;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, ValidCryptoMaterialStringExt};
use starcoin_genesis::Genesis as StarcoinGenesis;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_storage::BlockStore;
use starcoin_time_service::MockTimeService;
use starcoin_txpool::TxPoolService;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::block_permit::{validate_block_permit, BlockPermitPolicy};
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::account_config::genesis_address;
use std::sync::Arc;

#[test]
fn test_block_permit_key_file_validation() -> Result<()> {
    let temp_path = temp_dir();
    let key_path = temp_path.path().join("permit.key");
    let private_key = Ed25519PrivateKey::try_from(&[7u8; 32][..])?;
    let policy =
        BlockPermitPolicy::new_for_test(1, AuthenticationKey::ed25519(&private_key.public_key()));
    let encoded =
        starcoin_account_api::AccountPrivateKey::Single(private_key).to_encoded_string()?;
    std::fs::write(&key_path, format!("  {}\n", encoded))?;
    assert!(load_block_permit_private_key(Some(&key_path), policy)?.is_some());

    let wrong_key = Ed25519PrivateKey::try_from(&[8u8; 32][..])?;
    let wrong_policy =
        BlockPermitPolicy::new_for_test(1, AuthenticationKey::ed25519(&wrong_key.public_key()));
    assert!(load_block_permit_private_key(Some(&key_path), wrong_policy).is_err());
    std::fs::write(&key_path, "not-a-private-key")?;
    assert!(load_block_permit_private_key(Some(&key_path), policy).is_err());
    assert!(load_block_permit_private_key(None, policy)?.is_none());
    Ok(())
}

#[stest::test(timeout = 120)]
fn test_active_block_permit_template_is_fail_closed_and_executes_outside_vm() -> Result<()> {
    let mut opt = StarcoinOpt::default();
    let temp_path = temp_dir();
    opt.net = Some(ChainNetworkID::MAIN);
    opt.base_data_dir = Some(temp_path.path().to_path_buf());
    let node_config = Arc::new(NodeConfig::load_with_opt(&opt)?);
    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())?;
    let genesis_id = genesis.block().id();
    let private_key = Ed25519PrivateKey::try_from(&[7u8; 32][..])?;
    let policy =
        BlockPermitPolicy::new_for_test(1, AuthenticationKey::ed25519(&private_key.public_key()));
    let miner_account = AccountInfo::new(
        genesis_address(),
        AccountPublicKey::Single(private_key.public_key()),
        true,
        false,
        false,
    );

    let keyless = Inner::new_with_block_permit_signer(
        node_config.net(),
        storage.clone(),
        genesis_id,
        EmptyProvider,
        None,
        miner_account.clone(),
        None,
        None,
        policy,
        None,
    )?;
    assert!(keyless.create_block_template().is_err());

    let wrong_key = Arc::new(Ed25519PrivateKey::try_from(&[8u8; 32][..])?);
    let wrong_signer = Inner::new_with_block_permit_signer(
        node_config.net(),
        storage.clone(),
        genesis_id,
        EmptyProvider,
        None,
        miner_account.clone(),
        None,
        None,
        policy,
        Some(wrong_key),
    )?;
    assert!(wrong_signer.create_block_template().is_err());

    let mut signer = Inner::new_with_block_permit_signer(
        node_config.net(),
        storage,
        genesis_id,
        EmptyProvider,
        None,
        miner_account,
        None,
        None,
        policy,
        Some(Arc::new(private_key)),
    )?;
    let response = signer.create_block_template()?;
    let template = response.template;
    assert_eq!(template.body.transactions.len(), 1);
    let permit = &template.body.transactions[0];
    assert_eq!(permit.sequence_number(), 0);
    assert_eq!(permit.max_gas_amount(), 0);
    assert_eq!(permit.gas_unit_price(), 0);
    assert_eq!(permit.expiration_timestamp_secs(), 0);
    assert_eq!(template.gas_used, 0);

    let block = template.clone().into_block(0, BlockHeaderExtra::default());
    validate_block_permit(
        policy,
        node_config.net().chain_id(),
        &response.parent,
        &block,
    )?;
    let missing_body = starcoin_types::block::BlockBody::new(vec![], None);
    let missing_header = block
        .header()
        .as_builder()
        .with_body_hash(missing_body.hash())
        .build();
    let missing_block = starcoin_types::block::Block::new(missing_header, missing_body);
    assert!(signer
        .chain
        .apply_with_verifier::<BasicVerifier>(missing_block.clone())
        .is_err());
    assert!(signer
        .chain
        .apply_with_verifier::<NoneVerifier>(missing_block.clone())
        .is_err());
    let mut legacy_chain = BlockChain::new(
        node_config.net().time_service(),
        genesis_id,
        signer.storage.clone(),
        None,
    )?;
    assert!(legacy_chain
        .apply_with_verifier::<BasicVerifier>(block.clone())
        .is_err());
    let no_save = signer
        .chain
        .verify_without_save::<BasicVerifier>(block.clone())?;
    assert_eq!(
        no_save
            .block_info()
            .get_txn_accumulator_info()
            .get_accumulator_root(),
        &template.txn_accumulator_root
    );
    let mut forged_connect_chain = BlockChain::new_with_block_permit_policy(
        node_config.net().time_service(),
        genesis_id,
        signer.storage.clone(),
        None,
        node_config.net().chain_id(),
        policy,
    )?;
    assert!(forged_connect_chain
        .connect(starcoin_types::block::ExecutedBlock::new(
            missing_block,
            no_save.block_info().clone(),
        ))
        .is_err());
    let executed = signer.chain.apply_with_verifier::<BasicVerifier>(block)?;
    assert_eq!(executed.header().state_root(), template.state_root);
    assert_eq!(executed.header().gas_used(), template.gas_used);
    assert_eq!(
        executed
            .block_info()
            .get_txn_accumulator_info()
            .get_accumulator_root(),
        &template.txn_accumulator_root
    );
    let permit_info = signer.chain.get_transaction_info(permit.id())?.unwrap();
    assert_eq!(permit_info.transaction_index, 1);
    assert_eq!(permit_info.gas_used(), 0);
    assert_eq!(
        permit_info.status(),
        &starcoin_types::vm_error::KeptVMStatus::Executed
    );
    assert_eq!(permit_info.state_root_hash(), template.state_root);
    assert!(signer
        .storage
        .get_contract_events(permit_info.id())?
        .unwrap()
        .is_empty());
    BlockChain::new_with_block_permit_policy(
        node_config.net().time_service(),
        executed.block().id(),
        signer.storage.clone(),
        None,
        node_config.net().chain_id(),
        policy,
    )?;
    Ok(())
}

#[stest::test]
fn test_create_block_template() {
    test_create_block_template_by_net(ChainNetworkID::TEST);
    test_create_block_template_by_net(ChainNetworkID::DEV);
    test_create_block_template_by_net(ChainNetworkID::HALLEY);
    //test_create_block_template_by_net(ChainNetwork::PROXIMA);
}

fn test_create_block_template_by_net(net: ChainNetworkID) {
    debug!("test_create_block_template_by_net {:?}", net);
    let mut opt = StarcoinOpt::default();
    let temp_path = temp_dir();
    opt.net = Some(net);
    opt.base_data_dir = Some(temp_path.path().to_path_buf());

    let node_config = Arc::new(NodeConfig::load_with_opt(&opt).unwrap());
    let (storage, chain_info, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let miner_account = AccountInfo::random();
    let inner = Inner::new(
        node_config.net(),
        storage,
        genesis_id,
        EmptyProvider,
        None,
        miner_account,
        None,
        None,
    )
    .unwrap();

    let block_template = inner.create_block_template().unwrap().template;
    assert_eq!(block_template.parent_hash, genesis_id);
    assert_eq!(block_template.parent_hash, chain_info.head().id());
    assert_eq!(block_template.number, 1);
}

#[stest::test(timeout = 120)]
fn test_switch_main() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let times = 10;

    let miner_account = AccountInfo::random();
    // main
    let mut head_id = genesis_id;
    let mut main_inner = None;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let net = node_config.net();
    for i in 0..times {
        let mut main = BlockChain::new(net.time_service(), head_id, storage.clone(), None).unwrap();

        let mut tmp_inner = Inner::new(
            net,
            storage.clone(),
            head_id,
            txpool.clone(),
            None,
            miner_account.clone(),
            None,
            None,
        )
        .unwrap();

        let block_template = tmp_inner.create_block_template().unwrap().template;

        let block = main
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();

        let block_header = block.header().clone();
        let executed_block = main.apply(block.clone()).unwrap();
        tmp_inner.update_chain(executed_block).unwrap();
        main_inner = Some(tmp_inner);

        if i != (times - 1) {
            head_id = block_header.id();
        } else {
            main_inner
                .as_mut()
                .unwrap()
                .insert_uncle(block_header.clone());
        }
    }

    for i in 0..3 {
        let mut new_main =
            BlockChain::new(net.time_service(), head_id, storage.clone(), None).unwrap();

        let block_template = if i == 0 {
            let tmp = Inner::new(
                net,
                storage.clone(),
                head_id,
                txpool.clone(),
                None,
                miner_account.clone(),
                None,
                None,
            )
            .unwrap();

            tmp.create_block_template().unwrap().template
        } else {
            main_inner
                .as_ref()
                .unwrap()
                .create_block_template()
                .unwrap()
                .template
        };

        let block = new_main
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();

        let executed_block = new_main.apply(block.clone()).unwrap();

        head_id = block.id();
        if i == 0 {
            let block_header = block.header().clone();
            assert_eq!(main_inner.as_ref().unwrap().uncles.len(), 1);
            main_inner
                .as_mut()
                .unwrap()
                .update_chain(executed_block)
                .unwrap();
            main_inner.as_mut().unwrap().insert_uncle(block_header);
        } else if i == 1 {
            assert_eq!(main_inner.as_ref().unwrap().uncles.len(), 2);
            assert!(block.body.uncles.is_some());
            assert_eq!(block.body.uncles.as_ref().unwrap().len(), 1);
            main_inner
                .as_mut()
                .unwrap()
                .update_chain(executed_block)
                .unwrap();
        } else if i == 2 {
            assert_eq!(main_inner.as_ref().unwrap().uncles.len(), 2);
            assert!(block.body.uncles.is_none());
        }
    }
}

#[stest::test]
fn test_do_uncles() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let times = 2;

    let miner_account = AccountInfo::random();
    // main
    let mut head_id = genesis_id;
    let mut main_inner = None;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let net = node_config.net();
    for _i in 0..times {
        let mut main = BlockChain::new(net.time_service(), head_id, storage.clone(), None).unwrap();

        let mut tmp_inner = Inner::new(
            net,
            storage.clone(),
            head_id,
            txpool.clone(),
            None,
            miner_account.clone(),
            None,
            None,
        )
        .unwrap();

        let block_template = tmp_inner.create_block_template().unwrap().template;

        let block = main
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        head_id = block.id();
        let executed_block = main.apply(block.clone()).unwrap();
        tmp_inner.update_chain(executed_block).unwrap();
        main_inner = Some(tmp_inner);
    }

    // branch
    for _i in 0..times {
        let mut branch =
            BlockChain::new(net.time_service(), genesis_id, storage.clone(), None).unwrap();
        let inner = Inner::new(
            net,
            storage.clone(),
            genesis_id,
            txpool.clone(),
            None,
            miner_account.clone(),
            None,
            None,
        )
        .unwrap();

        let block_template = inner.create_block_template().unwrap().template;
        let uncle_block = branch
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        let uncle_block_header = uncle_block.header().clone();
        branch.apply(uncle_block).unwrap();

        main_inner
            .as_mut()
            .unwrap()
            .insert_uncle(uncle_block_header);
    }

    // uncles
    for i in 0..times {
        let mut main = BlockChain::new(net.time_service(), head_id, storage.clone(), None).unwrap();

        let block_template = main_inner
            .as_ref()
            .unwrap()
            .create_block_template()
            .unwrap()
            .template;
        let block = main
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        if i == 0 {
            assert_eq!(block.uncles().unwrap().len(), times);
        } else {
            assert!(block.uncles().is_none());
        }
        head_id = block.id();
        let executed_block = main.apply(block.clone()).unwrap();
        main_inner
            .as_mut()
            .unwrap()
            .update_chain(executed_block)
            .unwrap();
    }
}

#[stest::test(timeout = 120)]
fn test_new_head() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let times = 10;

    let miner_account = AccountInfo::random();
    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let mut main_inner = Inner::new(
        node_config.net(),
        storage,
        genesis_id,
        txpool,
        None,
        miner_account,
        None,
        None,
    )
    .unwrap();

    for i in 0..times {
        let block_template = main_inner.create_block_template().unwrap().template;
        let block = main_inner
            .chain
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        let executed_block = main_inner.chain.apply(block.clone()).unwrap();
        if i % 2 == 0 {
            main_inner.update_chain(executed_block).unwrap();
        }
        assert_eq!(main_inner.chain.current_header().number(), i + 1);
    }
}

#[stest::test(timeout = 120)]
fn test_new_branch() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let times = 5;

    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);

    let miner_account = AccountInfo::random();
    // main

    let mut main_inner = Inner::new(
        node_config.net(),
        storage.clone(),
        genesis_id,
        txpool.clone(),
        None,
        miner_account.clone(),
        None,
        None,
    )
    .unwrap();
    for _i in 0..times {
        let block_template = main_inner.create_block_template().unwrap().template;
        let block = main_inner
            .chain
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        main_inner.chain.apply(block.clone()).unwrap();
    }

    // branch
    let mut new_head_id = genesis_id;
    let net = node_config.net();
    for i in 0..(times * 2) {
        let mut branch =
            BlockChain::new(net.time_service(), new_head_id, storage.clone(), None).unwrap();
        let inner = Inner::new(
            net,
            storage.clone(),
            new_head_id,
            txpool.clone(),
            None,
            miner_account.clone(),
            None,
            None,
        )
        .unwrap();
        let block_template = inner.create_block_template().unwrap().template;
        let new_block = branch
            .consensus()
            .create_block(block_template, node_config.net().time_service().as_ref())
            .unwrap();
        new_head_id = new_block.id();
        let executed_block = branch.apply(new_block.clone()).unwrap();

        if i > times {
            main_inner.update_chain(executed_block).unwrap();
            assert_eq!(main_inner.chain.current_header().number(), i + 1);
        }
    }
}

#[stest::test(timeout = 480)]
async fn test_create_block_template_actor() {
    let node_config = Arc::new(NodeConfig::random_for_test());
    let registry = RegistryService::launch();
    registry.put_shared(node_config.clone()).await.unwrap();

    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())
        .expect("init storage by genesis fail.");
    let genesis_id = genesis.block().id();
    let chain_header = storage
        .get_block_header_by_hash(genesis_id)
        .unwrap()
        .unwrap();

    //TODO mock txpool.
    let txpool = TxPoolService::new(node_config.clone(), storage.clone(), chain_header, None);
    registry.put_shared(txpool).await.unwrap();

    registry.put_shared(storage).await.unwrap();
    registry
        .register_mocker(AccountService::mock().unwrap())
        .await
        .unwrap();

    let create_block_template_service = registry.register::<BlockBuilderService>().await.unwrap();
    let response = create_block_template_service
        .send(BlockTemplateRequest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.template.number, 1);
}

#[stest::test]
fn test_create_block_template_by_adjust_time() -> Result<()> {
    let node_config = Arc::new(NodeConfig::random_for_test());

    let (storage, _, genesis) = StarcoinGenesis::init_storage_for_test(node_config.net())?;
    let mut inner = Inner::new(
        node_config.net(),
        storage,
        genesis.block().id(),
        EmptyProvider,
        None,
        AccountInfo::random(),
        None,
        None,
    )?;
    let template = inner.create_block_template()?.template;
    let previous_block_time = template.timestamp;
    let block = node_config
        .net()
        .genesis_config()
        .consensus()
        .create_block(template, node_config.net().time_service().as_ref())?;
    inner.chain.apply(block)?;
    // adjust time to previous than parent.
    let time_service = node_config.net().time_service();
    let mock_time_service = time_service
        .as_any()
        .downcast_ref::<MockTimeService>()
        .unwrap();
    mock_time_service.set(previous_block_time - 1);
    // then create block template, create_block_template() should adjust new block's timestamp.
    let template = inner.create_block_template()?.template;
    let block = node_config
        .net()
        .genesis_config()
        .consensus()
        .create_block(template, node_config.net().time_service().as_ref())?;
    assert!(block.header().timestamp() > previous_block_time);
    inner.chain.apply(block)?;
    Ok(())
}
