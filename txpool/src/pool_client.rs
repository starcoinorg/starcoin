use crate::pool::{AccountSeqNumberClient, UnverifiedUserTransaction};
use crate::verifier_pool::VerifierPool;
use anyhow::Result;
use parking_lot::RwLock;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_pipeline_timing::global_collector;
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_storage::Store2;
use starcoin_types::multi_transaction::{
    MultiAccountAddress, MultiSignatureCheckedTransaction, MultiSignedUserTransaction,
    MultiTransactionError,
};
use starcoin_types::transaction::CallError;
use starcoin_types::transaction::TransactionError;
use starcoin_vm2_state_api::AccountStateReader as AccountStateReader2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_types::transaction::{
    CallError as CallError2, TransactionError as TransactionError2,
};
use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Instant};

/// Cache for state nonces.
#[derive(Clone)]
pub struct NonceCache {
    nonces: Arc<RwLock<HashMap<MultiAccountAddress, u64>>>,
    limit: usize,
}

impl NonceCache {
    /// Create new cache with a limit of `limit` entries.
    pub fn new(limit: usize) -> Self {
        Self {
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

    pub fn new_with_state_dbs(
        statedb: Arc<ChainStateDB>,
        statedb2: Arc<ChainStateDB2>,
        cache: NonceCache,
    ) -> Self {
        Self {
            statedb,
            statedb2,
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
    state_root1: HashValue,
    state_root2: HashValue,
    nonce_client: CachedSeqNumberClient,
    vm_metrics: Option<VMMetrics>,
    verifier_pool: Option<Arc<VerifierPool>>,
}

impl std::fmt::Debug for PoolClient {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PoolClient")
    }
}

impl PoolClient {
    pub fn new(
        state_root1: HashValue,
        state_root2: HashValue,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        cache: NonceCache,
        vm_metrics: Option<VMMetrics>,
        verifier_pool: Option<Arc<VerifierPool>>,
    ) -> Self {
        let statedb = ChainStateDB::new(storage.into_super_arc(), Some(state_root1));
        let statedb2 = ChainStateDB2::new(storage2.into_super_arc(), Some(state_root2));
        let nonce_client = CachedSeqNumberClient::new(statedb, statedb2, cache);
        Self {
            state_root1,
            state_root2,
            nonce_client,
            vm_metrics,
            verifier_pool,
        }
    }

    pub fn new_with_state_dbs(
        state_root1: HashValue,
        state_root2: HashValue,
        statedb: Arc<ChainStateDB>,
        statedb2: Arc<ChainStateDB2>,
        cache: NonceCache,
        vm_metrics: Option<VMMetrics>,
    ) -> Self {
        let nonce_client = CachedSeqNumberClient::new_with_state_dbs(statedb, statedb2, cache);
        Self {
            state_root1,
            state_root2,
            nonce_client,
            vm_metrics,
            verifier_pool: None,
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
        let total_start = Instant::now();
        let tx_hash = tx.hash();
        let txn = MultiSignedUserTransaction::from(tx);
        let vm_type = match &txn {
            MultiSignedUserTransaction::VM1(_) => "vm1",
            MultiSignedUserTransaction::VM2(_) => "vm2",
        };
        let sig_start = Instant::now();
        let checked_txn = txn.clone().check_signature().map_err(|e| {
            debug!(
                target: "txpool",
                "verify_transaction failed at signature tx={:?} type={} total_ms={:.3} err={}",
                tx_hash,
                vm_type,
                total_start.elapsed().as_secs_f64() * 1000.0,
                e
            );
            MultiTransactionError::VM1(TransactionError::InvalidSignature(e.to_string()))
        })?;
        let sig_dur = sig_start.elapsed();
        match txn {
            MultiSignedUserTransaction::VM1(txn) => {
                match starcoin_executor::validate_transaction(
                    self.nonce_client.statedb.as_ref(),
                    txn,
                    self.vm_metrics.clone(),
                ) {
                    None => {
                        // Record timing for pipeline analysis
                        global_collector().record_txn_verify(tx_hash, total_start.elapsed().as_secs_f64() * 1000.0);
                        Ok(checked_txn)
                    }
                    Some(status) => {
                        Err(TransactionError::CallErr(CallError::ExecutionError(status)).into())
                    }
                }
            }
            MultiSignedUserTransaction::VM2(txn) => {
                let vm_start = Instant::now();
                let status = if let Some(pool) = &self.verifier_pool {
                    let mut entry = pool.checkout(self.state_root1, self.state_root2);
                    entry.verify_vm2(txn)
                } else {
                    starcoin_vm2_executor::validate_transaction(
                        self.nonce_client.statedb2.as_ref(),
                        txn,
                        self.vm_metrics.clone(),
                    )
                };
                match status {
                    None => {
                        let vm_dur = vm_start.elapsed();
                        if vm_type == "vm2" {
                            debug!(
                                target: "txpool",
                                "verify_transaction tx={:?} type={} sig_ms={:.3} vm_ms={:.3} total_ms={:.3}",
                                tx_hash,
                                vm_type,
                                sig_dur.as_secs_f64() * 1000.0,
                                vm_dur.as_secs_f64() * 1000.0,
                                total_start.elapsed().as_secs_f64() * 1000.0,
                            );
                        }
                        // Record timing for pipeline analysis
                        global_collector().record_txn_verify(tx_hash, total_start.elapsed().as_secs_f64() * 1000.0);
                        Ok(checked_txn)
                    }
                    Some(status) => {
                        let vm_dur = vm_start.elapsed();
                        if vm_type == "vm2" {
                            debug!(
                                target: "txpool",
                                "verify_transaction failed tx={:?} type={} sig_ms={:.3} vm_ms={:.3} total_ms={:.3} status={:?}",
                                tx_hash,
                                vm_type,
                                sig_dur.as_secs_f64() * 1000.0,
                                vm_dur.as_secs_f64() * 1000.0,
                                total_start.elapsed().as_secs_f64() * 1000.0,
                                status,
                            );
                        }
                        Err(TransactionError2::CallErr(CallError2::ExecutionError(status)).into())
                    }
                }
            }
        }
    }
}
