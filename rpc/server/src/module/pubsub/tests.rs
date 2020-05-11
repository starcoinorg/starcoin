use crate::metadata::Metadata;
use crate::module::PubSubImpl;
use anyhow::Result;
use jsonrpc_core::futures as futures01;
use jsonrpc_core::MetaIoHandler;
use jsonrpc_pubsub::Session;
// use starcoin_crypto::hash::HashValue;
use futures::compat::Stream01CompatExt;
use futures::StreamExt;
use starcoin_rpc_api::pubsub::StarcoinPubSub;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_types::account_address::AccountAddress;
use std::sync::Arc;
use txpool::test_helper::start_txpool;
// use txpool::TxPoolRef;
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::{Genesis, PrivateKey};
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;

#[stest::test]
pub async fn test_subscribe_to_pending_transactions() -> Result<()> {
    // given
    let txpool = start_txpool();
    let pubsub = PubSubImpl::new();
    pubsub.start_transaction_subscription_handler(txpool.clone());
    let pubsub = pubsub.to_delegate();

    let mut io = MetaIoHandler::default();
    io.extend_with(pubsub);

    let mut metadata = Metadata::default();
    let (sender, receiver) = futures01::sync::mpsc::channel(8);
    metadata.session = Some(Arc::new(Session::new(sender)));

    // Fail if params are provided
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_subscribe", "params": ["newPendingTransactions", {}], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: Invalid Pub-Sub parameters."},"id":1}"#;
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
        let pri_key = Ed25519PrivateKey::genesis();
        let public_key = pri_key.public_key();
        let account_address = AccountAddress::from_public_key(&public_key);
        let auth_prefix = AccountAddress::authentication_key(&public_key)
            .prefix()
            .to_vec();
        let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 10000);
        let txn = txn.as_signed_user_txn()?.clone();
        txn
    };
    txpool.clone().add_txns(vec![txn]).await?;
    let mut receiver = receiver.compat();
    let res = receiver.next().await.transpose().unwrap();
    let response = r#"{"jsonrpc":"2.0","method":"starcoin_subscription","params":{"result":["c224e1ab9542528a15cdb39d9aa09ff21999330f92d387dca92a6748cf1827cb"],"subscription":0}}"#;
    assert_eq!(res, Some(response.into()));
    // And unsubscribe
    let request = r#"{"jsonrpc": "2.0", "method": "starcoin_unsubscribe", "params": [0], "id": 1}"#;
    let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
    assert_eq!(
        io.handle_request_sync(request, metadata),
        Some(response.to_owned())
    );

    let res = receiver.next().await.transpose().unwrap();
    assert_eq!(res, None);
    Ok(())
}
