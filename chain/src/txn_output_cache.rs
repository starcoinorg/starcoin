// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_executor::BlockExecutedData as BlockExecutedData1;
use starcoin_statedb::ChainStateDB;
use starcoin_vm2_executor::block_executor::BlockExecutedData as BlockExecutedData2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_types::transaction::TransactionOutput as TransactionOutput2;
use starcoin_vm_types::transaction::TransactionOutput as TransactionOutput1;
use std::sync::Arc;

/// Cached block outputs containing VM1 and VM2 transaction outputs
#[derive(Debug, Clone)]
pub struct CachedBlockOutputs {
    pub vm1_outputs: Option<Arc<Vec<TransactionOutput1>>>,
    pub vm2_outputs: Option<Arc<Vec<TransactionOutput2>>>,
}

impl CachedBlockOutputs {
    pub fn new(
        vm1_outputs: Option<Vec<TransactionOutput1>>,
        vm2_outputs: Option<Vec<TransactionOutput2>>,
    ) -> Self {
        Self {
            vm1_outputs: vm1_outputs.map(Arc::new),
            vm2_outputs: vm2_outputs.map(Arc::new),
        }
    }
}

/// Cached block state including StateDB and execution results.
/// This allows block execution to skip re-execution and apply_write_set entirely.
#[derive(Clone)]
pub struct CachedBlockState {
    /// Cached StateDB for VM1 (already applied write_sets and committed, not flushed)
    pub statedb: Option<Arc<ChainStateDB>>,
    /// Cached StateDB for VM2 (already applied write_sets and committed, not flushed)
    pub statedb2: Option<Arc<ChainStateDB2>>,
    /// Block executed data for VM1 (txn_infos, events, etc.)
    pub executed_data: Option<BlockExecutedData1>,
    /// Block executed data for VM2 (txn_infos, events, etc.)
    pub executed_data2: Option<BlockExecutedData2>,
}

impl CachedBlockState {
    pub fn new(
        statedb: Option<Arc<ChainStateDB>>,
        statedb2: Option<Arc<ChainStateDB2>>,
        executed_data: Option<BlockExecutedData1>,
        executed_data2: Option<BlockExecutedData2>,
    ) -> Self {
        Self {
            statedb,
            statedb2,
            executed_data,
            executed_data2,
        }
    }
}

/// Cache for block state, keyed by txn_accumulator_root
pub struct BlockStateCache {
    cache: DashMap<HashValue, CachedBlockState>,
}

impl BlockStateCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Insert cached state using txn_accumulator_root as key
    pub fn insert(&self, txn_accumulator_root: HashValue, state: CachedBlockState) {
        self.cache.insert(txn_accumulator_root, state);
    }

    /// Get cached state
    pub fn get(&self, txn_accumulator_root: HashValue) -> Option<CachedBlockState> {
        self.cache.get(&txn_accumulator_root).map(|v| v.clone())
    }

    /// Remove and return cached state
    pub fn remove(&self, txn_accumulator_root: HashValue) -> Option<CachedBlockState> {
        self.cache.remove(&txn_accumulator_root).map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for BlockStateCache {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_BLOCK_STATE_CACHE: Lazy<BlockStateCache> = Lazy::new(BlockStateCache::new);

pub fn global_block_state_cache() -> &'static BlockStateCache {
    &GLOBAL_BLOCK_STATE_CACHE
}

/// Cache for transaction outputs, keyed by txn_accumulator_root
///
/// The txn_accumulator_root uniquely identifies a transaction list,
/// so it can be used as a cache key even before the block_id is known.
pub struct TransactionOutputCache {
    cache: DashMap<HashValue, CachedBlockOutputs>,
}

impl TransactionOutputCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Insert outputs using txn_accumulator_root as key
    pub fn insert_outputs(
        &self,
        txn_accumulator_root: HashValue,
        vm1_outputs: Option<Vec<TransactionOutput1>>,
        vm2_outputs: Option<Vec<TransactionOutput2>>,
    ) {
        let cached = CachedBlockOutputs::new(vm1_outputs, vm2_outputs);
        self.cache.insert(txn_accumulator_root, cached);
    }

    /// Get outputs using txn_accumulator_root as key
    pub fn get(&self, txn_accumulator_root: HashValue) -> Option<CachedBlockOutputs> {
        self.cache.get(&txn_accumulator_root).map(|v| v.clone())
    }

    /// Remove outputs using txn_accumulator_root as key
    pub fn remove(&self, txn_accumulator_root: HashValue) {
        self.cache.remove(&txn_accumulator_root);
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for TransactionOutputCache {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_TXN_OUTPUT_CACHE: Lazy<TransactionOutputCache> =
    Lazy::new(TransactionOutputCache::new);

pub fn global_txn_output_cache() -> &'static TransactionOutputCache {
    &GLOBAL_TXN_OUTPUT_CACHE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = TransactionOutputCache::new();
        let txn_root = HashValue::random();

        assert!(cache.is_empty());
        assert!(cache.get(txn_root).is_none());

        cache.insert_outputs(txn_root, None, None);
        assert_eq!(cache.len(), 1);

        let result = cache.get(txn_root);
        assert!(result.is_some());
    }

    #[test]
    fn test_cache_remove() {
        let cache = TransactionOutputCache::new();
        let txn_root = HashValue::random();

        cache.insert_outputs(txn_root, None, None);
        assert_eq!(cache.len(), 1);

        cache.remove(txn_root);
        assert!(cache.is_empty());
        assert!(cache.get(txn_root).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = TransactionOutputCache::new();

        for _ in 0..5 {
            let txn_root = HashValue::random();
            cache.insert_outputs(txn_root, None, None);
        }

        assert_eq!(cache.len(), 5);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_global_cache() {
        let cache = global_txn_output_cache();
        let initial_len = cache.len();

        let txn_root = HashValue::random();
        cache.insert_outputs(txn_root, None, None);

        assert_eq!(cache.len(), initial_len + 1);

        cache.remove(txn_root);
        assert_eq!(cache.len(), initial_len);
    }
}
