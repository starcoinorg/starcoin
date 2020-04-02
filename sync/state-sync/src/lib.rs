use anyhow::Result;
use forkable_jellyfish_merkle::{node_type::Node, SPARSE_MERKLE_PLACEHOLDER_HASH};
use logger::prelude::*;
use parking_lot::RwLock;
use starcoin_crypto::hash::HashValue;
use starcoin_network::NetworkAsyncService;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_state_tree::{StateNode, StateNodeStore, StateTree};
use std::collections::hash_map::HashMap;
use std::collections::BTreeMap;
use std::sync::Arc;

struct StateNodeCache {
    root: HashValue,
    state_nodes: RwLock<HashMap<HashValue, StateNode>>,
}

impl StateNodeCache {
    pub fn new(root: HashValue) -> StateNodeCache {
        Self {
            root,
            state_nodes: RwLock::new(HashMap::new()),
        }
    }

    fn all_son_exist(&self, node_key: &HashValue) -> bool {
        if let Some(current_node) = self.get(node_key).unwrap() {
            let node = current_node.0;
            match node {
                Node::Leaf(_) => true,
                Node::Internal(n) => {
                    for child in n.all_child() {
                        if !self.state_nodes.read().contains_key(&child) {
                            warn!("node {:?} child {:?} not exist.", node_key, child);
                            return false;
                        }
                    }
                    true
                }
                _ => {
                    warn!("node {:?} is null.", node_key);
                    false
                }
            }
        } else {
            warn!("node {:?} not exist.", node_key);
            false
        }
    }

    pub fn is_complete(&self) -> bool {
        self.all_son_exist(&self.root)
    }
}

impl StateNodeStore for StateNodeCache {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        if let Some(node) = self.state_nodes.read().get(hash) {
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_nodes.write().insert(key, node);
        Ok(())
    }

    fn write_nodes(&self, _nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        unimplemented!()
    }
}

#[test]
fn test_state_node_cache_complete() {
    use starcoin_state_tree::update_nibble;

    let s = Arc::new(MockStateNodeStore::new());
    let store = Arc::clone(&s);
    let state = StateTree::new(s, None);
    assert_eq!(state.root_hash(), *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let hash_value = HashValue::random();

    let account1 = update_nibble(&hash_value, 0, 1);
    let account1 = update_nibble(&account1, 2, 2);
    state.put(account1, vec![0, 0, 0]);

    assert_eq!(state.get(&account1).unwrap(), Some(vec![0, 0, 0]));
    assert_eq!(state.get(&update_nibble(&hash_value, 0, 8)).unwrap(), None);

    let new_root_hash = state.commit().unwrap();
    state.flush().unwrap();
    assert_eq!(state.root_hash(), new_root_hash);

    let state_node_cache = StateNodeCache::new(new_root_hash);

    for (k, v) in store.all_nodes() {
        let _ = state_node_cache.put(k, v);
    }
    assert_eq!(state_node_cache.is_complete(), true);
}

#[test]
fn test_state_node_cache_not_complete() {
    use starcoin_state_tree::update_nibble;

    let s = MockStateNodeStore::new();
    let state = StateTree::new(Arc::new(s), None);
    assert_eq!(state.root_hash(), *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let hash_value = HashValue::random();

    let account1 = update_nibble(&hash_value, 0, 1);
    let account1 = update_nibble(&account1, 2, 2);
    state.put(account1, vec![0, 0, 0]);

    assert_eq!(state.get(&account1).unwrap(), Some(vec![0, 0, 0]));
    assert_eq!(state.get(&update_nibble(&hash_value, 0, 8)).unwrap(), None);

    let new_root_hash = state.commit().unwrap();
    assert_eq!(state.root_hash(), new_root_hash);

    let state_node_cache = StateNodeCache::new(new_root_hash);
    assert_eq!(state_node_cache.is_complete(), false);
}

struct StateSyncActor {
    cache: Arc<StateNodeCache>,
    network: NetworkAsyncService,
}

impl StateSyncActor {
    pub fn _launch(root: HashValue, network: NetworkAsyncService) -> StateSyncActor {
        Self {
            cache: Arc::new(StateNodeCache::new(root)),
            network,
        }
    }
}
