use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::{format_err, Result};
use atomic_refcell::AtomicRefCell;
use consensus::Consensus;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use forkable_jellyfish_merkle::{node_type::Node, SPARSE_MERKLE_PLACEHOLDER_HASH};
use logger::prelude::*;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use parking_lot::RwLock;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_state_tree::{StateNode, StateNodeStore, StateTree};
use std::collections::hash_map::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::ops::DerefMut;
use std::sync::Arc;

struct StateSyncTask<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    root: AtomicRefCell<HashValue>,
    pub state_node_storage: Arc<dyn StateNodeStore>,
    syncing: RwLock<HashSet<HashValue>>,
    network_service: NetworkAsyncService,
    downloader: Arc<Downloader<E, C>>,
}

impl<E, C> StateSyncTask<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(
        root: HashValue,
        state_node_storage: Arc<dyn StateNodeStore>,
        network_service: NetworkAsyncService,
        downloader: Arc<Downloader<E, C>>,
    ) -> StateSyncTask<E, C> {
        Self {
            root: AtomicRefCell::new(root),
            state_node_storage,
            syncing: RwLock::new(HashSet::new()),
            network_service,
            downloader,
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

    async fn sync_state_node(&self, node_key: HashValue) -> Result<StateNode> {
        match self.state_node_storage.get(&node_key)? {
            Some(node) => return Ok(node),
            None => {
                let _ = self.syncing.write().insert(node_key);
                let best_peer = Downloader::best_peer(self.downloader.clone())
                    .await
                    .unwrap();
                let get_state_node_by_node_hash_req = RPCRequest::GetStateNodeByNodeHash(node_key);
                match self
                    .network_service
                    .clone()
                    .send_request(
                        best_peer.id.clone().into(),
                        get_state_node_by_node_hash_req.clone(),
                        do_duration(DELAY_TIME),
                    )
                    .await
                    .unwrap()
                {
                    RPCResponse::GetStateNodeByNodeHash(state_node) => {
                        debug!("get_state_node_by_node_hash_resp:{:?}", state_node);
                        let _ = self.syncing.write().remove(&node_key);
                        return Ok(state_node);
                    }
                    _ => return Err(format_err!("{:?}", "error RPCResponse type.")),
                };
            }
        };
    }

    async fn sync_state(&self, node_key: &HashValue) {
        let node = match self.state_node_storage.get(node_key).unwrap() {
            Some(node) => node,
            None => self.sync_state_node(node_key.clone()).await.unwrap(),
        };

        match node.inner() {
            Node::Leaf(_) => {}
            Node::Internal(n) => {
                for child in n.all_child() {
                    self.sync_state(&child);
                }
            }
            _ => {
                warn!("node {:?} is null.", node_key);
            }
        }
    }

    async fn state_sync(&self) {
        let root = &*self.root.borrow();
        self.sync_state(root).await;
        // Arbiter::spawn(move {
        //
        // })
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

    // let state_node_cache = StateSyncTask::new(new_root_hash);
    //
    // for (k, v) in store.all_nodes() {
    //     let _ = state_node_cache.put(k, v);
    // }
    // assert_eq!(state_node_cache.is_complete(), true);
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

    // let state_node_cache = StateSyncTask::new(new_root_hash);
    // assert_eq!(state_node_cache.is_complete(), false);
}

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncEvent {
    root: HashValue,
}

struct StateSyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    sync_task: Arc<StateSyncTask<E, C>>,
}

impl<E, C> StateSyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn _launch(
        root: HashValue,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        downloader: Arc<Downloader<E, C>>,
    ) -> Result<Addr<StateSyncActor<E, C>>> {
        let state_sync_actor = StateSyncActor::create(move |ctx| Self {
            sync_task: Arc::new(StateSyncTask::new(
                root,
                state_node_storage,
                network,
                downloader,
            )),
        });
        Ok(state_sync_actor)
    }

    fn put_nodes(&self, state_nodes: Vec<(HashValue, StateNode)>) {
        for (node_key, state_node) in state_nodes {
            let _ = self.sync_task.state_node_storage.put(node_key, state_node);
        }
    }
}

impl<E, C> Actor for StateSyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("{:?}", "state sync actor started.");
    }
}

impl<E, C> Handler<StateSyncEvent> for StateSyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, _msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}
