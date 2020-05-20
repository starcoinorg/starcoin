use crate::helper::{get_accumulator_node_by_node_hash, get_state_node_by_node_hash};
use crate::sync_metrics::{LABEL_ACCUMULATOR, LABEL_STATE, SYNC_METRICS};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use crypto::hash::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use forkable_jellyfish_merkle::SPARSE_MERKLE_PLACEHOLDER_HASH;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use parking_lot::Mutex;
use starcoin_accumulator::node::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_accumulator::AccumulatorNode;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
use starcoin_sync_api::{StateSyncReset, SyncMetadata};
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use types::{account_state::AccountState, peer_info::PeerId};

struct Roots {
    state: HashValue,
    txn_accumulator: HashValue,
    block_accumulator: HashValue,
}

impl Roots {
    pub fn new(state: HashValue, txn_accumulator: HashValue, block_accumulator: HashValue) -> Self {
        Roots {
            state,
            txn_accumulator,
            block_accumulator,
        }
    }

    fn state_root(&self) -> &HashValue {
        &self.state
    }

    fn txn_accumulator_root(&self) -> &HashValue {
        &self.txn_accumulator
    }

    fn block_accumulator_root(&self) -> &HashValue {
        &self.block_accumulator
    }
}

async fn sync_accumulator_node(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor>,
) {
    let accumulator_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_ACCUMULATOR])
        .start_timer();
    let accumulator_node = match get_accumulator_node_by_node_hash(
        &network_service,
        peer_id.clone(),
        node_key,
    )
    .await
    {
        Ok(accumulator_node) => {
            debug!(
                "get_accumulator_node_by_node_hash_resp:{:?}",
                accumulator_node
            );
            if node_key == accumulator_node.hash() {
                SYNC_METRICS
                    .sync_succ_count
                    .with_label_values(&[LABEL_ACCUMULATOR])
                    .inc();
                Some(accumulator_node)
            } else {
                SYNC_METRICS
                    .sync_verify_fail_count
                    .with_label_values(&[LABEL_ACCUMULATOR])
                    .inc();
                warn!(
                    "accumulator node hash not match {} :{:?}",
                    node_key,
                    accumulator_node.hash()
                );
                None
            }
        }
        Err(e) => {
            SYNC_METRICS
                .sync_fail_count
                .with_label_values(&[LABEL_ACCUMULATOR])
                .inc();
            error!("error: {:?}", e);
            None
        }
    };
    accumulator_timer.observe_duration();

    if let Err(err) = address.try_send(StateSyncTaskEvent::new_accumulator(
        peer_id,
        node_key,
        accumulator_node,
    )) {
        warn!("err:{:?}", err);
    };
}

async fn sync_state_node(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor>,
) {
    debug!("sync_state_node : {:?}", node_key);

    let state_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_STATE])
        .start_timer();
    let state_node =
        match get_state_node_by_node_hash(&network_service, peer_id.clone(), node_key).await {
            Ok(state_node) => {
                debug!("get_state_node_by_node_hash_resp:{:?}", state_node);
                if node_key == state_node.0.hash() {
                    SYNC_METRICS
                        .sync_succ_count
                        .with_label_values(&[LABEL_STATE])
                        .inc();
                    Some(state_node)
                } else {
                    SYNC_METRICS
                        .sync_verify_fail_count
                        .with_label_values(&[LABEL_STATE])
                        .inc();
                    warn!(
                        "state node hash not match {} :{:?}",
                        node_key,
                        state_node.0.hash()
                    );
                    None
                }
            }
            Err(e) => {
                SYNC_METRICS
                    .sync_fail_count
                    .with_label_values(&[LABEL_STATE])
                    .inc();
                error!("error: {:?}", e);
                None
            }
        };
    state_timer.observe_duration();

    if let Err(err) = address.try_send(StateSyncTaskEvent::new_state(peer_id, node_key, state_node))
    {
        warn!("err:{:?}", err);
    };
}

#[derive(Clone)]
pub struct StateSyncTaskRef {
    address: Addr<StateSyncTaskActor>,
}

#[async_trait::async_trait]
impl StateSyncReset for StateSyncTaskRef {
    async fn reset(
        &self,
        state_root: HashValue,
        txn_accumulator_root: HashValue,
        block_accumulator_root: HashValue,
    ) {
        if let Err(e) = self
            .address
            .send(StateSyncEvent::RESET(RestRoots {
                state_root,
                txn_accumulator_root,
                block_accumulator_root,
            }))
            .await
        {
            warn!("err : {:?}", e);
        }
    }

    async fn act(&self) {
        if let Err(e) = self.address.send(StateSyncEvent::ACT {}).await {
            warn!("err : {:?}", e);
        }
    }
}

#[derive(Debug, PartialEq)]
enum TaskType {
    STATE,
    ACCUMULATOR,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncTaskEvent {
    peer_id: PeerId,
    node_key: HashValue,
    state_node: Option<StateNode>,
    accumulator_node: Option<AccumulatorNode>,
    task_type: TaskType,
}

impl StateSyncTaskEvent {
    pub fn new_state(peer_id: PeerId, node_key: HashValue, state_node: Option<StateNode>) -> Self {
        StateSyncTaskEvent {
            peer_id,
            node_key,
            state_node,
            accumulator_node: None,
            task_type: TaskType::STATE,
        }
    }

    pub fn new_accumulator(
        peer_id: PeerId,
        node_key: HashValue,
        accumulator_node: Option<AccumulatorNode>,
    ) -> Self {
        StateSyncTaskEvent {
            peer_id,
            node_key,
            state_node: None,
            accumulator_node,
            task_type: TaskType::ACCUMULATOR,
        }
    }

    fn is_state(&self) -> bool {
        self.task_type == TaskType::STATE
    }
}

pub struct StateSyncTaskActor {
    self_peer_id: PeerId,
    roots: Roots,
    storage: Arc<dyn Store>,
    network_service: NetworkAsyncService,
    sync_metadata: SyncMetadata,
    state_sync_task: Arc<Mutex<SyncTask<(HashValue, bool)>>>,
    accumulator_sync_task: Arc<Mutex<SyncTask<HashValue>>>,
    state_sync_count: AtomicU64,
    accumulator_sync_count: AtomicU64,
}

pub struct SyncTask<T> {
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

    fn task_len(&self) -> (usize, usize) {
        (self.wait_2_sync.len(), self.syncing_nodes.len())
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

    pub fn get(&self, peer_id: &PeerId) -> Option<&T> {
        self.syncing_nodes.get(peer_id)
    }

    pub fn remove(&mut self, peer_id: &PeerId) -> Option<T> {
        self.syncing_nodes.remove(peer_id)
    }
}

impl StateSyncTaskActor {
    pub fn launch(
        self_peer_id: PeerId,
        root: (HashValue, HashValue, HashValue),
        storage: Arc<dyn Store>,
        network_service: NetworkAsyncService,
        sync_metadata: SyncMetadata,
    ) -> StateSyncTaskRef {
        let roots = Roots::new(root.0, root.1, root.2);
        let mut state_sync_task = SyncTask::new();
        state_sync_task.push_back((*roots.state_root(), true));
        let mut accumulator_sync_task = SyncTask::new();
        accumulator_sync_task.push_back(*roots.txn_accumulator_root());
        accumulator_sync_task.push_back(*roots.block_accumulator_root());
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            self_peer_id,
            roots,
            storage,
            network_service,
            sync_metadata,
            state_sync_task: Arc::new(Mutex::new(state_sync_task)),
            accumulator_sync_task: Arc::new(Mutex::new(accumulator_sync_task)),
            state_sync_count: AtomicU64::new(0),
            accumulator_sync_count: AtomicU64::new(0),
        });
        StateSyncTaskRef { address }
    }

    fn sync_end(&self) -> bool {
        info!(
            "state_sync_task len : {:?}, accumulator_sync_task len : {:?},\
         save state nodes : {} , save accumulator nodes : {}",
            self.state_sync_task.lock().task_len(),
            self.accumulator_sync_task.lock().task_len(),
            self.state_sync_count.load(Ordering::Relaxed),
            self.accumulator_sync_count.load(Ordering::Relaxed),
        );
        self.state_sync_task.lock().is_empty() && self.accumulator_sync_task.lock().is_empty()
    }

    fn exe_state_sync_task(&mut self, address: Addr<StateSyncTaskActor>) {
        let mut lock = self.state_sync_task.lock();
        let value = lock.pop_front();
        if let Some((node_key, is_global)) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_STATE])
                .inc();
            if let Some(state_node) = self.storage.get(&node_key).unwrap() {
                debug!("find state_node {:?} in db.", node_key);
                lock.insert(self.self_peer_id.clone(), (node_key, is_global));
                if let Err(err) = address.try_send(StateSyncTaskEvent::new_state(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(state_node),
                )) {
                    warn!("err:{:?}", err);
                };
            } else {
                let network_service = self.network_service.clone();
                let best_peer_info =
                    block_on(async move { network_service.best_peer().await.unwrap() });
                debug!(
                    "sync state_node {:?} from peer {:?}.",
                    node_key, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id != best_peer.get_peer_id() {
                        let network_service = self.network_service.clone();
                        lock.insert(best_peer.get_peer_id(), (node_key, is_global));
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
                    self.sync_metadata.update_failed(true);
                }
            }
        }
    }

    fn handle_state_sync(&mut self, task_event: StateSyncTaskEvent) {
        let mut lock = self.state_sync_task.lock();
        if let Some((state_node_hash, is_global)) = lock.get(&task_event.peer_id) {
            let is_global = *is_global;
            //1. push back
            let current_node_key = task_event.node_key;
            if state_node_hash != &current_node_key {
                warn!(
                    "hash not match {:} : {:?}",
                    state_node_hash, current_node_key
                );
                return;
            }
            let _ = lock.remove(&task_event.peer_id);
            if let Some(state_node) = task_event.state_node {
                if let Err(e) = self.storage.put(current_node_key, state_node.clone()) {
                    error!("error : {:?}", e);
                    lock.push_back((current_node_key, is_global));
                } else {
                    debug!("receive state_node: {:?}", state_node.0.hash());
                    self.state_sync_count.fetch_add(1, Ordering::Relaxed);
                    match state_node.inner() {
                        Node::Leaf(leaf) => {
                            if !is_global {
                                return;
                            }
                            match AccountState::try_from(leaf.blob().as_ref()) {
                                Err(e) => {
                                    error!("error : {:?}", e);
                                }
                                Ok(account_state) => {
                                    account_state.storage_roots().iter().for_each(|key| {
                                        if let Some(hash) = key {
                                            if *hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                                                lock.push_back((*hash, false));
                                            }
                                        }
                                    });
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
        } else {
            warn!("discard state event : {:?}", task_event);
        }
    }

    fn exe_accumulator_sync_task(&mut self, address: Addr<StateSyncTaskActor>) {
        let mut lock = self.accumulator_sync_task.lock();
        let value = lock.pop_front();
        if let Some(node_key) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_ACCUMULATOR])
                .inc();
            if let Some(accumulator_node) = self.storage.get_node(node_key).unwrap() {
                debug!("find accumulator_node {:?} in db.", node_key);
                lock.insert(self.self_peer_id.clone(), node_key);
                if let Err(err) = address.try_send(StateSyncTaskEvent::new_accumulator(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(accumulator_node),
                )) {
                    warn!("err:{:?}", err);
                };
            } else {
                let network_service = self.network_service.clone();
                let best_peer_info =
                    block_on(async move { network_service.best_peer().await.unwrap() });
                debug!(
                    "sync accumulator_node {:?} from peer {:?}.",
                    node_key, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id != best_peer.get_peer_id() {
                        let network_service = self.network_service.clone();
                        lock.insert(best_peer.get_peer_id(), node_key);
                        Arbiter::spawn(async move {
                            sync_accumulator_node(
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
                    self.sync_metadata.update_failed(true);
                }
            }
        }
    }

    fn handle_accumulator_sync(&mut self, task_event: StateSyncTaskEvent) {
        let mut lock = self.accumulator_sync_task.lock();
        if let Some(accumulator_node_hash) = lock.get(&task_event.peer_id) {
            //1. push back
            let current_node_key = task_event.node_key;
            if accumulator_node_hash != &current_node_key {
                warn!(
                    "hash not match {:} : {:?}",
                    accumulator_node_hash, current_node_key
                );
                return;
            }
            let _ = lock.remove(&task_event.peer_id);
            if let Some(accumulator_node) = task_event.accumulator_node {
                if let Err(e) = self.storage.save_node(accumulator_node.clone()) {
                    error!("error : {:?}", e);
                    lock.push_back(current_node_key);
                } else {
                    debug!("receive accumulator_node: {:?}", accumulator_node);
                    self.accumulator_sync_count.fetch_add(1, Ordering::Relaxed);
                    match accumulator_node {
                        AccumulatorNode::Leaf(_leaf) => {}
                        AccumulatorNode::Internal(n) => {
                            if n.left() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                lock.push_back(n.left());
                            }
                            if n.right() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                lock.push_back(n.right());
                            }
                        }
                        _ => {
                            warn!("node {:?} is null.", current_node_key);
                        }
                    }
                }
            } else {
                lock.push_back(current_node_key);
            }
        } else {
            warn!("discard state event : {:?}", task_event);
        }
    }

    pub fn reset(
        &mut self,
        state_root: &HashValue,
        txn_accumulator_root: &HashValue,
        block_accumulator_root: &HashValue,
        address: Addr<StateSyncTaskActor>,
    ) {
        info!("reset state sync task.");
        self.roots = Roots::new(*state_root, *txn_accumulator_root, *block_accumulator_root);
        let mut state_lock = self.state_sync_task.lock();
        let old_state_is_empty = state_lock.is_empty();
        state_lock.clear();
        state_lock.push_back((*self.roots.state_root(), true));
        drop(state_lock);
        let mut accumulator_lock = self.accumulator_sync_task.lock();
        let old_accumulator_is_empty = accumulator_lock.is_empty();
        accumulator_lock.clear();
        accumulator_lock.push_back(*self.roots.txn_accumulator_root());
        accumulator_lock.push_back(*self.roots.block_accumulator_root());
        drop(accumulator_lock);
        self.state_sync_count = AtomicU64::new(0);
        self.accumulator_sync_count = AtomicU64::new(0);
        if self.sync_metadata.is_failed() {
            self.activation_task(address);
        } else if old_state_is_empty {
            self.exe_state_sync_task(address.clone());
        } else if old_accumulator_is_empty {
            self.exe_accumulator_sync_task(address.clone());
        }
    }

    fn activation_task(&mut self, address: Addr<StateSyncTaskActor>) {
        info!("activation state sync task.");
        if self.sync_metadata.is_failed() {
            self.sync_metadata.update_failed(false);
            self.exe_state_sync_task(address.clone());
            self.exe_accumulator_sync_task(address);
        }
    }
}

impl Actor for StateSyncTaskActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor started.");
        self.exe_state_sync_task(ctx.address());
        self.exe_accumulator_sync_task(ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor stopped.");
    }
}

impl Handler<StateSyncTaskEvent> for StateSyncTaskActor {
    type Result = Result<()>;

    fn handle(&mut self, task_event: StateSyncTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let state_or_accumulator = task_event.is_state();
        if state_or_accumulator {
            self.handle_state_sync(task_event);
        } else {
            self.handle_accumulator_sync(task_event);
        }

        if self.sync_end() {
            info!("state sync end");
            if let Err(e) = self.sync_metadata.state_sync_done() {
                warn!("err:{:?}", e);
            } else {
                info!("sync_done : {:?}", self.sync_metadata.get_pivot());

                ctx.stop();
            }
        } else if state_or_accumulator {
            self.exe_state_sync_task(ctx.address());
        } else {
            self.exe_accumulator_sync_task(ctx.address());
        }
        Ok(())
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
enum StateSyncEvent {
    RESET(RestRoots),
    ACT,
}

#[derive(Debug, Clone)]
struct RestRoots {
    state_root: HashValue,
    txn_accumulator_root: HashValue,
    block_accumulator_root: HashValue,
}

impl Handler<StateSyncEvent> for StateSyncTaskActor {
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StateSyncEvent::ACT => self.activation_task(ctx.address()),
            StateSyncEvent::RESET(roots) => {
                self.reset(
                    &roots.state_root,
                    &roots.txn_accumulator_root,
                    &roots.block_accumulator_root,
                    ctx.address(),
                );
            }
        }
        Ok(())
    }
}
