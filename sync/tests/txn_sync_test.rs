use config::NodeConfig;
use consensus::Consensus;
use crypto::{hash::PlainCryptoHash, keygen::KeyGen};
use futures_timer::Delay;
use starcoin_txpool_api::TxPoolSyncService;
use std::sync::Arc;
use std::time::Duration;
use types::{
    account_address,
    transaction::{authenticator::AuthenticationKey, SignedUserTransaction},
};

#[stest::test]
async fn test_txn_sync_actor() {
    let mut first_config = NodeConfig::random_for_test();
    first_config.miner.enable_miner_client = false;
    let first_network_address = first_config.network.self_address().unwrap();
    let first_config = Arc::new(first_config);
    let txpool_1 = {
        let first_node = starcoin_node::node::start(first_config.clone(), None)
            .await
            .unwrap();
        first_node.txpool
    };

    // add txn to node1
    let user_txn = gen_user_txn(&first_config);
    let import_result = txpool_1
        .get_service()
        .add_txns(vec![user_txn.clone()])
        .pop();
    assert!(import_result.unwrap().is_ok());

    let mut second_config = NodeConfig::random_for_test();
    second_config.network.seeds = vec![first_network_address];
    second_config.miner.enable_miner_client = false;
    let second_config = Arc::new(second_config);
    let txpool_2 = {
        let second_node = starcoin_node::node::start(second_config.clone(), None)
            .await
            .unwrap();
        second_node.txpool
    };
    //wait sync finish.
    Delay::new(Duration::from_secs(2)).await;
    let current_timestamp = second_config.net().consensus().now();
    // check txn
    let mut txns = txpool_2
        .get_service()
        .get_pending_txns(None, Some(current_timestamp));
    assert_eq!(txns.len(), 1);
    let txn = txns.pop().unwrap();
    assert_eq!(user_txn.crypto_hash(), txn.crypto_hash());
}

fn gen_user_txn(config: &NodeConfig) -> SignedUserTransaction {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
    let txn = executor::build_transfer_from_association(
        account_address,
        auth_prefix,
        0,
        10000,
        config.net().consensus().now() + 40000,
        config.net().chain_id(),
    );
    txn.as_signed_user_txn().unwrap().clone()
}
