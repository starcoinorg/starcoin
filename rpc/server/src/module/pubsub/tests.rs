use crate::{
    metadata::Metadata,
    module::{PubSubImpl, PubSubService},
};
use anyhow::Result;
use jsonrpc_core::{futures as futures01, MetaIoHandler};
use jsonrpc_pubsub::Session;
// use starcoin_crypto::hash::HashValue;
use crate::module::test_helper;
use futures::{compat::Stream01CompatExt, StreamExt};
use starcoin_bus::{Bus, BusActor};
use starcoin_config::NodeConfig;
use starcoin_consensus::dev::DevConsensus;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, hash::PlainCryptoHash, Genesis, PrivateKey};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::pubsub::StarcoinPubSub;
use starcoin_state_api::AccountStateReader;
use starcoin_traits::{ChainReader, ChainWriter, Consensus};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address;
use starcoin_types::{
    block::BlockDetail, system_events::NewHeadBlock, transaction::authenticator::AuthenticationKey,
};
use starcoin_wallet_api::WalletAccount;
use std::sync::Arc;
use tokio::time::timeout;

use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_types::chain_config::ChainId;
use starcoin_vm_types::transaction::helpers::get_current_timestamp;
use tokio::time::Duration;

#[actix_rt::test]
pub async fn test_subscribe_to_events() -> Result<()> {
    starcoin_logger::init_for_test();
    // prepare
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test::<DevConsensus>(config.clone())?;
    let miner_account = WalletAccount::random();

    let pri_key = Ed25519PrivateKey::genesis();
    let public_key = pri_key.public_key();
    let account_address = account_address::from_public_key(&public_key);
    let txn = {
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            auth_prefix,
            0,
            10000,
            get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
            ChainId::test(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let (block_template, _) = block_chain.create_block_template(
        *miner_account.address(),
        Some(miner_account.get_auth_key().prefix().to_vec()),
        None,
        vec![txn.clone()],
        vec![],
    )?;
    debug!(
        "block_template: gas_used: {}, gas_limit: {}",
        block_template.gas_used, block_template.gas_limit
    );
    let new_block = DevConsensus::create_block(&block_chain, block_template)?;
    block_chain.apply(new_block.clone())?;

    let reader = AccountStateReader::new(block_chain.chain_state_reader());
    let balance = reader.get_balance(&account_address)?;
    assert_eq!(balance, Some(10000));

    // now block is applied, we can emit events.

    let service = PubSubService::new();
    let bus = BusActor::launch();
    service.start_chain_notify_handler(bus.clone(), block_chain.get_storage());
    let pubsub = PubSubImpl::new(service);
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, receiver) = futures01::sync::mpsc::channel(128);
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Subscribe
    let request =
        r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": ["events", {}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata.clone()),
        Some(response.to_owned())
    );

    // Subscribe error
    let request =
        r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": ["events"], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: events","data":"\"Expected a filter object.\""},"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata.clone()),
        Some(response.to_owned())
    );

    // send block
    let block_detail = Arc::new(BlockDetail::new(new_block, 0.into()));
    bus.broadcast(NewHeadBlock(block_detail)).await?;

    let mut receiver = receiver.compat();

    let res = timeout(Duration::from_secs(5), receiver.next())
        .await?
        .transpose()
        .unwrap();
    assert!(res.is_some());

    let res = res.unwrap();
    let notification = serde_json::from_str::<jsonrpc_core::Notification>(res.as_str()).unwrap();
    match notification.params {
        jsonrpc_core::Params::Map(s) => {
            let v = s.get("result").unwrap().get("blockNumber").unwrap();
            assert_eq!(v.as_u64(), Some(1));
        }
        p => {
            assert!(false, "subscribe return unexpected result, {:?}", &p);
        }
    }
    Ok(())
}

#[stest::test]
pub async fn test_subscribe_to_pending_transactions() -> Result<()> {
    // given
    let (txpool, _) = test_helper::start_txpool();
    let txpool_service = txpool.get_service();
    let service = PubSubService::new();
    let txn_receiver = txpool_service.subscribe_txns();
    service.start_transaction_subscription_handler(txn_receiver);
    let pubsub = PubSubImpl::new(service);
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, receiver) = futures01::sync::mpsc::channel(8);
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Fail if params are provided
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": ["newPendingTransactions", {}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: newPendingTransactions","data":"\"Expected no parameters.\""},"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata.clone()),
        Some(response.to_owned())
    );

    // Subscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": ["newPendingTransactions"], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":0,"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata.clone()),
        Some(response.to_owned())
    );
    // Send new transactions
    let txn = {
        let auth_key = AuthenticationKey::random();
        let account_address = auth_key.derived_address();
        let auth_prefix = auth_key.prefix().to_vec();
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            auth_prefix,
            0,
            10000,
            DEFAULT_EXPIRATION_TIME,
            ChainId::test(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let txn_id = txn.crypto_hash();
    txpool_service.add_txns(vec![txn]).pop().unwrap().unwrap();
    let mut receiver = receiver.compat();
    let res = receiver.next().await.transpose().unwrap();
    let prefix = r#"{"jsonrpc":"2.0","method":"starcoin_subscription","params":{"result":[""#;
    let suffix = r#""],"subscription":0}}"#;
    let response = format!("{}{}{}", prefix, txn_id.to_hex(), suffix);
    assert_eq!(res, Some(response));
    // And unsubscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_unsubscribe", "params": [0], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata),
        Some(response.to_owned())
    );

    let res = timeout(Duration::from_secs(5), receiver.next())
        .await?
        .transpose()
        .unwrap();

    assert_eq!(res, None);
    Ok(())
}
