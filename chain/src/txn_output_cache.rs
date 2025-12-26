// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use dashmap::DashMap;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_executor::BlockExecutedData as BlockExecutedData1;
use starcoin_statedb::ChainStateDB;
use starcoin_vm2_executor::block_executor::BlockExecutedData as BlockExecutedData2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use std::sync::Arc;

/// Maximum number of entries in the block state cache.
/// Since this cache is for blocks being mined, a small limit is sufficient.
const MAX_BLOCK_STATE_CACHE_SIZE: usize = 16;

/// Cached block state including StateDB and execution results.
/// This allows block execution to skip re-execution and apply_write_set entirely.
///
/// When a block is finalized during mining, the StateDB already has all write_sets
/// applied and committed. By caching this state along with BlockExecutedData,
/// we can completely skip re-execution when the mined block is received.
#[derive(Clone)]
pub struct CachedBlockState {
    /// Cached StateDB for VM1 (already applied write_sets and committed, not flushed)
    pub statedb: Option<Arc<ChainStateDB>>,
    /// Cached StateDB for VM2 (already applied write_sets and committed, not flushed)
    pub statedb2: Option<Arc<ChainStateDB2>>,
    /// Block executed data for VM1 (txn_infos, events, table_infos, write_sets)
    pub executed_data: Option<BlockExecutedData1>,
    /// Block executed data for VM2 (txn_infos, events, table_infos)
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

    /// Check if this cached state is complete (all fields present)
    pub fn is_complete(&self) -> bool {
        self.statedb.is_some()
            && self.statedb2.is_some()
            && self.executed_data.is_some()
            && self.executed_data2.is_some()
    }
}

/// Cache for block state, keyed by txn_accumulator_root.
///
/// The txn_accumulator_root uniquely identifies a block's transaction list,
/// making it an ideal cache key even before the block_id (which includes nonce) is known.
pub struct BlockStateCache {
    cache: DashMap<HashValue, CachedBlockState>,
    max_size: usize,
}

impl BlockStateCache {
    pub fn new() -> Self {
        Self::with_capacity(MAX_BLOCK_STATE_CACHE_SIZE)
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            cache: DashMap::new(),
            max_size,
        }
    }

    /// Insert cached state using txn_accumulator_root as key.
    /// If cache exceeds max_size, older entries are evicted.
    pub fn insert(&self, txn_accumulator_root: HashValue, state: CachedBlockState) {
        // Evict entries if cache is full
        if self.cache.len() >= self.max_size {
            let to_remove: Vec<_> = self
                .cache
                .iter()
                .take(self.max_size / 2)
                .map(|entry| *entry.key())
                .collect();
            for key in to_remove {
                self.cache.remove(&key);
            }
            log::debug!(
                "BlockStateCache evicted entries, new size: {}",
                self.cache.len()
            );
        }
        self.cache.insert(txn_accumulator_root, state);
    }

    /// Get cached state (without removing)
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

static GLOBAL_SHUTDOWN_FLAG: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn global_block_state_cache() -> &'static BlockStateCache {
    &GLOBAL_BLOCK_STATE_CACHE
}

pub fn is_node_shutting_down() -> bool {
    GLOBAL_SHUTDOWN_FLAG.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn set_node_shutting_down() {
    GLOBAL_SHUTDOWN_FLAG.store(true, std::sync::atomic::Ordering::SeqCst);
}

pub fn reset_node_shutdown_flag() {
    GLOBAL_SHUTDOWN_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
}

pub fn clear_global_block_state_cache() {
    set_node_shutting_down();
    let count = GLOBAL_BLOCK_STATE_CACHE.len();
    GLOBAL_BLOCK_STATE_CACHE.clear();
    log::info!(
        "Global block state cache cleared, removed {} entries",
        count
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_state_cache_insert_and_get() {
        let cache = BlockStateCache::new();
        let txn_root = HashValue::random();

        assert!(cache.is_empty());
        assert!(cache.get(txn_root).is_none());

        let state = CachedBlockState::new(None, None, None, None);
        cache.insert(txn_root, state);
        assert_eq!(cache.len(), 1);

        let result = cache.get(txn_root);
        assert!(result.is_some());
    }

    #[test]
    fn test_block_state_cache_remove() {
        let cache = BlockStateCache::new();
        let txn_root = HashValue::random();

        let state = CachedBlockState::new(None, None, None, None);
        cache.insert(txn_root, state);
        assert_eq!(cache.len(), 1);

        let removed = cache.remove(txn_root);
        assert!(removed.is_some());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_global_cache() {
        let cache = global_block_state_cache();
        let initial_len = cache.len();

        let txn_root = HashValue::random();
        let state = CachedBlockState::new(None, None, None, None);
        cache.insert(txn_root, state);

        assert_eq!(cache.len(), initial_len + 1);

        cache.remove(txn_root);
        assert_eq!(cache.len(), initial_len);
    }
}
