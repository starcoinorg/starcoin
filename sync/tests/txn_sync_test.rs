use config::NodeConfig;
use executor::DEFAULT_EXPIRATION_TIME;
use starcoin_crypto::keygen::KeyGen;
use starcoin_service_registry::RegistryAsyncService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{account_address, transaction::SignedUserTransaction};
use std::sync::Arc;
use std::time::Duration;
use test_helper::run_node_by_config;
use txpool::TxPoolService;

//TODO
#[ignore]
#[stest::test]
fn test_txn_sync_actor() {
    let mut first_config = NodeConfig::random_for_test();
    first_config.miner.disable_miner_client = Some(false);
    let first_network_address = first_config.network.self_address();
    let first_config = Arc::new(first_config);
    let first_node = run_node_by_config(first_config.clone()).unwrap();
    let txpool_1 = first_node
        .registry()
        .get_shared_sync::<TxPoolService>()
        .unwrap();

    // add txn to node1
    let user_txn = gen_user_txn(&first_config);
    let import_result = txpool_1.add_txns(vec![user_txn.clone()]).pop();
    assert!(import_result.unwrap().is_ok());

    let mut second_config = NodeConfig::random_for_test();
    second_config.network.seeds = vec![first_network_address].into();
    second_config.miner.disable_miner_client = Some(false);
    let second_config = Arc::new(second_config);

    let second_node = run_node_by_config(second_config.clone()).unwrap();
    let txpool_2 = second_node
        .registry()
        .get_shared_sync::<TxPoolService>()
        .unwrap();
    //wait sync finish.
    //Delay::new(Duration::from_secs(2)).await;
    std::thread::sleep(Duration::from_secs(2));
    let current_timestamp = second_config.net().time_service().now_secs();
    // check txn
    let mut txns = txpool_2.get_pending_txns(None, Some(current_timestamp));
    assert_eq!(txns.len(), 1);
    let txn = txns.pop().unwrap();
    assert_eq!(user_txn.id(), txn.id());
    second_node.stop().unwrap();
    first_node.stop().unwrap();
}

fn gen_user_txn(config: &NodeConfig) -> SignedUserTransaction {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let txn = executor::build_transfer_from_association(
        account_address,
        0,
        10000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    );
    txn.as_signed_user_txn().unwrap().clone()
}
