use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::Result;
use atomic_refcell::AtomicRefCell;
use crypto::hash::HashValue;
use forkable_jellyfish_merkle::{node_type::Node, SPARSE_MERKLE_PLACEHOLDER_HASH};
use logger::prelude::*;
use network::NetworkAsyncService;
use parking_lot::RwLock;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_state_tree::{StateNode, StateNodeStore, StateTree};
use std::collections::hash_map::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::ops::DerefMut;
use std::sync::Arc;

struct StateSyncTask {
    root: AtomicRefCell<HashValue>,
    pub state_node_storage: Arc<dyn StateNodeStore>,
    syncing: RwLock<HashSet<HashValue>>,
}

impl StateSyncTask {
    pub fn new(root: HashValue, state_node_storage: Arc<dyn StateNodeStore>) -> StateSyncTask {
        Self {
            root: AtomicRefCell::new(root),
            state_node_storage,
            syncing: RwLock::new(HashSet::new()),
        }
    }

    fn all_son_exist(&self, node_key: &HashValue) -> bool {
        if let Some(current_node) = self.state_node_storage.get(node_key).unwrap() {
            let node = current_node.0;
            match node {
                Node::Leaf(_) => true,
                Node::Internal(n) => {
                    for child in n.all_child() {
                        if !self.all_son_exist(&child) {
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
        self.all_son_exist(&self.root.borrow())
    }

    pub fn reset(&self, root: &HashValue) {
        self.syncing.write().clear();
        std::mem::swap(self.root.borrow_mut().deref_mut(), &mut root.clone());
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

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncEvent {}

struct StateSyncActor {
    sync_task: Arc<StateSyncTask>,
    network: NetworkAsyncService,
    //state_node_storage: Arc<dyn StateNodeStore>,
}

impl StateSyncActor {
    pub fn _launch(
        root: HashValue,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) -> Result<Addr<StateSyncActor>> {
        let state_sync_actor = StateSyncActor::create(move |ctx| Self {
            sync_task: Arc::new(StateSyncTask::new(root, state_node_storage)),
            network,
        });
        Ok(state_sync_actor)
    }

    fn state_nodes(
        &self,
        nodes_hash: Vec<HashValue>,
    ) -> Result<Vec<(HashValue, Option<StateNode>)>> {
        // let mut state_nodes = Vec::new();
        // nodes_hash.iter().for_each(|node_key| {
        //     let node = self.state_node_storage.get(node_key).unwrap();
        //     state_nodes.push((node_key.clone(), node));
        // });
        //
        // Ok(state_nodes)
        unimplemented!()
    }

    fn put_nodes(&self, state_nodes: Vec<(HashValue, StateNode)>) {
        for (node_key, state_node) in state_nodes {
            let _ = self.sync_task.state_node_storage.put(node_key, state_node);
        }
    }
}

impl Actor for StateSyncActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("{:?}", "state sync actor started.");
    }
}

impl Handler<StateSyncEvent> for StateSyncActor {
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, _msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}
