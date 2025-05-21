use crate::pool::{AccountSeqNumberClient, UnverifiedUserTransaction};
use anyhow::Result;
use parking_lot::RwLock;
use starcoin_executor::VMMetrics;
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_types::multi_transaction::{
    MultiAccountAddress, MultiSignatureCheckedTransaction, MultiSignedUserTransaction,
    MultiTransactionError,
};
use starcoin_types::{
    block::BlockHeader,
    transaction::{CallError, TransactionError},
};
use starcoin_vm2_state_api::AccountStateReader as AccountStateReader2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_storage::Store as Store2;
use starcoin_vm2_vm_types::transaction::{
    CallError as CallError2, TransactionError as TransactionError2,
};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

/// Cache for state nonces.
#[derive(Clone)]
pub struct NonceCache {
    nonces: Arc<RwLock<HashMap<MultiAccountAddress, u64>>>,
    limit: usize,
}

impl NonceCache {
    /// Create new cache with a limit of `limit` entries.
    pub fn new(limit: usize) -> Self {
        NonceCache {
            nonces: Arc::new(RwLock::new(HashMap::with_capacity(limit / 2))),
            limit,
        }
    }

    /// Retrieve a cached nonce for given sender.
    pub fn get(&self, sender: &MultiAccountAddress) -> Option<u64> {
        self.nonces.read().get(sender).cloned()
    }

    /// Clear all entries from the cache.
    pub fn clear(&self) {
        self.nonces.write().clear();
    }
}

impl std::fmt::Debug for NonceCache {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("NonceCache")
            .field("cache", &self.nonces.read().len())
            .field("limit", &self.limit)
            .finish()
    }
}

#[derive(Clone)]
pub struct CachedSeqNumberClient {
    statedb: Arc<ChainStateDB>,
    statedb2: Arc<ChainStateDB2>,
    cache: NonceCache,
}

impl Debug for CachedSeqNumberClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedSequenceNumberClient")
            .field("cache", &self.cache.nonces.read().len())
            .field("limit", &self.cache.limit)
            .finish()
    }
}

impl CachedSeqNumberClient {
    pub fn new(statedb: ChainStateDB, statedb2: ChainStateDB2, cache: NonceCache) -> Self {
        Self {
            statedb: Arc::new(statedb),
            statedb2: Arc::new(statedb2),
            cache,
        }
    }

    fn latest_sequence_number(&self, address: &MultiAccountAddress) -> u64 {
        match address {
            MultiAccountAddress::VM1(address) => {
                let account_state_reader = AccountStateReader::new(self.statedb.as_ref());
                match account_state_reader.get_account_resource(address) {
                    Err(e) => {
                        error!(
                    "Get account {} resource from statedb error: {:?}, return 0 as sequence_number",
                    address, e
                );
                        0
                    }
                    Ok(account_resource) => account_resource
                        .map(|res| res.sequence_number())
                        .unwrap_or_default(),
                }
            }
            MultiAccountAddress::VM2(address) => {
                let account_state_reader2 = AccountStateReader2::new(self.statedb2.as_ref());
                match account_state_reader2.get_account_resource(address) {
                    Err(e) => {
                        error!(
                    "Get account {} resource from statedb2 error: {:?}, return 0 as sequence_number",
                    address, e
                );
                        0
                    }
                    Ok(account_resource) => account_resource.sequence_number(),
                }
            }
        }
    }
}

impl AccountSeqNumberClient for CachedSeqNumberClient {
    fn account_seq_number(&self, address: &MultiAccountAddress) -> u64 {
        if let Some(nonce) = self.cache.get(address) {
            return nonce;
        }
        let mut cache = self.cache.nonces.write();
        let sequence_number = self.latest_sequence_number(address);
        cache.insert(*address, sequence_number);
        if cache.len() < self.cache.limit {
            return sequence_number;
        }

        debug!(target: "txpool", "NonceCache: reached limit");
        trace_time!("nonce_cache: clear");
        let to_remove: Vec<_> = cache.keys().take(self.cache.limit / 2).cloned().collect();
        for x in to_remove {
            cache.remove(&x);
        }

        sequence_number
    }
}

#[derive(Clone)]
pub struct PoolClient {
    best_block_header: BlockHeader,
    nonce_client: CachedSeqNumberClient,
    vm_metrics: Option<VMMetrics>,
}

impl std::fmt::Debug for PoolClient {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PoolClient")
    }
}

impl PoolClient {
    pub fn new(
        best_block_header: BlockHeader,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        cache: NonceCache,
        vm_metrics: Option<VMMetrics>,
    ) -> Self {
        let state = storage.get_vm_multi_state(best_block_header.id()).unwrap();
        let (state_root1, state_root2) = (state.state_root1(), state.state_root2());
        let statedb = ChainStateDB::new(storage.into_super_arc(), Some(state_root1));
        let statedb2 = ChainStateDB2::new(storage2.into_super_arc(), Some(state_root2));
        let nonce_client = CachedSeqNumberClient::new(statedb, statedb2, cache);
        Self {
            best_block_header,
            nonce_client,
            vm_metrics,
        }
    }
}

impl crate::pool::AccountSeqNumberClient for PoolClient {
    fn account_seq_number(&self, address: &MultiAccountAddress) -> u64 {
        self.nonce_client.account_seq_number(address)
    }
}

impl crate::pool::Client for PoolClient {
    fn verify_transaction(
        &self,
        tx: UnverifiedUserTransaction,
    ) -> Result<MultiSignatureCheckedTransaction, MultiTransactionError> {
        let txn = MultiSignedUserTransaction::from(tx);
        let checked_txn = txn.clone().check_signature().map_err(|e| {
            MultiTransactionError::VM1(TransactionError::InvalidSignature(e.to_string()))
        })?;
        match txn {
            MultiSignedUserTransaction::VM1(txn) => {
                match starcoin_executor::validate_transaction(
                    self.nonce_client.statedb.as_ref(),
                    txn,
                    self.vm_metrics.clone(),
                ) {
                    None => Ok(checked_txn),
                    Some(status) => {
                        Err(TransactionError::CallErr(CallError::ExecutionError(status)).into())
                    }
                }
            }
            MultiSignedUserTransaction::VM2(txn) => {
                match starcoin_vm2_executor::validate_transaction(
                    self.nonce_client.statedb2.as_ref(),
                    txn,
                    self.vm_metrics.clone(),
                ) {
                    None => Ok(checked_txn),
                    Some(status) => {
                        Err(TransactionError2::CallErr(CallError2::ExecutionError(status)).into())
                    }
                }
            }
        }
    }
}
