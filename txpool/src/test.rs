use super::TxPool;
use crate::pool::AccountSeqNumberClient;
use common_crypto::hash::CryptoHash;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use traits::TxPoolAsyncService;
use types::{account_address::AccountAddress, transaction::SignedUserTransaction};

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
    let pool = TxPool::start(MockNonceClient::default());

    let txn = SignedUserTransaction::mock();
    let txn_hash = txn.crypto_hash();
    let mut result = pool.import_txns(vec![txn]).await.unwrap();
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = pool.get_pending_txns(Some(10)).await.unwrap();
    assert_eq!(pending_txns.pop().unwrap().crypto_hash(), txn_hash);
}

#[actix_rt::test]
async fn test_subscribe_txns() {
    let pool = TxPool::start(MockNonceClient::default());
    let _ = pool.subscribe_txns().await.unwrap();
}

#[actix_rt::test]
async fn test_rollback() {
    let pool = TxPool::start(MockNonceClient::default());
    let txn = SignedUserTransaction::mock();
    let txn_hash = txn.crypto_hash();
    let mut result = pool.import_txns(vec![txn.clone()]).await.unwrap();
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
