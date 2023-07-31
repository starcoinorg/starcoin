use core::hash::Hash;
use starcoin_storage::cache_storage::GCacheStorage;
use std::sync::Arc;

#[derive(Clone)]
pub struct DagCache<K: Hash + Eq + Default, V: Default> {
    cache: Arc<GCacheStorage<K, V>>,
}

impl<K, V> DagCache<K, V>
where
    K: Hash + Eq + Default,
    V: Default + Clone,
{
    pub(crate) fn new_with_capacity(size: u64) -> Self {
        Self {
            cache: Arc::new(GCacheStorage::new_with_capacity(size as usize, None)),
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<V> {
        self.cache.get_inner(key)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub(crate) fn insert(&self, key: K, data: V) {
        self.cache.put_inner(key, data);
    }

    pub(crate) fn remove(&self, key: &K) {
        self.cache.remove_inner(key);
    }

    pub(crate) fn remove_many(&self, key_iter: &mut impl Iterator<Item = K>) {
        key_iter.for_each(|k| self.remove(&k));
    }

    pub(crate) fn remove_all(&self) {
        self.cache.remove_all();
    }
}
