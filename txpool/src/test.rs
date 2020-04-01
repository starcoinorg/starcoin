use crate::{pool::AccountSeqNumberClient, TxPoolRef};
use common_crypto::hash::{CryptoHash, HashValue};
use forkable_jellyfish_merkle::node_type::Node;
use parking_lot::RwLock;
use starcoin_bus::BusActor;
use starcoin_state_tree::StateNodeStore;
use starcoin_txpool_api::TxPoolAsyncService;
use std::{collections::HashMap, sync::Arc};
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::StarcoinStorage;
use types::{
    account_address::AccountAddress, block::BlockHeader, transaction::SignedUserTransaction,
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
                self.cache.write().insert(address.clone(), 0);
                0
            }
        }
    }
}

#[actix_rt::test]
async fn test_tx_pool() {
    let pool = gen_pool_for_test();
    let txn = SignedUserTransaction::mock();
    let txn_hash = txn.crypto_hash();
    let mut result = pool.clone().add_txns(vec![txn]).await.unwrap();
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = pool.clone().get_pending_txns(Some(10)).await.unwrap();
    assert_eq!(pending_txns.pop().unwrap().crypto_hash(), txn_hash);
}

#[actix_rt::test]
async fn test_subscribe_txns() {
    let pool = gen_pool_for_test();
    let _ = pool.subscribe_txns().await.unwrap();
}

#[actix_rt::test]
async fn test_rollback() {
    let pool = gen_pool_for_test();
    let txn = SignedUserTransaction::mock();
    let _txn_hash = txn.crypto_hash();
    let _result = pool.clone().add_txns(vec![txn.clone()]).await.unwrap();
    let new_txn = SignedUserTransaction::mock();
    pool.clone()
        .rollback(vec![txn], vec![new_txn.clone()])
        .await
        .unwrap();
    let txns = pool.clone().get_pending_txns(Some(100)).await.unwrap();
    assert_eq!(txns.len(), 1);
    let pending = txns.into_iter().next().unwrap();
    assert_eq!(
        CryptoHash::crypto_hash(&pending),
        CryptoHash::crypto_hash(&new_txn)
    );
}

fn gen_pool_for_test() -> TxPoolRef {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Arc::new(StarcoinStorage::new(cache_storage, db_storage).unwrap());
    let _ = storage.put(HashValue::zero(), Node::new_null().into());
    let header = BlockHeader::genesis_block_header(HashValue::random(), HashValue::zero(), vec![]);
    let bus = BusActor::launch();
    let pool = TxPoolRef::start_with_best_block_header(storage, header, bus);
    pool
}
