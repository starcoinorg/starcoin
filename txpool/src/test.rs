use super::test_helper;
use crate::pool::AccountSeqNumberClient;
use anyhow::Result;
use common_crypto::{hash::PlainCryptoHash, keygen::KeyGen};
use parking_lot::RwLock;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::TxPoolSyncService;
use std::{collections::HashMap, sync::Arc};
use storage::BlockStore;
use types::{
    account_address::{self, AccountAddress},
    account_config,
    transaction::{authenticator::AuthenticationKey, SignedUserTransaction, Transaction},
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
async fn test_tx_pool() -> Result<()> {
    let (pool, _storage) = test_helper::start_txpool();
    let txpool_service = pool.get_service();
    let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
    let account_address = account_address::from_public_key(&public_key);
    let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
    let txn =
        starcoin_executor::build_transfer_from_association(account_address, auth_prefix, 0, 10000);
    let txn = txn.as_signed_user_txn()?.clone();
    let txn_hash = txn.crypto_hash();
    let mut result = txpool_service.add_txns(vec![txn]);
    assert!(result.pop().unwrap().is_ok());
    let mut pending_txns = txpool_service.get_pending_txns(Some(10));
    assert_eq!(pending_txns.pop().unwrap().crypto_hash(), txn_hash);

    let next_sequence_number =
        txpool_service.next_sequence_number(account_config::association_address());
    assert_eq!(next_sequence_number, Some(1));
    Ok(())
}

#[stest::test]
async fn test_subscribe_txns() {
    let (pool, _storage) = test_helper::start_txpool();
    let _ = pool.get_service().subscribe_txns();
}

// TODO: ignore test for now. will fire another PR to make this happen
#[stest::test]
async fn test_rollback() -> Result<()> {
    let (pool, storage) = test_helper::start_txpool();
    let retracted_txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            auth_prefix,
            0,
            10000,
        );
        txn.as_signed_user_txn()?.clone()
    };
    let _ = pool.get_service().add_txns(vec![retracted_txn.clone()]);

    let enacted_txn = {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let txn = starcoin_executor::build_transfer_from_association(
            account_address,
            auth_prefix,
            0,
            20000,
        );
        txn.as_signed_user_txn()?.clone()
    };

    let pack_txn_to_block = |txn: SignedUserTransaction| {
        let (_private_key, public_key) = KeyGen::from_os_rng().generate_keypair();
        let account_address = account_address::from_public_key(&public_key);
        let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
        let storage = storage.clone();
        let master = storage.get_startup_info()?.unwrap().master;
        let block_header = storage.get_block_header_by_hash(master)?.unwrap();

        let mut open_block = OpenedBlock::new(
            storage,
            block_header,
            u64::MAX,
            account_address,
            Some(auth_prefix),
            vec![],
        )?;
        let excluded_txns = open_block.push_txns(vec![txn])?;
        assert_eq!(excluded_txns.discarded_txns.len(), 0);
        assert_eq!(excluded_txns.untouched_txns.len(), 0);

        let block_template = open_block.finalize()?;
        let block = block_template.into_block(vec![], U256::from(1024u64));
        Ok::<_, anyhow::Error>(block)
    };

    let retracted_block = pack_txn_to_block(retracted_txn)?;
    let enacted_block = pack_txn_to_block(enacted_txn)?;

    // flush the state, to make txpool happy
    {
        let master = storage.get_startup_info()?.unwrap().master;
        let block_header = storage.get_block_header_by_hash(master)?.unwrap();
        let chain_state = ChainStateDB::new(storage.clone(), Some(block_header.state_root()));
        let mut txns: Vec<_> = enacted_block
            .transactions()
            .iter()
            .map(|t| Transaction::UserTransaction(t.clone()))
            .collect();
        txns.insert(
            0,
            Transaction::BlockMetadata(enacted_block.header().clone().into_metadata()),
        );
        let root = starcoin_executor::block_execute(
            &chain_state,
            txns,
            enacted_block.header().gas_limit(),
        )?
        .state_root;

        assert_eq!(root, enacted_block.header().state_root());
        chain_state.flush()?;
    }
    pool.get_service()
        .rollback(vec![enacted_block], vec![retracted_block])
        .unwrap();
    let txns = pool.get_service().get_pending_txns(Some(100));
    assert_eq!(txns.len(), 0);
    Ok(())
}
