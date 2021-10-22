// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::{PubSubImpl, PubSubService, PubSubServiceFactory};
use anyhow::Result;
use futures::StreamExt;
use jsonrpc_core::{futures, MetaIoHandler};
use jsonrpc_pubsub::Session;
use serde_json::Value;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_chain_notify::ChainNotifyHandlerService;
use starcoin_consensus::Consensus;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, Genesis, HashValue, PrivateKey};
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::metadata::Metadata;
use starcoin_rpc_api::pubsub::StarcoinPubSub;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::RegistryAsyncService;
use starcoin_state_api::StateReaderExt;
use starcoin_storage::BlockStore;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::{account_address, U256};
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::sync::Arc;
use tokio::time::timeout;
use tokio::time::Duration;

#[actix_rt::test]
pub async fn test_subscribe_to_events() -> Result<()> {
    starcoin_logger::init_for_test();
    // prepare

    let (_txpool_service, storage, config, _, registry) =
        test_helper::start_txpool_with_miner(1000, true).await;
    let startup_info = storage.get_startup_info()?.unwrap();
    let net = config.net();
    let mut block_chain = BlockChain::new(net.time_service(), startup_info.main, storage, None)?;
    let miner_account = AccountInfo::random();

    let pri_key = Ed25519PrivateKey::genesis();
    let public_key = pri_key.public_key();
    let account_address = account_address::from_public_key(&public_key);
    let txn = {
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            0,
            10000,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let (block_template, _) = block_chain.create_block_template(
        *miner_account.address(),
        None,
        vec![txn.clone()],
        vec![],
        None,
    )?;
    debug!("block_template: gas_used: {}", block_template.gas_used);
    let new_block = block_chain
        .consensus()
        .create_block(block_template, net.time_service().as_ref())?;
    let executed_block = block_chain.apply(new_block.clone())?;

    let reader = block_chain.chain_state_reader();
    let balance = reader.get_balance(account_address)?;
    assert_eq!(balance, Some(10000));

    // now block is applied, we can emit events.

    let bus = registry.service_ref::<BusService>().await?;
    let _notify_service = registry
        .register::<ChainNotifyHandlerService>()
        .await
        .unwrap();

    let service = registry
        .register_by_factory::<PubSubService, PubSubServiceFactory>()
        .await?;

    let pubsub = PubSubImpl::new(service);
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Subscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": [{"type_name":"events"}, {}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;
    let resp = io.handle_request(request, metadata.clone()).await;
    assert_eq!(resp.unwrap(), response.to_owned());

    // Subscribe error
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": [{"type_name":"events"}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: events","data":"\"Expected a filter object.\""},"id":1}"#;

    let resp = io.handle_request(request, metadata.clone()).await;
    assert_eq!(response.to_owned(), resp.unwrap());

    // send block
    let block_detail = Arc::new(executed_block);
    bus.broadcast(NewHeadBlock(block_detail))?;

    let mut receiver = receiver;

    let res = timeout(Duration::from_secs(5), receiver.next()).await?;
    assert!(res.is_some());

    let res = res.unwrap();
    let notification = serde_json::from_str::<jsonrpc_core::Notification>(res.as_str()).unwrap();
    match notification.params {
        jsonrpc_core::Params::Map(s) => {
            let v = s.get("result").unwrap().get("block_number").unwrap();
            assert_eq!(v.as_str(), Some("1"));
        }
        p => {
            panic!("subscribe return unexpected result, {:?}", &p);
        }
    }
    Ok(())
}

#[stest::test]
pub async fn test_subscribe_to_pending_transactions() -> Result<()> {
    // given
    let (txpool_service, _, config, _, registry) =
        test_helper::start_txpool_with_miner(1000, true).await;
    let service = registry
        .register_by_factory::<PubSubService, PubSubServiceFactory>()
        .await?;
    let pubsub = PubSubImpl::new(service);
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, receiver) = futures::channel::mpsc::unbounded();
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Fail if params are provided
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": [{"type_name":"newPendingTransactions"}, {}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: newPendingTransactions","data":"\"Expected no parameters.\""},"id":1}"#;
    let resp = io.handle_request(request, metadata.clone()).await;
    assert_eq!(resp, Some(response.to_owned()));

    // Subscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": [{"type_name":"newPendingTransactions"}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;
    let resp = io.handle_request(request, metadata.clone()).await;
    assert_eq!(resp, Some(response.to_owned()));

    // Send new transactions
    let txn = {
        let account = AccountInfo::random();
        let txn = starcoin_executor::build_transfer_from_association(
            account.address,
            0,
            10000,
            DEFAULT_EXPIRATION_TIME,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let txn_id = txn.id();
    txpool_service.add_txns(vec![txn]).pop().unwrap().unwrap();
    let mut receiver = receiver;
    let res = receiver.next().await.unwrap();
    let prefix = r#"{"jsonrpc":"2.0","method":"starcoin_subscription","params":{"subscription":0,"result":[""#;
    let suffix = r#""]}}"#;
    let response = format!("{}0x{}{}", prefix, txn_id.to_hex(), suffix);
    assert_eq!(res, response);
    // And unsubscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_unsubscribe", "params": [0], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
    let resp = io.handle_request(request, metadata).await;
    assert_eq!(resp, Some(response.to_owned()));

    let res = timeout(Duration::from_secs(1), receiver.next()).await?;
    assert_eq!(res, None);
    Ok(())
}

#[stest::test]
pub async fn test_subscribe_to_mint_block() -> Result<()> {
    let (_txpool_service, .., registry) = test_helper::start_txpool_with_miner(1000, true).await;
    let bus = registry.service_ref::<BusService>().await?;
    let service = registry
        .register_by_factory::<PubSubService, PubSubServiceFactory>()
        .await?;
    let pubsub = PubSubImpl::new(service);
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, mut receiver) = futures::channel::mpsc::unbounded();
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Subscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": [{"type_name":"newMintBlock"}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;
    let resp = io.handle_request(request, metadata.clone()).await;
    assert_eq!(resp, Some(response.to_owned()));
    // Generate a event
    let diff = U256::from(1024);
    let header_hash = vec![0u8; 76];
    let mint_block_event = MintBlockEvent::new(
        HashValue::random(),
        ConsensusStrategy::Dummy,
        header_hash.clone(),
        diff,
        0,
        None,
    );
    bus.broadcast(mint_block_event.clone()).unwrap();
    let res = timeout(Duration::from_secs(1), receiver.next())
        .await?
        .ok_or_else(|| anyhow::anyhow!("Empty value"))?;
    let r: Value = serde_json::from_str(&res).unwrap();
    let v = r["params"]["result"].clone();
    let mint_block: MintBlockEvent = serde_json::from_value(v).unwrap();
    assert_eq!(mint_block.difficulty, diff);
    assert_eq!(&mint_block.minting_blob, &header_hash);
    // Unsubscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_unsubscribe", "params": [0], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
    let resp = io.handle_request(request, metadata).await;
    assert_eq!(resp, Some(response.to_owned()));
    Ok(())
}
