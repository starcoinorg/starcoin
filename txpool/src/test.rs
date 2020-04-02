use crate::TxPoolRef;
use anyhow::Result;
use common_crypto::ed25519;
use common_crypto::hash::CryptoHash;
use starcoin_bus::BusActor;
use starcoin_config::{NodeConfig, TxPoolConfig};
use starcoin_consensus::argon_consensus::ArgonConsensus;
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_genesis::Genesis;
use starcoin_txpool_api::TxPoolAsyncService;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use types::account_address::AccountAddress;

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
    let pool = gen_pool_for_test();
    let (_private_key, public_key) = ed25519::compat::generate_keypair(None);
    let account_address = AccountAddress::from_public_key(&public_key);
    let auth_prefix = AccountAddress::authentication_key(&public_key)
        .prefix()
        .to_vec();
    let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 10000);
    let txn = txn.as_signed_user_txn()?.clone();
    let txn_hash = txn.crypto_hash();
    let mut result = pool.clone().add_txns(vec![txn]).await?;
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = pool.clone().get_pending_txns(Some(10)).await?;
    assert_eq!(pending_txns.pop().unwrap().crypto_hash(), txn_hash);
    Ok(())
}

#[actix_rt::test]
async fn test_subscribe_txns() {
    let pool = gen_pool_for_test();
    let _ = pool.subscribe_txns().await.unwrap();
}

#[actix_rt::test]
async fn test_rollback() -> Result<()> {
    let pool = gen_pool_for_test();
    let txn = {
        let (_private_key, public_key) = ed25519::compat::generate_keypair(None);
        let account_address = AccountAddress::from_public_key(&public_key);
        let auth_prefix = AccountAddress::authentication_key(&public_key)
            .prefix()
            .to_vec();
        let txn = Executor::build_mint_txn(account_address, auth_prefix, 1, 10000);
        let txn = txn.as_signed_user_txn()?.clone();
        txn
    };
    let _ = pool.clone().add_txns(vec![txn.clone()]).await?;
    let new_txn = {
        let (_private_key, public_key) = ed25519::compat::generate_keypair(None);
        let account_address = AccountAddress::from_public_key(&public_key);
        let auth_prefix = AccountAddress::authentication_key(&public_key)
            .prefix()
            .to_vec();
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
    assert_eq!(
        CryptoHash::crypto_hash(&pending),
        CryptoHash::crypto_hash(&new_txn)
    );
    Ok(())
}

fn gen_pool_for_test() -> TxPoolRef {
    let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = tempfile::tempdir().unwrap();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            cache_storage,
            db_storage,
        ))
        .unwrap(),
    );
    let node_config = NodeConfig::random_for_test();

    let genesis = Genesis::new::<Executor, ArgonConsensus, StarcoinStorage>(
        Arc::new(node_config),
        storage.clone(),
    )
    .expect("init gensis fail");
    let startup_info = genesis.startup_info().clone();
    let bus = BusActor::launch();
    let pool = TxPoolRef::start(
        TxPoolConfig::default(),
        storage.clone(),
        startup_info.head.get_head(),
        bus,
    );

    pool
}
