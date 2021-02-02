use anyhow::Result;
use forkable_jellyfish_merkle::blob::Blob;
use forkable_jellyfish_merkle::iterator::JellyfishMerkleIterator;
use forkable_jellyfish_merkle::node_type::{Node, NodeKey};
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use forkable_jellyfish_merkle::{
    JellyfishMerkleTree, RawKey, StaleNodeIndex, TreeReader, TreeUpdateBatch,
};
use parking_lot::{Mutex, RwLock};
use starcoin_crypto::hash::*;
use starcoin_state_store_api::*;
use starcoin_types::state_set::StateSet;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ops::DerefMut;
use std::sync::Arc;

pub struct StateCache<K: RawKey> {
    root_hash: HashValue,
    change_set: TreeUpdateBatch<K>,
}

impl<K> StateCache<K>
where
    K: RawKey,
{
    pub fn new(initial_root: HashValue) -> Self {
        Self {
            root_hash: initial_root,
            change_set: TreeUpdateBatch::default(),
        }
    }

    fn reset(&mut self, root_hash: HashValue) {
        self.root_hash = root_hash;
        self.change_set = TreeUpdateBatch::default();
    }

    fn add_changeset(&mut self, root_hash: HashValue, cs: TreeUpdateBatch<K>) {
        let cur_change_set = &mut self.change_set;
        let mut cs_num_stale_leaves = cs.num_stale_leaves;
        for stale_node in cs.stale_node_index_batch.iter() {
            match cur_change_set.node_batch.remove(&stale_node.node_key) {
                None => {
                    cur_change_set
                        .stale_node_index_batch
                        .insert(StaleNodeIndex {
                            stale_since_version: root_hash,
                            node_key: stale_node.node_key,
                        });
                }
                Some(n) => {
                    if n.is_leaf() {
                        cur_change_set.num_new_leaves -= 1;
                        cs_num_stale_leaves -= 1;
                    }
                }
            }
        }
        cur_change_set.num_stale_leaves += cs_num_stale_leaves;
        for (nk, n) in cs.node_batch.iter() {
            cur_change_set.node_batch.insert(*nk, n.clone());
            if n.is_leaf() {
                cur_change_set.num_new_leaves += 1;
            }
        }

        self.root_hash = root_hash;
    }
}

//TODO remove the Lock.
pub struct StateTree<K: RawKey> {
    storage: Arc<dyn StateNodeStore>,
    storage_root_hash: RwLock<HashValue>,
    updates: RwLock<BTreeMap<K, Option<Blob>>>,
    cache: Mutex<StateCache<K>>,
}

impl<K> Clone for StateTree<K>
where
    K: RawKey,
{
    fn clone(&self) -> Self {
        StateTree::new(self.storage.clone(), Some(*self.storage_root_hash.read()))
    }
}

impl<K> StateTree<K>
where
    K: RawKey,
{
    /// Construct a new state_db from provided `state_root_hash` with underline `state_storage`
    pub fn new(state_storage: Arc<dyn StateNodeStore>, state_root_hash: Option<HashValue>) -> Self {
        let state_root_hash = state_root_hash.unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        Self {
            storage: state_storage,
            storage_root_hash: RwLock::new(state_root_hash),
            updates: RwLock::new(BTreeMap::new()),
            cache: Mutex::new(StateCache::new(state_root_hash)),
        }
    }

    /// get current root hash
    /// if any modification is not committed into state tree, the root hash is not changed.
    /// You can use `commit` to make current modification committed into local state tree.
    pub fn root_hash(&self) -> HashValue {
        self.cache.lock().root_hash
    }

    /// put a kv pair into tree.
    /// Users need to hash the origin key into a fixed-length(here is 256bit) HashValue,
    /// and use it as the `key_hash`.
    /// this will not compute new root hash,
    /// Use `commit` to recompute the root hash.
    pub fn put(&self, key: K, value: Vec<u8>) {
        self.updates.write().insert(key, Some(value.into()));
    }

    /// Remove key_hash's data.
    /// this will not compute new root hash,
    /// Use `commit` to recompute the root hash.
    pub fn remove(&self, key: &K) {
        self.updates.write().insert(key.clone(), None);
    }

    /// use a key's hash `key_hash` to read a value.
    /// This will also read un-committed modification.
    pub fn get(&self, key: &K) -> Result<Option<Vec<u8>>> {
        let updates_guard = self.updates.read();
        if let Some(uncomputed) = updates_guard.get(key).cloned() {
            return Ok(uncomputed.map(|b| b.into()));
        }
        Ok(self.get_with_proof(key)?.0)
    }

    pub fn contains(&self, key: &K) -> Result<bool> {
        self.get(key).map(|result| result.is_some())
    }

    /// return value with it proof.
    /// NOTICE: this will only read from state tree.
    /// Any un-committed modification will not visible to the method.
    pub fn get_with_proof(&self, key: &K) -> Result<(Option<Vec<u8>>, SparseMerkleProof)> {
        let mut cache_guard = self.cache.lock();
        let cache = cache_guard.deref_mut();
        let cur_root_hash = cache.root_hash;
        let reader = CachedTreeReader {
            store: self.storage.as_ref(),
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let (data, proof) = tree.get_with_proof(cur_root_hash, key.key_hash())?;
        match data {
            Some(b) => Ok((Some(b.into()), proof)),
            None => Ok((None, proof)),
        }
    }

    /// Commit current modification into state tree's local cache,
    /// and return new root hash.
    /// NOTICE: this method will not flush the changes into disk.
    /// It'just commit the changes into local state-tree, and cache it there.
    pub fn commit(&self) -> Result<HashValue> {
        let mut guard = self.updates.write();
        let updates = guard
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        let new_root_hash = self.updates(updates)?;
        guard.clear();
        Ok(new_root_hash)
    }

    /// check if there is data that has not been commit.
    pub fn is_dirty(&self) -> bool {
        self.updates.read().len() > 0
    }

    /// Write state_set to state tree.
    pub fn apply(&self, state_set: StateSet) -> Result<()> {
        let inner: Vec<(Vec<u8>, Vec<u8>)> = state_set.into();
        let updates = inner
            .into_iter()
            .map(|(k, v)| Ok((K::decode_key(k.as_slice())?, Some(v.into()))))
            .collect::<Result<Vec<_>>>();
        self.updates(updates?)?;
        Ok(())
    }

    /// commit the state change into underline storage.
    pub fn flush(&self) -> Result<()> {
        let (root_hash, change_sets) = self.change_sets();

        let mut node_map = BTreeMap::new();
        for (nk, n) in change_sets.node_batch.into_iter() {
            node_map.insert(nk, n.try_into()?);
        }
        self.storage.write_nodes(node_map)?;
        // and then advance the storage root hash
        *self.storage_root_hash.write() = root_hash;
        self.cache.lock().reset(root_hash);
        Ok(())
    }

    /// Dump tree to state set.
    pub fn dump(&self) -> Result<StateSet> {
        let cur_root_hash = self.root_hash();
        let mut cache_guard = self.cache.lock();
        let cache = cache_guard.deref_mut();
        let reader = CachedTreeReader {
            store: self.storage.as_ref(),
            cache,
        };
        let iterator = JellyfishMerkleIterator::new(&reader, cur_root_hash, HashValue::zero())?;
        let mut states = vec![];
        for item in iterator {
            let item = item?;
            states.push((item.0.encode_key()?, item.1.into()));
        }
        Ok(StateSet::new(states))
    }

    /// passing None value with a key means delete the key
    fn updates(&self, updates: Vec<(K, Option<Blob>)>) -> Result<HashValue> {
        let cur_root_hash = self.root_hash();
        //TODO should throw a error?
        if updates.is_empty() {
            return Ok(cur_root_hash);
        }
        let mut cache_guard = self.cache.lock();
        let cache = cache_guard.deref_mut();
        let reader = CachedTreeReader {
            store: self.storage.as_ref(),
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let (new_state_root, change_set) = tree.updates(Some(cur_root_hash), updates)?;
        // cache.root_hashes.push(new_state_root);
        // cache.change_sets.push(change_set);
        // cache.root_hash = new_state_root;
        cache.add_changeset(new_state_root, change_set);
        Ok(new_state_root)
    }

    //
    // /// rollback last write
    // pub fn rollback(&self) {
    //     let mut cache_guard = self.cache.lock();
    //     if let Some(root_hash) = cache_guard.root_hashes.pop() {
    //         let _ = cache_guard.change_sets.pop();
    //     }
    // }
    //
    // /// rollback current state to a history state with the provided `root_hash`
    // pub fn rollback_to(&self, root_hash: HashValue) -> Result<()> {
    //     let mut cache_guard = self.cache.lock();
    //     let mut state_index = None;
    //     for (i, root) in cache_guard.root_hashes.iter().enumerate() {
    //         if root == &root_hash {
    //             state_index = Some(i);
    //         }
    //     }
    //
    //     if let Some(i) = state_index {
    //         cache_guard.truncate(i + 1);
    //     } else if self.storage_root_hash.read().deref() == &root_hash {
    //         cache_guard.clear();
    //     } else {
    //         bail!("the root_hash is not found in write history");
    //     }
    //     Ok(())
    // }

    /// get all changes so far based on initial root_hash.
    pub fn change_sets(&self) -> (HashValue, TreeUpdateBatch<K>) {
        let cache_guard = self.cache.lock();
        (cache_guard.root_hash, cache_guard.change_set.clone())
    }
    // TODO: to keep atomic with other commit.
    // TODO: think about the WriteBatch trait position.
    // pub fn save<T>(&self, batch: &mut T) -> Result<()>
    // where
    //     T: WriteBatch,
    // {
    //     todo!()
    // }

    pub fn is_genesis(&self) -> bool {
        self.root_hash() == *SPARSE_MERKLE_PLACEHOLDER_HASH
    }
}

struct CachedTreeReader<'a, K: RawKey> {
    store: &'a dyn StateNodeStore,
    cache: &'a StateCache<K>,
}

impl<'a, K> TreeReader<K> for CachedTreeReader<'a, K>
where
    K: RawKey,
{
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<K>>> {
        if node_key == &*SPARSE_MERKLE_PLACEHOLDER_HASH {
            return Ok(Some(Node::new_null()));
        }
        if let Some(n) = self.cache.change_set.node_batch.get(node_key).cloned() {
            return Ok(Some(n));
        }
        match self.store.get(node_key) {
            Ok(Some(n)) => Ok(Some(n.try_into()?)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
