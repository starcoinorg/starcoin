use crate::helper::get_state_node_by_node_hash;
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use crypto::hash::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use forkable_jellyfish_merkle::SPARSE_MERKLE_PLACEHOLDER_HASH;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use parking_lot::Mutex;
use starcoin_state_tree::{StateNode, StateNodeStore};
use starcoin_sync_api::{StateSyncReset, SyncMetadata};
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::sync::Arc;
use types::{account_state::AccountState, peer_info::PeerId};

struct Roots {
    state: HashValue,
    accumulator: HashValue,
}

impl Roots {
    fn new(state: HashValue, accumulator: HashValue) -> Self {
        Roots { state, accumulator }
    }

    fn state_root(&self) -> &HashValue {
        &self.state
    }

    fn accumulator_root(&self) -> &HashValue {
        &self.accumulator
    }
}

async fn sync_state_node(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor>,
) {
    debug!("sync_state_node : {:?}", node_key);

    let state_node =
        match get_state_node_by_node_hash(&network_service, peer_id.clone(), node_key).await {
            Ok(state_node) => {
                debug!("get_state_node_by_node_hash_resp:{:?}", state_node);
                if node_key == state_node.0.hash() {
                    Some(state_node)
                } else {
                    warn!(
                        "state node hash not match {} :{:?}",
                        node_key,
                        state_node.0.hash()
                    );
                    None
                }
            }
            Err(e) => {
                error!("error: {:?}", e);
                None
            }
        };

    if let Err(err) = address.try_send(StateSyncTaskEvent {
        peer_id,
        node_key,
        state_node,
    }) {
        warn!("err:{:?}", err);
    };
}

#[derive(Clone)]
pub struct StateSyncTaskRef {
    address: Addr<StateSyncTaskActor>,
}

#[async_trait::async_trait]
impl StateSyncReset for StateSyncTaskRef {
    async fn reset(&self, state_root: HashValue, accumulator_root: HashValue) {
        if let Err(e) = self
            .address
            .send(StateSyncEvent {
                state_root,
                accumulator_root,
            })
            .await
        {
            warn!("err : {:?}", e);
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncTaskEvent {
    peer_id: PeerId,
    node_key: HashValue,
    state_node: Option<StateNode>,
}

pub struct StateSyncTaskActor {
    self_peer_id: PeerId,
    roots: Roots,
    state_node_storage: Arc<dyn StateNodeStore>,
    network_service: NetworkAsyncService,
    sync_metadata: SyncMetadata,
    state_sync_task: Arc<Mutex<SyncTask<(HashValue, bool)>>>,
    accumulator_sync_task: Arc<Mutex<SyncTask<HashValue>>>,
}

struct SyncTask<T> {
    wait_2_sync: VecDeque<T>,
    syncing_nodes: HashMap<PeerId, T>,
}

impl<T> SyncTask<T> {
    fn new() -> Self {
        Self {
            wait_2_sync: VecDeque::new(),
            syncing_nodes: HashMap::new(),
        }
    }

    fn is_empty(&mut self) -> bool {
        self.wait_2_sync.is_empty() && self.syncing_nodes.is_empty()
    }

    pub fn push_back(&mut self, value: T) {
        self.wait_2_sync.push_back(value)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.wait_2_sync.pop_front()
    }

    pub fn clear(&mut self) {
        self.wait_2_sync.clear();
        self.syncing_nodes.clear();
    }

    pub fn insert(&mut self, peer_id: PeerId, value: T) -> Option<T> {
        self.syncing_nodes.insert(peer_id, value)
    }

    pub fn get(&mut self, peer_id: &PeerId) -> Option<&T> {
        self.syncing_nodes.get(peer_id)
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> Option<T> {
        self.syncing_nodes.remove(peer_id)
    }
}

impl StateSyncTaskActor {
    pub fn launch(
        self_peer_id: PeerId,
        root: (HashValue, HashValue),
        state_node_storage: Arc<dyn StateNodeStore>,
        network_service: NetworkAsyncService,
        sync_metadata: SyncMetadata,
    ) -> StateSyncTaskRef {
        let roots = Roots::new(root.0, root.1);
        let mut state_sync_task = SyncTask::new();
        state_sync_task.push_back((roots.state_root().clone(), true));
        let mut accumulator_sync_task = SyncTask::new();
        accumulator_sync_task.push_back(roots.accumulator_root().clone());
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            self_peer_id,
            roots,
            state_node_storage,
            network_service,
            sync_metadata,
            state_sync_task: Arc::new(Mutex::new(state_sync_task)),
            accumulator_sync_task: Arc::new(Mutex::new(accumulator_sync_task)),
        });
        StateSyncTaskRef { address }
    }

    fn sync_end(&self) -> bool {
        self.state_sync_task.lock().is_empty() && self.accumulator_sync_task.lock().is_empty()
    }

    fn exe_state_sync_task(&mut self, address: Addr<StateSyncTaskActor>) {
        let mut lock = self.state_sync_task.lock();
        let (node_key, is_global) = lock.pop_front().unwrap();
        if let Some(state_node) = self.state_node_storage.get(&node_key).unwrap() {
            debug!("find state_node {:?} in db.", node_key);
            lock.insert(self.self_peer_id.clone(), (node_key.clone(), is_global));
            if let Err(err) = address.try_send(StateSyncTaskEvent {
                peer_id: self.self_peer_id.clone(),
                node_key,
                state_node: Some(state_node),
            }) {
                warn!("err:{:?}", err);
            };
        } else {
            let network_service = self.network_service.clone();
            let best_peer_info = block_on(async move {
                let peer_info = network_service.best_peer().await.unwrap();
                peer_info
            });
            debug!(
                "sync state_node {:?} from peer {:?}.",
                node_key, best_peer_info
            );
            if let Some(best_peer) = best_peer_info {
                if self.self_peer_id != best_peer.get_peer_id() {
                    let network_service = self.network_service.clone();
                    lock.insert(best_peer.get_peer_id(), (node_key.clone(), is_global));
                    Arbiter::spawn(async move {
                        sync_state_node(
                            node_key,
                            best_peer.get_peer_id(),
                            network_service,
                            address,
                        )
                        .await;
                    });
                }
            } else {
                warn!("{:?}", "best peer is none.");
            }
        }
    }

    pub fn reset(&mut self, state_root: &HashValue, accumulator_root: &HashValue) {
        info!("reset state sync task.");
        let mut lock = self.state_sync_task.lock();
        lock.clear();
        self.roots = Roots::new(state_root.clone(), accumulator_root.clone());
        lock.push_back((self.roots.state_root().clone(), true));
    }
}

impl Actor for StateSyncTaskActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor started.");
        self.exe_state_sync_task(ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor stopped.");
    }
}

impl Handler<StateSyncTaskEvent> for StateSyncTaskActor {
    type Result = Result<()>;

    fn handle(&mut self, task_event: StateSyncTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let mut lock = self.state_sync_task.lock();
        if let Some((state_node_hash, is_global)) = lock.get(&task_event.peer_id) {
            let is_global = is_global.clone();
            //1. push back
            let current_node_key = task_event.node_key;
            if state_node_hash == &current_node_key {
                let _ = lock.remove(&task_event.peer_id);
                if let Some(state_node) = task_event.state_node {
                    if let Err(e) = self
                        .state_node_storage
                        .put(current_node_key, state_node.clone())
                    {
                        error!("error : {:?}", e);
                        lock.push_back((current_node_key, is_global));
                    } else {
                        debug!("receive state_node: {:?}", state_node);
                        match state_node.inner() {
                            Node::Leaf(leaf) => {
                                if is_global {
                                    match AccountState::try_from(leaf.blob().as_ref()) {
                                        Err(e) => {
                                            error!("error : {:?}", e);
                                        }
                                        Ok(account_state) => {
                                            account_state.storage_roots().iter().for_each(|key| {
                                                if key.is_some() {
                                                    let hash = key.unwrap().clone();
                                                    if hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                                                        lock.push_back((hash, false));
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                            Node::Internal(n) => {
                                for child in n.all_child() {
                                    lock.push_back((child, is_global));
                                }
                            }
                            _ => {
                                warn!("node {:?} is null.", current_node_key);
                            }
                        }
                    }
                } else {
                    lock.push_back((current_node_key, is_global));
                }

                //2. exe_task
                drop(lock);
                if self.sync_end() {
                    if let Err(e) = self.sync_metadata.state_sync_done() {
                        warn!("err:{:?}", e);
                    } else {
                        info!("sync_done : {:?}", self.sync_metadata.get_pivot());

                        ctx.stop();
                    }
                } else {
                    self.exe_state_sync_task(ctx.address());
                }
            } else {
                warn!(
                    "hash not match {:} : {:?}",
                    state_node_hash, current_node_key
                );
            }
        } else {
            warn!("discard state event : {:?}", task_event);
        }

        Ok(())
    }
}

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncEvent {
    state_root: HashValue,
    accumulator_root: HashValue,
}

impl Handler<StateSyncEvent> for StateSyncTaskActor {
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.reset(&msg.state_root, &msg.accumulator_root);
        Ok(())
    }
}
