mod stc_cache;
pub use stc_cache::*;

pub trait DagCache {
    type TKey: Clone + std::hash::Hash + Eq + Send + Sync + AsRef<[u8]>;
    type TData: Clone + Send + Sync + AsRef<[u8]>;

    fn new_with_capacity(size: u64) -> Self;
    fn get(&self, key: &Self::TKey) -> Option<Self::TData>;
    fn contains_key(&self, key: &Self::TKey) -> bool;
    fn insert(&self, key: Self::TKey, data: Self::TData);
    fn remove(&self, key: &Self::TKey);
    fn remove_many(&self, key_iter: &mut impl Iterator<Item = Self::TKey>);
    fn remove_all(&self);
}
