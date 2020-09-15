use crate::block_sync::BlockSyncTaskRef;
use crate::download::DownloadActor;
use crate::helper::{
    get_accumulator_node_by_node_hash, get_state_node_by_node_hash, get_txn_infos,
};
use crate::sync_event_handle::SendSyncEventHandler;
use crate::sync_metrics::{LABEL_ACCUMULATOR, LABEL_STATE, LABEL_TXN_INFO, SYNC_METRICS};
use crate::sync_task::{
    SyncTaskAction, SyncTaskRequest, SyncTaskResponse, SyncTaskState, SyncTaskType,
};
use crate::StateSyncReset;
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use crypto::hash::{ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH};
use crypto::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use futures::executor::block_on;
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_network_rpc_api::gen_client::NetworkRpcClient;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use types::{
    account_state::AccountState,
    peer_info::{PeerId, PeerInfo},
    transaction::TransactionInfo,
};

#[cfg(test)]
mod test_state_sync;

#[derive(Default, Debug, Message)]
#[rtype(result = "()")]
struct TxnInfoEvent(Option<HashValue>);

struct Roots {
    state: HashValue,
    block_accumulator: HashValue,
    pivot_id: HashValue,
}

impl Roots {
    pub fn new(state: HashValue, block_accumulator: HashValue, pivot_id: HashValue) -> Self {
        Roots {
            state,
            block_accumulator,
            pivot_id,
        }
    }

    fn state_root(&self) -> &HashValue {
        &self.state
    }

    fn block_accumulator_root(&self) -> &HashValue {
        &self.block_accumulator
    }

    fn pivot_id(&self) -> &HashValue {
        &self.pivot_id
    }
}

async fn sync_accumulator_node<N>(
    node_key: HashValue,
    peer_id: PeerId,
    rpc_client: NetworkRpcClient<N>,
    state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
) where
    N: NetworkService + 'static,
{
    let accumulator_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_ACCUMULATOR])
        .start_timer();
    let accumulator_node = match get_accumulator_node_by_node_hash(
        &rpc_client,
        peer_id.clone(),
        node_key,
        AccumulatorStoreType::Block,
    )
    .await
    {
        Ok(accumulator_node) => {
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
                    "accumulator node hash miss match {} :{:?}",
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
            debug!("{:?}", e);
            None
        }
    };
    accumulator_timer.observe_duration();

    state_sync_task_event_handler.send_event(StateSyncTaskEvent::new_accumulator(
        peer_id,
        node_key,
        accumulator_node,
    ));
}

async fn sync_state_node<N>(
    node_key: HashValue,
    peer_id: PeerId,
    rpc_client: NetworkRpcClient<N>,
    state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
) where
    N: NetworkService + 'static,
{
    let state_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_STATE])
        .start_timer();
    let state_node = match get_state_node_by_node_hash(&rpc_client, peer_id.clone(), node_key).await
    {
        Ok(state_node) => {
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
                    "state node hash miss match {} :{:?}",
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
            debug!("{:?}", e);
            None
        }
    };
    state_timer.observe_duration();

    state_sync_task_event_handler
        .send_event(StateSyncTaskEvent::new_state(peer_id, node_key, state_node));
}

async fn sync_txn_info<N>(
    block_id: HashValue,
    peer_id: PeerId,
    rpc_client: NetworkRpcClient<N>,
    state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
) where
    N: NetworkService + 'static,
{
    let state_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_TXN_INFO])
        .start_timer();
    let txn_infos = match get_txn_infos(&rpc_client, peer_id.clone(), block_id).await {
        Ok(infos) => {
            SYNC_METRICS
                .sync_succ_count
                .with_label_values(&[LABEL_TXN_INFO])
                .inc();
            infos
        }
        Err(e) => {
            SYNC_METRICS
                .sync_fail_count
                .with_label_values(&[LABEL_TXN_INFO])
                .inc();
            debug!("{:?}", e);
            None
        }
    };
    state_timer.observe_duration();

    state_sync_task_event_handler.send_event(StateSyncTaskEvent::new_txn_info(
        peer_id, block_id, txn_infos,
    ));
}

#[derive(Clone)]
pub struct StateSyncTaskRef<N>
where
    N: NetworkService + 'static,
{
    address: Addr<StateSyncTaskActor<N>>,
}

impl<N> SyncTaskAction for StateSyncTaskRef<N>
where
    N: NetworkService + 'static,
{
    fn activate(&self) {
        let address = self.address.clone();
        Arbiter::spawn(async move {
            let _ = address.send(SyncTaskRequest::ACTIVATE()).await;
        })
    }
}

#[async_trait::async_trait]
impl<N> StateSyncReset for StateSyncTaskRef<N>
where
    N: NetworkService + 'static,
{
    async fn reset(
        &self,
        state_root: HashValue,
        block_accumulator_root: HashValue,
        pivot_id: HashValue,
    ) {
        if let Err(e) = self
            .address
            .send(StateSyncEvent::RESET(ResetRoots {
                state_root,
                block_accumulator_root,
                pivot_id,
            }))
            .await
        {
            error!("Send RESET StateSyncEvent failed : {:?}", e);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TaskType {
    STATE,
    BlockAccumulator,
    TxnInfo,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct StateSyncTaskEvent {
    peer_id: PeerId,
    key: HashValue,
    state_node: Option<StateNode>,
    accumulator_node: Option<AccumulatorNode>,
    txn_infos: Option<Vec<TransactionInfo>>,
    task_type: TaskType,
}

impl StateSyncTaskEvent {
    pub fn new_state(peer_id: PeerId, node_key: HashValue, state_node: Option<StateNode>) -> Self {
        StateSyncTaskEvent {
            peer_id,
            key: node_key,
            state_node,
            accumulator_node: None,
            txn_infos: None,
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
            key: node_key,
            state_node: None,
            accumulator_node,
            txn_infos: None,
            task_type: TaskType::BlockAccumulator,
        }
    }

    pub fn new_txn_info(
        peer_id: PeerId,
        block_id: HashValue,
        txn_infos: Option<Vec<TransactionInfo>>,
    ) -> Self {
        StateSyncTaskEvent {
            peer_id,
            key: block_id,
            state_node: None,
            accumulator_node: None,
            txn_infos,
            task_type: TaskType::TxnInfo,
        }
    }
}

pub struct StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    block_sync_address: BlockSyncTaskRef<N>,
    download_address: Addr<DownloadActor<N>>,
    inner: Inner<N>,
}

struct Inner<N>
where
    N: NetworkService + 'static,
{
    self_peer_id: PeerId,
    roots: Roots,
    storage: Arc<dyn Store>,
    rpc_client: NetworkRpcClient<N>,
    network_service: N,
    state_sync_task: StateSyncTask<(HashValue, bool)>,
    txn_info_sync_task: StateSyncTask<HashValue>,
    block_accumulator_sync_task: StateSyncTask<HashValue>,
    state: SyncTaskState,
    total_txn_info_task: AtomicU64,
}

impl<N> Inner<N>
where
    N: NetworkService + 'static,
{
    fn new(
        self_peer_id: PeerId,
        root: (HashValue, HashValue, HashValue),
        storage: Arc<dyn Store>,
        network_service: N,
    ) -> Self {
        let roots = Roots::new(root.0, root.1, root.2);
        let mut state_sync_task = StateSyncTask::new();
        state_sync_task.push_back((*roots.state_root(), true));
        let mut block_accumulator_sync_task = StateSyncTask::new();
        block_accumulator_sync_task.push_back(*roots.block_accumulator_root());
        let mut txn_info_sync_task = StateSyncTask::new();
        txn_info_sync_task.push_back(*roots.pivot_id());
        let rpc_client = NetworkRpcClient::new(network_service.clone());

        Inner {
            self_peer_id,
            roots,
            storage,
            rpc_client,
            network_service,
            state_sync_task,
            txn_info_sync_task,
            block_accumulator_sync_task,
            state: SyncTaskState::Ready,
            total_txn_info_task: AtomicU64::new(1),
        }
    }

    fn _get_network_client(&self) -> &NetworkRpcClient<N> {
        &self.rpc_client
    }

    fn do_finish(&mut self) -> bool {
        if !self.state.is_finish() {
            info!(
                "state sync task info : {:?},\
             block accumulator sync task info : {:?},\
             txn info sync task info : {:?}.",
                self.state_sync_task.task_info(),
                self.block_accumulator_sync_task.task_info(),
                self.txn_info_sync_task.task_info(),
            );
            if self.state_sync_task.is_empty() && self.accumulator_sync_finish() {
                info!("State sync task finish.");
                self.state = SyncTaskState::Finish;
            }
        }

        self.state.is_finish()
    }

    fn accumulator_sync_finish(&self) -> bool {
        self.block_accumulator_sync_task.is_empty() && self.txn_info_sync_task.is_empty()
    }

    fn exe_state_sync_task(
        &mut self,
        state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
    ) {
        let value = self.state_sync_task.pop_front();
        if let Some((node_key, is_global)) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_STATE])
                .inc();
            if let Ok(Some(state_node)) = self.storage.get(&node_key) {
                debug!("find state_node {:?} in db.", node_key);
                self.state_sync_task
                    .insert(self.self_peer_id.clone(), (node_key, is_global));
                state_sync_task_event_handler.send_event(StateSyncTaskEvent::new_state(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(state_node),
                ));
            } else {
                let best_peer_info = get_best_peer_info(self.network_service.clone());
                debug!(
                    "sync state node {:?} from peer {:?}.",
                    node_key, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id == best_peer.get_peer_id() {
                        return;
                    }
                    let rpc_client = self.rpc_client.clone();
                    self.state_sync_task
                        .insert(best_peer.get_peer_id(), (node_key, is_global));
                    Arbiter::spawn(async move {
                        sync_state_node(
                            node_key,
                            best_peer.get_peer_id(),
                            rpc_client,
                            state_sync_task_event_handler,
                        )
                        .await;
                    });
                } else {
                    warn!("best peer is none, state sync may be failed.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_state_sync(&mut self, task_event: StateSyncTaskEvent) {
        if let Some((state_node_hash, is_global)) = self.state_sync_task.get(&task_event.peer_id) {
            let is_global = *is_global;
            //1. push back
            let current_node_key = task_event.key;
            if state_node_hash != &current_node_key {
                debug!(
                    "hash miss match {:} : {:?}",
                    state_node_hash, current_node_key
                );
                return;
            }
            let _ = self.state_sync_task.remove(&task_event.peer_id);
            if let Some(state_node) = task_event.state_node {
                if let Err(e) = self.storage.put(current_node_key, state_node.clone()) {
                    debug!("{:?}, retry {:?}.", e, current_node_key);
                    self.state_sync_task
                        .push_back((current_node_key, is_global));
                } else {
                    self.state_sync_task.do_one_task();
                    match state_node.inner() {
                        Node::Leaf(leaf) => {
                            if !is_global {
                                return;
                            }
                            match AccountState::try_from(leaf.blob().as_ref()) {
                                Err(e) => {
                                    error!("AccountState decode from blob failed : {:?}", e);
                                }
                                Ok(account_state) => {
                                    account_state.storage_roots().iter().for_each(|key| {
                                        if let Some(hash) = key {
                                            if *hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
                                                self.state_sync_task.push_back((*hash, false));
                                            }
                                        }
                                    });
                                }
                            }
                        }
                        Node::Internal(n) => {
                            for child in n.all_child() {
                                self.state_sync_task.push_back((child, is_global));
                            }
                        }
                        _ => {
                            debug!("node {:?} is null.", current_node_key);
                        }
                    }
                }
            } else {
                self.state_sync_task
                    .push_back((current_node_key, is_global));
            }
        } else {
            debug!("discard state event : {:?}", task_event);
        }
    }

    fn exe_accumulator_sync_task(
        &mut self,
        state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
    ) {
        let value = self.block_accumulator_sync_task.pop_front();
        if let Some(node_key) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_ACCUMULATOR])
                .inc();
            if let Ok(Some(accumulator_node)) =
                self.storage.get_node(AccumulatorStoreType::Block, node_key)
            {
                debug!("find accumulator_node {:?} in db.", node_key);
                self.block_accumulator_sync_task
                    .insert(self.self_peer_id.clone(), node_key);
                state_sync_task_event_handler.send_event(StateSyncTaskEvent::new_accumulator(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(accumulator_node),
                ));
            } else {
                let best_peer_info = get_best_peer_info(self.network_service.clone());
                debug!(
                    "sync accumulator node {:?} from peer {:?}.",
                    node_key, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id != best_peer.get_peer_id() {
                        self.block_accumulator_sync_task
                            .insert(best_peer.get_peer_id(), node_key);
                        let rpc_client = self.rpc_client.clone();
                        Arbiter::spawn(async move {
                            sync_accumulator_node(
                                node_key,
                                best_peer.get_peer_id(),
                                rpc_client,
                                state_sync_task_event_handler,
                            )
                            .await;
                        });
                    }
                } else {
                    warn!("best peer is none.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_accumulator_sync(
        &mut self,
        task_event: StateSyncTaskEvent,
        txn_info_event_handler: Box<dyn SendSyncEventHandler<TxnInfoEvent>>,
    ) {
        if let Some(accumulator_node_hash) =
            self.block_accumulator_sync_task.get(&task_event.peer_id)
        {
            //1. push back
            let current_node_key = task_event.key;
            if accumulator_node_hash != &current_node_key {
                warn!(
                    "hash miss match {:} : {:?}",
                    accumulator_node_hash, current_node_key
                );
                return;
            }
            let _ = self.block_accumulator_sync_task.remove(&task_event.peer_id);
            if let Some(accumulator_node) = task_event.accumulator_node {
                if let Err(e) = self
                    .storage
                    .save_node(AccumulatorStoreType::Block, accumulator_node.clone())
                {
                    debug!("{:?}", e);
                    self.block_accumulator_sync_task.push_back(current_node_key);
                } else {
                    debug!("receive accumulator_node: {:?}", accumulator_node);
                    self.block_accumulator_sync_task.do_one_task();
                    match accumulator_node {
                        AccumulatorNode::Leaf(leaf) => {
                            self.txn_info_sync_task.push_back(leaf.value());
                            self.total_txn_info_task.fetch_add(1, Ordering::Relaxed);
                            txn_info_event_handler.send_event(TxnInfoEvent(None));
                        }
                        AccumulatorNode::Internal(n) => {
                            if n.left() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                self.block_accumulator_sync_task.push_back(n.left());
                            }
                            if n.right() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                self.block_accumulator_sync_task.push_back(n.right());
                            }
                        }
                        _ => {
                            debug!("node {:?} is null.", current_node_key);
                        }
                    }
                }
            } else {
                self.block_accumulator_sync_task.push_back(current_node_key);
            }
        } else {
            debug!("discard state event : {:?}", task_event);
        }
    }

    fn exe_txn_info_sync_task(
        &mut self,
        state_sync_task_event_handler: Box<dyn SendSyncEventHandler<StateSyncTaskEvent>>,
    ) {
        let value = self.txn_info_sync_task.pop_front();
        if let Some(block_id) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_TXN_INFO])
                .inc();
            if let Ok(txn_infos) = self.storage.get_block_transaction_infos(block_id) {
                debug!("find txn info {:?} in db.", block_id);
                self.txn_info_sync_task
                    .insert(self.self_peer_id.clone(), block_id);
                state_sync_task_event_handler.send_event(StateSyncTaskEvent::new_txn_info(
                    self.self_peer_id.clone(),
                    block_id,
                    Some(txn_infos),
                ));
            } else {
                let best_peer_info = get_best_peer_info(self.network_service.clone());
                debug!(
                    "sync txn info {:?} from peer {:?}.",
                    block_id, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id == best_peer.get_peer_id() {
                        return;
                    }
                    let rpc_client = self.rpc_client.clone();
                    self.txn_info_sync_task
                        .insert(best_peer.get_peer_id(), block_id);
                    Arbiter::spawn(async move {
                        sync_txn_info(
                            block_id,
                            best_peer.get_peer_id(),
                            rpc_client,
                            state_sync_task_event_handler,
                        )
                        .await;
                    });
                } else {
                    warn!("best peer is none, state sync may be failed.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_txn_info_sync(
        &mut self,
        task_event: StateSyncTaskEvent,
        txn_info_event_handler: Box<dyn SendSyncEventHandler<TxnInfoEvent>>,
    ) {
        // if let Some(block_id) = self.txn_info_sync_task.get(&task_event.peer_id) {
        //1. push back
        let current_block_id = task_event.key;
        // if block_id != &current_block_id {
        //     debug!("hash miss match {:} : {:?}", block_id, current_block_id);
        //     return;
        // }
        let _ = self.txn_info_sync_task.remove(&task_event.peer_id);
        if let Some(txn_infos) = task_event.txn_infos {
            if let Err(e) = self.save_txn_infos(current_block_id, txn_infos) {
                debug!("{:?}, retry {:?}.", e, current_block_id);
                self.txn_info_sync_task.push_back(current_block_id);
                txn_info_event_handler.send_event(TxnInfoEvent(None));
            } else {
                self.txn_info_sync_task.do_one_task();
            }
        } else {
            self.txn_info_sync_task.push_back(current_block_id);
            txn_info_event_handler.send_event(TxnInfoEvent(None));
        }
        // } else {
        //     info!("discard state event : {:?}", task_event);
        // }
    }

    pub fn save_txn_infos(
        &self,
        block_id: HashValue,
        txn_infos: Vec<TransactionInfo>,
    ) -> Result<()> {
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        self.storage
            .save_block_txn_info_ids(block_id, txn_info_ids)?;
        self.storage.save_transaction_infos(txn_infos)
    }
}

pub struct StateSyncTask<T> {
    wait_2_sync: VecDeque<T>,
    syncing_nodes: HashMap<PeerId, T>,
    done_tasks: AtomicU64,
}

impl<T> StateSyncTask<T> {
    fn new() -> Self {
        Self {
            wait_2_sync: VecDeque::new(),
            syncing_nodes: HashMap::new(),
            done_tasks: AtomicU64::new(0),
        }
    }

    fn do_one_task(&self) {
        self.done_tasks.fetch_add(1, Ordering::Relaxed);
    }

    fn is_empty(&self) -> bool {
        self.wait_2_sync.is_empty() && self.syncing_nodes.is_empty()
    }

    fn task_info(&self) -> (usize, usize, u64) {
        (
            self.wait_2_sync.len(),
            self.syncing_nodes.len(),
            self.done_tasks.load(Ordering::Relaxed),
        )
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
        self.done_tasks = AtomicU64::new(0);
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

impl<N> StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    pub fn launch(
        self_peer_id: PeerId,
        root: (HashValue, HashValue, HashValue),
        storage: Arc<dyn Store>,
        network_service: N,
        block_sync_address: BlockSyncTaskRef<N>,
        download_address: Addr<DownloadActor<N>>,
    ) -> StateSyncTaskRef<N> {
        let inner = Inner::new(self_peer_id, root, storage, network_service);
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            inner,
            block_sync_address,
            download_address,
        });
        StateSyncTaskRef { address }
    }

    pub fn reset(
        &mut self,
        state_root: &HashValue,
        block_accumulator_root: &HashValue,
        pivot_id: &HashValue,
        address: Addr<StateSyncTaskActor<N>>,
    ) {
        debug!(
            "reset state sync task with state root : {:?}, block accumulator root : {:?}.",
            state_root, block_accumulator_root
        );
        self.inner.roots = Roots::new(*state_root, *block_accumulator_root, *pivot_id);

        let old_state_is_empty = self.inner.state_sync_task.is_empty();
        self.inner.state_sync_task.clear();
        self.inner
            .state_sync_task
            .push_back((*self.inner.roots.state_root(), true));

        let old_block_accumulator_is_empty = self.inner.block_accumulator_sync_task.is_empty();
        self.inner.block_accumulator_sync_task.clear();
        self.inner
            .block_accumulator_sync_task
            .push_back(*self.inner.roots.block_accumulator_root());

        let old_txn_info_is_empty = self.inner.txn_info_sync_task.is_empty();
        self.inner.txn_info_sync_task.clear();
        self.inner
            .txn_info_sync_task
            .push_back(*self.inner.roots.pivot_id());

        if old_state_is_empty {
            self.inner.exe_state_sync_task(Box::new(address.clone()));
        }
        if old_block_accumulator_is_empty {
            self.inner
                .exe_accumulator_sync_task(Box::new(address.clone()));
        }
        if old_txn_info_is_empty {
            self.inner.exe_txn_info_sync_task(Box::new(address));
        }
    }

    fn activation_task(&mut self, address: Addr<StateSyncTaskActor<N>>) {
        if self.inner.state.is_failed() {
            debug!("activation state sync task.");
            self.inner.state = SyncTaskState::Syncing;
            self.inner.exe_state_sync_task(Box::new(address.clone()));
            self.inner
                .exe_accumulator_sync_task(Box::new(address.clone()));
            self.inner.exe_txn_info_sync_task(Box::new(address));
        }
    }
}

impl<N> Actor for StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("state sync task started.");
        self.inner.exe_state_sync_task(Box::new(ctx.address()));
        self.inner
            .exe_accumulator_sync_task(Box::new(ctx.address()));
        self.inner.exe_txn_info_sync_task(Box::new(ctx.address()));
    }
}

impl<N> Handler<TxnInfoEvent> for StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, event: TxnInfoEvent, ctx: &mut Self::Context) -> Self::Result {
        if let Some(block_id) = event.0 {
            self.inner.txn_info_sync_task.push_back(block_id);
            self.inner
                .total_txn_info_task
                .fetch_add(1, Ordering::Relaxed);
        }
        self.inner.exe_txn_info_sync_task(Box::new(ctx.address()));
    }
}

impl<N> Handler<StateSyncTaskEvent> for StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, task_event: StateSyncTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let task_type = task_event.task_type.clone();
        match task_type {
            TaskType::STATE => self.inner.handle_state_sync(task_event),
            TaskType::TxnInfo => self
                .inner
                .handle_txn_info_sync(task_event, Box::new(ctx.address())),
            TaskType::BlockAccumulator => self
                .inner
                .handle_accumulator_sync(task_event, Box::new(ctx.address())),
        }

        if self.inner.accumulator_sync_finish() {
            self.block_sync_address.start();
        }

        if !self.inner.do_finish() {
            match task_type {
                TaskType::STATE => self.inner.exe_state_sync_task(Box::new(ctx.address())),
                TaskType::BlockAccumulator => self
                    .inner
                    .exe_accumulator_sync_task(Box::new(ctx.address())),
                TaskType::TxnInfo => {}
            }
        } else if self.inner.total_txn_info_task.load(Ordering::Relaxed)
            == self
                .inner
                .txn_info_sync_task
                .done_tasks
                .load(Ordering::Relaxed)
        {
            self.download_address.do_send(SyncTaskType::STATE);
            ctx.stop();
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
enum StateSyncEvent {
    RESET(ResetRoots),
}

#[derive(Debug, Clone)]
struct ResetRoots {
    state_root: HashValue,
    block_accumulator_root: HashValue,
    pivot_id: HashValue,
}

impl<N> Handler<StateSyncEvent> for StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            StateSyncEvent::RESET(roots) => {
                self.reset(
                    &roots.state_root,
                    &roots.block_accumulator_root,
                    &roots.pivot_id,
                    ctx.address(),
                );
            }
        }
        Ok(())
    }
}

impl<N> Handler<SyncTaskRequest> for StateSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = Result<SyncTaskResponse>;

    fn handle(&mut self, action: SyncTaskRequest, ctx: &mut Self::Context) -> Self::Result {
        match action {
            SyncTaskRequest::ACTIVATE() => {
                self.activation_task(ctx.address());
                Ok(SyncTaskResponse::None)
            }
        }
    }
}

fn get_best_peer_info<N>(network_service: N) -> Option<PeerInfo>
where
    N: NetworkService + 'static,
{
    block_on(async move {
        if let Ok(peer_info) = network_service.best_peer().await {
            peer_info
        } else {
            None
        }
    })
}
