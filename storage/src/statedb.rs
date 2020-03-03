use crate::storage::CodecStorage;
use crate::storage::Repository;
use crate::storage::{KeyCodec, ValueCodec};
use anyhow::bail;
use anyhow::Result;
use crypto::hash::*;
use forkable_jellyfish_merkle::blob::Blob;
use forkable_jellyfish_merkle::node_type::{LeafNode, Node, NodeKey};
use forkable_jellyfish_merkle::{
    tree_cache::TreeCache, JellyfishMerkleTree, StaleNodeIndex, TreeReader, TreeUpdateBatch,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNode(Node);

impl ValueCodec for StateNode {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.0.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Node::decode(data).map(|n| StateNode(n))
    }
}

pub type StateStorage = CodecStorage<HashValue, StateNode>;

pub struct StateCache {
    root_hashes: Vec<HashValue>,
    change_sets: Vec<TreeUpdateBatch>,
}

impl StateCache {
    fn clear(&mut self) {
        self.root_hashes.clear();
        self.change_sets.clear();
    }

    fn truncate(&mut self, len: usize) {
        self.root_hashes.truncate(len);
        self.change_sets.truncate(len);
    }
}

impl Default for StateCache {
    fn default() -> Self {
        Self {
            root_hashes: vec![],
            change_sets: vec![],
        }
    }
}

pub struct StateDB {
    storage: StateStorage,
    storage_root_hash: RwLock<HashValue>,
    cache: Mutex<StateCache>,
}

impl StateDB {
    /// Construct a new state_db from provided `state_root_hash` with underline `state_storage`
    pub fn new(state_storage: StateStorage, state_root_hash: HashValue) -> Self {
        Self {
            storage: state_storage,
            storage_root_hash: RwLock::new(state_root_hash),
            cache: Mutex::new(StateCache::default()),
        }
    }

    /// get current root hash
    pub fn root_hash(&self) -> HashValue {
        self.cache
            .lock()
            .unwrap()
            .root_hashes
            .last()
            .cloned()
            .unwrap_or(self.storage_root_hash.read().unwrap().clone())
    }

    /// write a new states into local cache, return new root hash
    pub fn put_blob_set(&self, blob_set: Vec<(HashValue, Vec<u8>)>) -> Result<HashValue> {
        let cur_root_hash = self.root_hash();
        let mut cache_guard = self.cache.lock().unwrap();
        let mut cache = cache_guard.deref_mut();
        let reader = CachedTreeReader {
            store: &self.storage,
            cache,
        };
        let tree = JellyfishMerkleTree::new(&reader);
        let blob_set = blob_set
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect::<Vec<_>>();
        let (new_state_root, change_set) =
            tree.put_blob_sets(Some(cur_root_hash), vec![blob_set])?;
        let new_state_root = new_state_root
            .into_iter()
            .next()
            .expect("put blob sets should return a root hash");
        cache.root_hashes.push(new_state_root);
        cache.change_sets.push(change_set);
        Ok(new_state_root)
    }

    /// rollback last write
    pub fn rollback(&self) {
        let mut cache_guard = self.cache.lock().unwrap();
        if let Some(root_hash) = cache_guard.root_hashes.pop() {
            let _ = cache_guard.change_sets.pop();
        }
    }

    /// rollback current state to a history state with the provided `root_hash`
    pub fn rollback_to(&self, root_hash: HashValue) -> Result<()> {
        let mut cache_guard = self.cache.lock().unwrap();
        let mut state_index = None;
        for (i, root) in cache_guard.root_hashes.iter().enumerate() {
            if root == &root_hash {
                state_index = Some(i);
            }
        }

        if let Some(i) = state_index {
            cache_guard.truncate(i + 1);
        } else if self.storage_root_hash.read().unwrap().deref() == &root_hash {
            cache_guard.clear();
        } else {
            bail!("the root_hash is not found in write history");
        }
        Ok(())
    }

    /// get all changes so far based on initial root_hash.
    fn get_change_sets(&self) -> (HashValue, TreeUpdateBatch) {
        let root_hash = self.root_hash();
        let cache_guard = self.cache.lock().unwrap();
        let mut node_batch = BTreeMap::<NodeKey, Node>::new();
        let mut stale_nodes = BTreeSet::new();
        let mut num_new_leaves = 0usize;
        let mut num_stale_leaves = 0usize;
        for cs in cache_guard.change_sets.iter() {
            let mut current_cs_num_stale_leaves = cs.num_stale_leaves;
            for stale_node in cs.stale_node_index_batch.iter() {
                match node_batch.remove(&stale_node.node_key) {
                    None => {
                        stale_nodes.insert(StaleNodeIndex {
                            stale_since_version: root_hash,
                            node_key: stale_node.node_key.clone(),
                        });
                    }
                    Some(n) => {
                        if n.is_leaf() {
                            num_new_leaves -= 1;
                            current_cs_num_stale_leaves -= 1;
                        }
                    }
                }
            }
            num_stale_leaves += current_cs_num_stale_leaves;
            for (nk, n) in cs.node_batch.iter() {
                node_batch.insert(nk.clone(), n.clone());
                if n.is_leaf() {
                    num_new_leaves += 1;
                }
            }
        }

        let cs = TreeUpdateBatch {
            node_batch,
            stale_node_index_batch: stale_nodes,
            num_new_leaves: num_stale_leaves,
            num_stale_leaves,
        };
        (root_hash, cs)
    }

    /// commit the state change into underline storage.
    pub fn commit(&self) -> Result<()> {
        let (root_hash, change_sets) = self.get_change_sets();
        todo!("save the change set to storage");
        // and then advance the storage root hash
        *self.storage_root_hash.write().unwrap() = root_hash;
        self.cache.lock().unwrap().clear();
        Ok(())
    }
}

struct CachedTreeReader<'a> {
    store: &'a StateStorage,
    cache: &'a StateCache,
}

impl<'a> TreeReader for CachedTreeReader<'a> {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node>> {
        for cs in self.cache.change_sets.iter().rev() {
            if let Some(n) = cs.node_batch.get(node_key).cloned() {
                return Ok(Some(n));
            }
        }
        match self.store.get(node_key.clone()) {
            Ok(Some(n)) => Ok(Some(n.0)),
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
