use anyhow::{bail, Result};
use forkable_jellyfish_merkle::blob::Blob;
use forkable_jellyfish_merkle::node_type::{LeafNode, Node, NodeKey};
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use forkable_jellyfish_merkle::{
    tree_cache::TreeCache, JellyfishMerkleTree, StaleNodeIndex, TreeReader, TreeUpdateBatch,
    SPARSE_MERKLE_PLACEHOLDER_HASH,
};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNode(pub Node);

impl StateNode {
    pub fn inner(&self) -> &Node {
        &self.0
    }
}

impl From<Node> for StateNode {
    fn from(n: Node) -> Self {
        StateNode(n)
    }
}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHash)]
pub struct StateProof {}

pub trait StateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>>;
    fn put(&self, key: HashValue, node: StateNode) -> Result<()>;
}

pub struct StateCache {
    root_hash: HashValue,
    change_set: TreeUpdateBatch,
}

impl StateCache {
    fn reset(&mut self, root_hash: HashValue) {
        self.root_hash = root_hash;
        self.change_set = TreeUpdateBatch::default();
    }

    fn add_changeset(&mut self, root_hash: HashValue, cs: TreeUpdateBatch) {
        let cur_change_set = &mut self.change_set;
        let mut cs_num_stale_leaves = cs.num_stale_leaves;
        for stale_node in cs.stale_node_index_batch.iter() {
            match cur_change_set.node_batch.remove(&stale_node.node_key) {
                None => {
                    cur_change_set
                        .stale_node_index_batch
                        .insert(StaleNodeIndex {
                            stale_since_version: root_hash,
                            node_key: stale_node.node_key.clone(),
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
            cur_change_set.node_batch.insert(nk.clone(), n.clone());
            if n.is_leaf() {
                cur_change_set.num_new_leaves += 1;
            }
        }

        self.root_hash = root_hash;
    }
}

impl StateCache {
    pub fn new(initial_root: HashValue) -> Self {
        Self {
            root_hash: initial_root,
            change_set: TreeUpdateBatch::default(),
        }
    }
}

pub struct StateTree {
    storage: Arc<dyn StateNodeStore>,
    storage_root_hash: RwLock<HashValue>,
    cache: Mutex<StateCache>,
}

impl StateTree {
    /// Construct a new state_db from provided `state_root_hash` with underline `state_storage`
    pub fn new(state_storage: Arc<dyn StateNodeStore>, state_root_hash: Option<HashValue>) -> Self {
        let state_root_hash = state_root_hash.unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH);
        Self {
            storage: state_storage,
            storage_root_hash: RwLock::new(state_root_hash),
            cache: Mutex::new(StateCache::new(state_root_hash)),
        }
    }

    /// get current root hash
    pub fn root_hash(&self) -> HashValue {
        self.cache.lock().unwrap().root_hash
    }

    pub fn is_empty(&self) -> bool {
        self.root_hash() == *SPARSE_MERKLE_PLACEHOLDER_HASH
    }

    /// put a kv pair into tree.
    /// Users need to hash the origin key into a fixed-length(here is 256bit) HashValue,
    /// and use it as the `key_hash`.
    pub fn put(&self, key_hash: HashValue, value: Vec<u8>) -> Result<HashValue> {
        self.put_blob_set(vec![(key_hash, value)])
    }

    /// use a key's hash `key_hash` to read a value.
    pub fn get(&self, key_hash: &HashValue) -> Result<Option<Vec<u8>>> {
        Ok(self.get_with_proof(key_hash)?.0)
    }

    /// return value with it proof
    pub fn get_with_proof(
        &self,
        key_hash: &HashValue,
    ) -> Result<(Option<Vec<u8>>, SparseMerkleProof)> {
        let mut cache_guard = self.cache.lock().unwrap();
        let cache = cache_guard.deref_mut();
        let cur_root_hash = cache.root_hash;
        let reader = CachedTreeReader {
            store: self.storage.as_ref(),
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let (data, proof) = tree.get_with_proof(cur_root_hash, key_hash.clone())?;
        match data {
            Some(b) => Ok((Some(b.into()), proof)),
            None => Ok((None, proof)),
        }
    }

    /// Remove key_hash's data and return new root.
    pub fn remove(&self, key_hash: &HashValue) -> Result<HashValue> {
        self.updates(vec![(key_hash.clone(), None)])
    }

    pub fn contains(&self, key_hash: &HashValue) -> Result<bool> {
        self.get(key_hash).map(|result| result.is_some())
    }

    /// write a new states into local cache, return new root hash
    pub fn put_blob_set(&self, blob_set: Vec<(HashValue, Vec<u8>)>) -> Result<HashValue> {
        let blob_set = blob_set
            .into_iter()
            .map(|(k, v)| (k, Some(v)))
            .collect::<Vec<_>>();
        self.updates(blob_set)
    }

    /// passing None value with a key means delete the key
    fn updates(&self, updates: Vec<(HashValue, Option<Vec<u8>>)>) -> Result<HashValue> {
        let cur_root_hash = self.root_hash();
        let mut cache_guard = self.cache.lock().unwrap();
        let mut cache = cache_guard.deref_mut();
        let reader = CachedTreeReader {
            store: self.storage.as_ref(),
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let blob_set = updates
            .into_iter()
            .map(|(k, v)| (k, v.map(|d| d.into())))
            .collect::<Vec<_>>();
        let (new_state_root, change_set) = tree.updates(Some(cur_root_hash), blob_set)?;
        // cache.root_hashes.push(new_state_root);
        // cache.change_sets.push(change_set);
        // cache.root_hash = new_state_root;
        cache.add_changeset(new_state_root, change_set);
        Ok(new_state_root)
    }

    //
    // /// rollback last write
    // pub fn rollback(&self) {
    //     let mut cache_guard = self.cache.lock().unwrap();
    //     if let Some(root_hash) = cache_guard.root_hashes.pop() {
    //         let _ = cache_guard.change_sets.pop();
    //     }
    // }
    //
    // /// rollback current state to a history state with the provided `root_hash`
    // pub fn rollback_to(&self, root_hash: HashValue) -> Result<()> {
    //     let mut cache_guard = self.cache.lock().unwrap();
    //     let mut state_index = None;
    //     for (i, root) in cache_guard.root_hashes.iter().enumerate() {
    //         if root == &root_hash {
    //             state_index = Some(i);
    //         }
    //     }
    //
    //     if let Some(i) = state_index {
    //         cache_guard.truncate(i + 1);
    //     } else if self.storage_root_hash.read().unwrap().deref() == &root_hash {
    //         cache_guard.clear();
    //     } else {
    //         bail!("the root_hash is not found in write history");
    //     }
    //     Ok(())
    // }

    #[cfg(test)]
    pub fn change_sets(&self) -> (HashValue, TreeUpdateBatch) {
        self.get_change_sets()
    }

    /// get all changes so far based on initial root_hash.
    fn get_change_sets(&self) -> (HashValue, TreeUpdateBatch) {
        let mut cache_guard = self.cache.lock().unwrap();
        (cache_guard.root_hash, cache_guard.change_set.clone())
    }

    /// commit the state change into underline storage.
    pub fn commit(&self) -> Result<()> {
        let (root_hash, change_sets) = self.get_change_sets();

        // TODO: save in batch to preserve atomic.
        for (nk, n) in change_sets.node_batch.into_iter() {
            self.storage.put(nk, StateNode(n))?;
        }
        // and then advance the storage root hash
        *self.storage_root_hash.write().unwrap() = root_hash;
        self.cache.lock().unwrap().reset(root_hash);
        Ok(())
    }

    // TODO: to keep atomic with other commit.
    // TODO: think about the WriteBatch trait position.
    // pub fn save<T>(&self, batch: &mut T) -> Result<()>
    // where
    //     T: WriteBatch,
    // {
    //     todo!()
    // }
}

struct CachedTreeReader<'a> {
    store: &'a dyn StateNodeStore,
    cache: &'a StateCache,
}

impl<'a> TreeReader for CachedTreeReader<'a> {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        if let Some(n) = self.cache.change_set.node_batch.get(node_key).cloned() {
            return Ok(Some(n));
        }
        match self.store.get(node_key) {
            Ok(Some(n)) => Ok(Some(n.0)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
