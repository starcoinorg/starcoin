// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::pool::AccountSeqNumberClient;
use crate::TxStatus;
use anyhow::Result;
use crypto::keygen::KeyGen;
use network_api::messages::{PeerTransactionsMessage, TransactionsMessage};
use network_api::PeerId;
use parking_lot::RwLock;
use starcoin_config::{MetricsConfig, NodeConfig};
use starcoin_executor::{
    create_signed_txn_with_association_account, encode_transfer_script_function,
    DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::{TxPoolSyncService, TxnStatusFullEvent};
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use stest::actix_export::time::delay_for;
use storage::BlockStore;
use types::{
    account_address::{self, AccountAddress},
    account_config,
    transaction::{SignedUserTransaction, Transaction, TransactionPayload},
    U256,
};

#[derive(Clone, Debug)]
struct MockNonceClient {
    cache: Arc<RwLock<HashMap<AccountAddress, u64>>>,
}

impl Default for MockNonceClient {
    fn default() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AccountSeqNumberClient for MockNonceClient {
    fn account_seq_number(&self, address: &AccountAddress) -> u64 {
        let cached = self.cache.read().get(address).cloned();
        match cached {
            Some(v) => v,
            None => {
                self.cache.write().insert(*address, 0);
                0
            }
        }
    }
}

#[stest::test]
async fn test_txn_expire() -> Result<()> {
    let (txpool_service, _storage, config, _, _) = test_helper::start_txpool().await;
    let txn = generate_txn(config, 0);
    txpool_service.add_txns(vec![txn]).pop().unwrap()?;
    let pendings = txpool_service.get_pending_txns(None, Some(0));
    assert_eq!(pendings.len(), 1);

    let pendings = txpool_service.get_pending_txns(None, Some(2));
    assert_eq!(pendings.len(), 0);

    Ok(())
}

#[stest::test]
async fn test_tx_pool() -> Result<()> {
    let (txpool_service, _storage, config, _, _) = test_helper::start_txpool().await;
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let txn = starcoin_executor::build_transfer_from_association(
        account_address,
        0,
        10000,
        1,
        config.net(),
    );
    let txn = txn.as_signed_user_txn()?.clone();
    let txn_hash = txn.id();
    let mut result = txpool_service.add_txns(vec![txn]);
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = txpool_service.get_pending_txns(Some(10), Some(0));
    assert_eq!(pending_txns.pop().unwrap().id(), txn_hash);

    let next_sequence_number =
        txpool_service.next_sequence_number(account_config::association_address());
    assert_eq!(next_sequence_number, Some(1));
    Ok(())
}

#[stest::test]
async fn test_subscribe_txns() {
    let (pool, ..) = test_helper::start_txpool().await;
    let _ = pool.subscribe_txns();
}

#[stest::test(timeout = 200)]
async fn test_pool_pending() -> Result<()> {
    let pool_size = 5;
    let expect_reject = 3;
    let (txpool_service, _storage, node_config, _, _) =
        test_helper::start_txpool_with_size(pool_size).await;
    let metrics_config: &MetricsConfig = &node_config.metrics;

    let txn_vec = (0..pool_size + expect_reject)
        .into_iter()
        .map(|index| generate_txn(node_config.clone(), index))
        .collect::<Vec<_>>();

    let _ = txpool_service.add_txns(txn_vec.clone());
    delay_for(Duration::from_millis(200)).await;

    let txn_count_metric = metrics_config
        .get_metric("txpool_status", Some(("name", "count")))
        .unwrap()
        .pop()
        .unwrap();
    let txn_count_metric_value = txn_count_metric.get_gauge().get_value();
    assert_eq!(
        pool_size, txn_count_metric_value as u64,
        "expect {} txn in pool, but got: {}",
        pool_size, txn_count_metric_value
    );

    let txn_added_event_metric = metrics_config
        .get_metric("txpool_txn_event_total", Some(("type", "added")))
        .unwrap()
        .pop()
        .unwrap();
    let txn_added_event_metric_value = txn_added_event_metric.get_counter().get_value();
    assert_eq!(
        pool_size, txn_added_event_metric_value as u64,
        "expect {} added events, but got: {}",
        pool_size, txn_added_event_metric_value
    );

    let txn_rejected_event_metric = metrics_config
        .get_metric("txpool_txn_event_total", Some(("type", "rejected")))
        .unwrap()
        .pop()
        .unwrap();
    let txn_rejected_event_metric_value = txn_rejected_event_metric.get_counter().get_value();
    assert_eq!(
        expect_reject, txn_rejected_event_metric_value as u64,
        "expect {} rejected events, but got: {}",
        expect_reject, txn_rejected_event_metric_value
    );

    let txn_vec = (pool_size..(pool_size + expect_reject))
        .into_iter()
        .map(|index| generate_txn(node_config.clone(), index))
        .collect::<Vec<_>>();

    let _ = txpool_service.add_txns(txn_vec.clone());
    let pending = txpool_service.get_pending_txns(Some(pool_size), None);
    assert!(!pending.is_empty());

    delay_for(Duration::from_millis(200)).await;

    let txn_rejected_event_metric = metrics_config
        .get_metric("txpool_txn_event_total", Some(("type", "rejected")))
        .unwrap()
        .pop()
        .unwrap();
    let txn_rejected_event_metric_value = txn_rejected_event_metric.get_counter().get_value();
    assert_eq!(
        expect_reject * 2,
        txn_rejected_event_metric_value as u64,
        "expect {} rejected events, but got: {}",
        expect_reject * 2,
        txn_rejected_event_metric_value
    );

    Ok(())
}

#[stest::test]
async fn test_rollback() -> Result<()> {
    let (pool, storage, config, _, _) = test_helper::start_txpool().await;
    let start_timestamp = 0;
    let retracted_txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            0,
            10000,
            start_timestamp + DEFAULT_EXPIRATION_TIME,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };
    let _ = pool.add_txns(vec![retracted_txn.clone()]);

    let enacted_txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            0,
            20000,
            start_timestamp + DEFAULT_EXPIRATION_TIME,
            config.net(),
        );
        txn.as_signed_user_txn()?.clone()
    };

    let pack_txn_to_block = |txn: SignedUserTransaction| {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let storage = storage.clone();
        let main = storage.get_startup_info()?.unwrap().main;
        let block_header = storage.get_block_header_by_hash(main)?.unwrap();

        let mut open_block = OpenedBlock::new(
            storage,
            block_header,
            u64::MAX,
            account_address,
            (start_timestamp + 60 * 10) * 1000,
            vec![],
            U256::from(1024u64),
            config.net().genesis_config().consensus(),
            None,
        )?;
        let excluded_txns = open_block.push_txns(vec![txn])?;
        assert_eq!(excluded_txns.discarded_txns.len(), 0);
        assert_eq!(excluded_txns.untouched_txns.len(), 0);

        let block_template = open_block.finalize()?;
        let block = block_template.into_block(0, types::block::BlockHeaderExtra::new([0u8; 4]));
        Ok::<_, anyhow::Error>(block)
    };

    let retracted_block = pack_txn_to_block(retracted_txn)?;
    let enacted_block = pack_txn_to_block(enacted_txn)?;

    // flush the state, to make txpool happy
    {
        let main = storage.get_startup_info()?.unwrap().main;
        let block_header = storage.get_block_header_by_hash(main)?.unwrap();
        let chain_state = ChainStateDB::new(storage.clone(), Some(block_header.state_root()));
        let mut txns: Vec<_> = enacted_block
            .transactions()
            .iter()
            .map(|t| Transaction::UserTransaction(t.clone()))
            .collect();
        let parent_block_header = storage
            .get_block_header_by_hash(enacted_block.header().parent_hash())
            .unwrap()
            .unwrap();
        txns.insert(
            0,
            Transaction::BlockMetadata(enacted_block.to_metadata(parent_block_header.gas_used())),
        );
        let root = starcoin_executor::block_execute(&chain_state, txns, u64::MAX, None)?.state_root;

        assert_eq!(root, enacted_block.header().state_root());
        chain_state.flush()?;
    }
    pool.chain_new_block(vec![enacted_block], vec![retracted_block])
        .unwrap();
    let txns = pool.get_pending_txns(Some(100), Some(start_timestamp + 60 * 10));
    assert_eq!(txns.len(), 0);
    Ok(())
}

#[stest::test(timeout = 480)]
async fn test_txpool_actor_service() {
    let (_txpool_service, _storage, config, tx_pool_actor, _registry) =
        test_helper::start_txpool().await;
    let txn = generate_txn(config, 0);

    tx_pool_actor
        .notify(PeerTransactionsMessage::new(
            PeerId::random(),
            TransactionsMessage::new(vec![txn.clone()]),
        ))
        .unwrap();

    delay_for(Duration::from_millis(200)).await;
    tx_pool_actor
        .notify(Into::<TxnStatusFullEvent>::into(vec![(
            txn.id(),
            TxStatus::Added,
        )]))
        .unwrap();

    delay_for(Duration::from_millis(300)).await;
}

fn generate_txn(config: Arc<NodeConfig>, seq: u64) -> SignedUserTransaction {
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let txn = create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(encode_transfer_script_function(account_address, 10000)),
        seq,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        2,
        config.net(),
    );
    txn
}
