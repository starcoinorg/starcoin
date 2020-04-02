use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::{format_err, Result};
use atomic_refcell::AtomicRefCell;
use consensus::Consensus;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use forkable_jellyfish_merkle::{node_type::Node, SPARSE_MERKLE_PLACEHOLDER_HASH};
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use logger::prelude::*;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use parking_lot::RwLock;
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_state_tree::{StateNode, StateNodeStore};
use std::collections::HashSet;
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

    fn _all_son_exist(&self, node_key: &HashValue) -> bool {
        if let Some(current_node) = self.state_node_storage.get(node_key).unwrap() {
            let node = current_node.0;
            match node {
                Node::Leaf(_) => true,
                Node::Internal(n) => {
                    for child in n.all_child() {
                        if !self._all_son_exist(&child) {
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

    pub fn _is_complete(&self) -> bool {
        self._all_son_exist(&self.root.borrow())
    }

    pub fn _reset(&self, root: &HashValue) {
        self.syncing.write().clear();
        std::mem::swap(self.root.borrow_mut().deref_mut(), &mut root.clone());
    }

    fn sync_state_node(
        sync_task: Arc<StateSyncTask<E, C>>,
        node_key: HashValue,
        sender: UnboundedSender<Result<StateNode>>,
    ) {
        Arbiter::spawn(async move {
            let state_node = match sync_task.state_node_storage.get(&node_key).unwrap() {
                Some(node) => Ok(node),
                None => {
                    let _ = sync_task.syncing.write().insert(node_key);
                    let best_peer = Downloader::best_peer(sync_task.downloader.clone())
                        .await
                        .unwrap();
                    let get_state_node_by_node_hash_req =
                        RPCRequest::GetStateNodeByNodeHash(node_key);
                    if let RPCResponse::GetStateNodeByNodeHash(state_node) = sync_task
                        .network_service
                        .clone()
                        .send_request(
                            best_peer.get_peer_id().clone().into(),
                            get_state_node_by_node_hash_req.clone(),
                            do_duration(DELAY_TIME),
                        )
                        .await
                        .unwrap()
                    {
                        debug!("get_state_node_by_node_hash_resp:{:?}", state_node);
                        let _ = sync_task.syncing.write().remove(&node_key);
                        Ok(state_node)
                    } else {
                        Err(format_err!("{:?}", "error RPCResponse type."))
                    }
                }
            };

            let _ = sender.clone().send(state_node);
            ()
        });
    }

    fn sync_state(sync_task: Arc<StateSyncTask<E, C>>, node_key: &HashValue) {
        let node = match sync_task.clone().state_node_storage.get(node_key).unwrap() {
            Some(node) => node,
            None => {
                let (sender, mut receiver) = unbounded();
                Self::sync_state_node(sync_task.clone(), node_key.clone(), sender);

                async_std::task::block_on(async move {
                    let mut tmp = None;
                    loop {
                        ::futures::select! {
                            result = receiver.select_next_some() => {
                                match result {
                                    Ok(sync_state) => {
                                        tmp = Some(sync_state);
                                        break;
                                    },
                                    Err(err) => {
                                        warn!("error: {:?}", err);
                                    },
                                }
                            },
                            complete => {
                               break;
                            }
                        }
                    }
                    tmp.unwrap()
                })
            }
        };

        match node.inner() {
            Node::Leaf(_) => return,
            Node::Internal(n) => {
                for child in n.all_child() {
                    Self::sync_state(sync_task.clone(), &child);
                }
            }
            _ => {
                warn!("node {:?} is null.", node_key);
                return;
            }
        }
    }

    fn state_sync(sync_task: Arc<StateSyncTask<E, C>>) {
        let root = &*sync_task.root.borrow();
        Self::sync_state(sync_task.clone(), root);
    }
}

#[test]
fn test_state_node_cache_complete() {
    use starcoin_state_tree::update_nibble;
    use starcoin_state_tree::StateTree;

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
    use starcoin_state_tree::StateTree;

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

pub struct StateSyncActor<E, C>
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
    pub fn launch(
        root: HashValue,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        downloader: Arc<Downloader<E, C>>,
    ) -> Result<Addr<StateSyncActor<E, C>>> {
        let state_sync_actor = StateSyncActor::create(move |_ctx| Self {
            sync_task: Arc::new(StateSyncTask::new(
                root,
                state_node_storage,
                network,
                downloader,
            )),
        });
        Ok(state_sync_actor)
    }
}

impl<E, C> Actor for StateSyncActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let sync_task = self.sync_task.clone();
        Arbiter::spawn(async move {
            StateSyncTask::state_sync(sync_task);
        });
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
