use super::test_helper;
use crate::pool::AccountSeqNumberClient;
use anyhow::Result;
use common_crypto::hash::PlainCryptoHash;
use common_crypto::keygen::KeyGen;
use parking_lot::RwLock;
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_txpool_api::TxPoolAsyncService;
use std::collections::HashMap;
use std::sync::Arc;
use types::account_address::{self, AccountAddress};
use types::{account_config, transaction::authenticator::AuthenticationKey};

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
async fn test_tx_pool() -> Result<()> {
    let pool = test_helper::start_txpool();
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
    let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 10000);
    let txn = txn.as_signed_user_txn()?.clone();
    let txn_hash = txn.crypto_hash();
    let mut result = pool.clone().add_txns(vec![txn]).await?;
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = pool.clone().get_pending_txns(Some(10)).await?;
    assert_eq!(pending_txns.pop().unwrap().crypto_hash(), txn_hash);

    let next_sequence_number = pool
        .clone()
        .next_sequence_number(account_config::association_address())
        .await?;
    assert_eq!(next_sequence_number, Some(2));
    Ok(())
}

#[actix_rt::test]
async fn test_subscribe_txns() {
    let pool = test_helper::start_txpool();
    let _ = pool.subscribe_txns().await.unwrap();
}

#[actix_rt::test]
async fn test_rollback() -> Result<()> {
    let pool = test_helper::start_txpool();
    let txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 10000);
        let txn = txn.as_signed_user_txn()?.clone();
        txn
    };
    let _ = pool.clone().add_txns(vec![txn.clone()]).await?;
    let new_txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 20000);
        let txn = txn.as_signed_user_txn()?.clone();
        txn
    };
    pool.clone()
        .rollback(vec![txn], vec![new_txn.clone()])
        .await?;
    let txns = pool.clone().get_pending_txns(Some(100)).await?;
    assert_eq!(txns.len(), 1);
    let pending = txns.into_iter().next().unwrap();
    assert_eq!(pending.crypto_hash(), new_txn.crypto_hash());
    Ok(())
}
