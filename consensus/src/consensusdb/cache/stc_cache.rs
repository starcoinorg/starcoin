use super::DagCache;
use starcoin_storage::cache_storage::CacheStorage;
use std::{marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct Cache<TKey> {
    cache: Arc<CacheStorage>,
    _phantom: PhantomData<TKey>,
}

impl<TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>> DagCache for Cache<TKey> {
    type TKey = TKey;
    type TData = Vec<u8>;

    fn new_with_capacity(size: u64) -> Self {
        Self {
            cache: Arc::new(CacheStorage::new_with_capacity(size as usize, None)),
            _phantom: Default::default(),
        }
    }

    fn get(&self, key: &Self::TKey) -> Option<Self::TData> {
        self.cache.get_inner(None, key.as_ref().to_vec())
    }

    fn contains_key(&self, key: &Self::TKey) -> bool {
        self.get(key).is_some()
    }

    fn insert(&self, key: Self::TKey, data: Self::TData) {
        self.cache.put_inner(None, key.as_ref().to_vec(), data);
    }

    fn remove(&self, key: &Self::TKey) {
        self.cache.remove_inner(None, key.as_ref().to_vec());
    }

    fn remove_many(&self, key_iter: &mut impl Iterator<Item = Self::TKey>) {
        key_iter.for_each(|k| self.remove(&k));
    }

    fn remove_all(&self) {
        self.cache.remove_all();
    }
}
