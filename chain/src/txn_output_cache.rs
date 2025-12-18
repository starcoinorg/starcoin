use dashmap::DashMap;
use once_cell::sync::Lazy;
use starcoin_crypto::HashValue;
use starcoin_vm2_vm_types::transaction::TransactionOutput as TransactionOutput2;
use starcoin_vm_types::transaction::TransactionOutput as TransactionOutput1;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CacheKey {
    pub block_id: HashValue,
    pub state_root: HashValue,
}

impl CacheKey {
    pub fn new(block_id: HashValue, state_root: HashValue) -> Self {
        Self {
            block_id,
            state_root,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedBlockOutputs {
    pub vm1_outputs: Option<Vec<TransactionOutput1>>,
    pub vm2_outputs: Option<Vec<TransactionOutput2>>,
}

impl CachedBlockOutputs {
    pub fn new_vm1(outputs: Vec<TransactionOutput1>) -> Self {
        Self {
            vm1_outputs: Some(outputs),
            vm2_outputs: None,
        }
    }

    pub fn new_vm2(outputs: Vec<TransactionOutput2>) -> Self {
        Self {
            vm1_outputs: None,
            vm2_outputs: Some(outputs),
        }
    }

    pub fn with_both(
        vm1_outputs: Vec<TransactionOutput1>,
        vm2_outputs: Vec<TransactionOutput2>,
    ) -> Self {
        Self {
            vm1_outputs: Some(vm1_outputs),
            vm2_outputs: Some(vm2_outputs),
        }
    }
}

pub struct TransactionOutputCache {
    cache: DashMap<CacheKey, Arc<CachedBlockOutputs>>,
    cache_by_block_id: DashMap<HashValue, Arc<CachedBlockOutputs>>,
}

impl TransactionOutputCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            cache_by_block_id: DashMap::new(),
        }
    }

    pub fn insert_vm1_outputs(
        &self,
        block_id: HashValue,
        state_root: HashValue,
        outputs: Vec<TransactionOutput1>,
    ) {
        let key = CacheKey::new(block_id, state_root);
        let cached = Arc::new(CachedBlockOutputs::new_vm1(outputs.clone()));
        self.cache.insert(key, cached.clone());
        self.cache_by_block_id.insert(block_id, cached);
    }

    pub fn insert_vm2_outputs(
        &self,
        block_id: HashValue,
        state_root: HashValue,
        outputs: Vec<TransactionOutput2>,
    ) {
        let key = CacheKey::new(block_id, state_root);
        let cached = Arc::new(CachedBlockOutputs::new_vm2(outputs.clone()));
        self.cache.insert(key, cached.clone());
        self.cache_by_block_id.insert(block_id, cached);
    }

    pub fn insert_outputs(
        &self,
        block_id: HashValue,
        state_root: HashValue,
        vm1_outputs: Option<Vec<TransactionOutput1>>,
        vm2_outputs: Option<Vec<TransactionOutput2>>,
    ) {
        let cached = Arc::new(CachedBlockOutputs {
            vm1_outputs: vm1_outputs.clone(),
            vm2_outputs: vm2_outputs.clone(),
        });
        let key = CacheKey::new(block_id, state_root);
        self.cache.insert(key, cached.clone());
        self.cache_by_block_id.insert(block_id, cached);
    }

    pub fn get(
        &self,
        block_id: HashValue,
        state_root: HashValue,
    ) -> Option<Arc<CachedBlockOutputs>> {
        let key = CacheKey::new(block_id, state_root);
        self.cache.get(&key).map(|v| v.value().clone())
    }

    pub fn get_by_block_id(&self, block_id: HashValue) -> Option<Arc<CachedBlockOutputs>> {
        self.cache_by_block_id
            .get(&block_id)
            .map(|v| v.value().clone())
    }

    pub fn get_vm1_outputs(
        &self,
        block_id: HashValue,
        state_root: HashValue,
    ) -> Option<Vec<TransactionOutput1>> {
        self.get(block_id, state_root)
            .and_then(|cached| cached.vm1_outputs.clone())
    }

    pub fn get_vm2_outputs(
        &self,
        block_id: HashValue,
        state_root: HashValue,
    ) -> Option<Vec<TransactionOutput2>> {
        self.get(block_id, state_root)
            .and_then(|cached| cached.vm2_outputs.clone())
    }

    pub fn get_vm1_outputs_by_block_id(
        &self,
        block_id: HashValue,
    ) -> Option<Vec<TransactionOutput1>> {
        self.get_by_block_id(block_id)
            .and_then(|cached| cached.vm1_outputs.clone())
    }

    pub fn get_vm2_outputs_by_block_id(
        &self,
        block_id: HashValue,
    ) -> Option<Vec<TransactionOutput2>> {
        self.get_by_block_id(block_id)
            .and_then(|cached| cached.vm2_outputs.clone())
    }

    pub fn remove(&self, block_id: HashValue, state_root: HashValue) {
        let key = CacheKey::new(block_id, state_root);
        self.cache.remove(&key);
        self.cache_by_block_id.remove(&block_id);
    }

    pub fn remove_by_block_id(&self, block_id: HashValue) {
        self.cache_by_block_id.remove(&block_id);
        self.cache.retain(|k, _| k.block_id != block_id);
    }

    pub fn clear(&self) {
        self.cache.clear();
        self.cache_by_block_id.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn evict_if_needed(&self, max_size: usize) {
        if self.cache.len() > max_size {
            let to_remove = self.cache.len() - max_size;
            let keys_to_remove: Vec<CacheKey> = self
                .cache
                .iter()
                .take(to_remove)
                .map(|entry| *entry.key())
                .collect();

            for key in keys_to_remove {
                self.cache.remove(&key);
                self.cache_by_block_id.remove(&key.block_id);
            }
        }
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
    fn test_cache_insert_and_get_vm1() {
        let cache = TransactionOutputCache::new();
        let block_id = HashValue::random();
        let state_root = HashValue::random();

        let outputs = vec![];
        cache.insert_vm1_outputs(block_id, state_root, outputs.clone());

        let cached = cache.get(block_id, state_root);
        assert!(cached.is_some());
        assert!(cached.unwrap().vm1_outputs.is_some());

        let cached_by_id = cache.get_by_block_id(block_id);
        assert!(cached_by_id.is_some());
    }

    #[test]
    fn test_cache_remove() {
        let cache = TransactionOutputCache::new();
        let block_id = HashValue::random();
        let state_root = HashValue::random();

        cache.insert_vm1_outputs(block_id, state_root, vec![]);
        assert!(!cache.is_empty());

        cache.remove(block_id, state_root);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_global_cache() {
        let cache = global_txn_output_cache();
        let block_id = HashValue::random();
        let state_root = HashValue::random();

        cache.insert_vm1_outputs(block_id, state_root, vec![]);

        let cache2 = global_txn_output_cache();
        assert!(cache2.get(block_id, state_root).is_some());

        cache.remove(block_id, state_root);
    }
}
