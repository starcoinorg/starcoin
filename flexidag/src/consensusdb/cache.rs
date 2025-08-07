use core::hash::Hash;
use lru::LruCache;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct DagCache<K: Hash + Eq + Default, V: Default> {
    cache: Arc<Mutex<LruCache<K, V>>>,
}

impl<K, V> DagCache<K, V>
where
    K: Hash + Eq + Default + Clone,
    V: Default + Clone,
{
    pub(crate) fn new_with_capacity(size: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(size))),
        }
    }

    pub(crate) fn get(&self, key: &K) -> Option<V> {
        self.cache.lock().get(key).cloned()
    }

    pub(crate) fn contains_key(&self, key: &K) -> bool {
        self.cache.lock().contains(key)
    }

    pub(crate) fn insert(&self, key: K, data: V) {
        self.cache.lock().put(key, data);
    }

    pub(crate) fn remove(&self, key: &K) -> Option<V> {
        self.cache.lock().pop(key)
    }

    pub(crate) fn remove_all(&self) {
        self.cache.lock().clear();
    }

    pub(crate) fn remove_many<I>(&self, keys: I) 
    where
        I: Iterator<Item = K>,
    {
        let mut cache = self.cache.lock();
        for key in keys {
            cache.pop(&key);
        }
    }
}