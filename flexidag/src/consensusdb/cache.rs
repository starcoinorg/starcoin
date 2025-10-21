use core::hash::Hash;
use quick_cache::sync::Cache;
use std::sync::Arc;

#[derive(Clone)]
pub struct DagCache<K: Hash + Eq + Default, V: Default> {
    cache: Arc<Cache<K, V>>,
}

impl<K, V> DagCache<K, V>
where
    K: Hash + Eq + Default + Clone,
    V: Default + Clone,
{
    pub(crate) fn new_with_capacity(size: usize) -> Self {
        Self {
            cache: Arc::new(Cache::new(size)),
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key)
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.cache.contains_key(key)
    }

    pub(crate) fn insert(&self, key: K, data: V) {
        self.cache.insert(key, data);
    }

    pub(crate) fn remove(&self, key: &K) -> Option<V> {
        self.cache.remove(key).map(|(_, v)| v)
    }

    pub(crate) fn remove_all(&self) {
        self.cache.clear();
    }

    pub(crate) fn remove_many<I>(&self, keys: I)
    where
        I: Iterator<Item = K>,
    {
        for key in keys {
            self.cache.remove(&key);
        }
    }
}
