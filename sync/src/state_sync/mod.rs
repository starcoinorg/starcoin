use crate::block_sync::BlockSyncTaskRef;
use crate::download::DownloadActor;
use crate::helper::{get_accumulator_node_by_node_hash, get_state_node_by_node_hash, get_txn_info};
use crate::sync_metrics::{LABEL_ACCUMULATOR, LABEL_STATE, LABEL_TXN_INFO, SYNC_METRICS};
use crate::sync_task::{
    SyncTaskAction, SyncTaskRequest, SyncTaskResponse, SyncTaskState, SyncTaskType,
};
use crate::StateSyncReset;
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use crypto::{hash::PlainCryptoHash, HashValue};
use forkable_jellyfish_merkle::node_type::Node;
use forkable_jellyfish_merkle::SPARSE_MERKLE_PLACEHOLDER_HASH;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_accumulator::node::{AccumulatorStoreType, ACCUMULATOR_PLACEHOLDER_HASH};
use starcoin_accumulator::AccumulatorNode;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use traits::Consensus;
use types::{
    account_state::AccountState,
    peer_info::{PeerId, PeerInfo},
    transaction::TransactionInfo,
};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct TxnInfoEvent;

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

async fn sync_accumulator_node<C>(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor<C>>,
    accumulator_type: AccumulatorStoreType,
) where
    C: Consensus + Sync + Send + 'static + Clone,
{
    let accumulator_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_ACCUMULATOR])
        .start_timer();
    let accumulator_node = match get_accumulator_node_by_node_hash(
        &network_service,
        peer_id.clone(),
        node_key,
        accumulator_type.clone(),
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

    if let Err(err) = address.try_send(StateSyncTaskEvent::new_accumulator(
        peer_id,
        node_key,
        accumulator_node,
        accumulator_type,
    )) {
        error!("Send accumulator StateSyncTaskEvent failed : {:?}", err);
    };
}

async fn sync_state_node<C>(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor<C>>,
) where
    C: Consensus + Sync + Send + 'static + Clone,
{
    let state_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_STATE])
        .start_timer();
    let state_node =
        match get_state_node_by_node_hash(&network_service, peer_id.clone(), node_key).await {
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

    if let Err(err) = address.try_send(StateSyncTaskEvent::new_state(peer_id, node_key, state_node))
    {
        error!("Send state StateSyncTaskEvent failed : {:?}", err);
    };
}

async fn sync_txn_info<C>(
    txn_info_hash: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor<C>>,
) where
    C: Consensus + Sync + Send + 'static + Clone,
{
    let state_timer = SYNC_METRICS
        .sync_done_time
        .with_label_values(&[LABEL_TXN_INFO])
        .start_timer();
    let txn_info = match get_txn_info(&network_service, peer_id.clone(), txn_info_hash).await {
        Ok(Some(info)) => {
            if txn_info_hash == info.crypto_hash() {
                SYNC_METRICS
                    .sync_succ_count
                    .with_label_values(&[LABEL_TXN_INFO])
                    .inc();
                Some(info)
            } else {
                SYNC_METRICS
                    .sync_verify_fail_count
                    .with_label_values(&[LABEL_TXN_INFO])
                    .inc();
                warn!(
                    "txn info hash miss match {} :{:?}",
                    txn_info_hash,
                    info.crypto_hash()
                );
                None
            }
        }
        Ok(None) => {
            SYNC_METRICS
                .sync_fail_count
                .with_label_values(&[LABEL_TXN_INFO])
                .inc();
            None
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

    if let Err(err) = address.try_send(StateSyncTaskEvent::new_txn_info(
        peer_id,
        txn_info_hash,
        txn_info,
    )) {
        error!("Send txn info StateSyncTaskEvent failed : {:?}", err);
    };
}

#[derive(Clone)]
pub struct StateSyncTaskRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    address: Addr<StateSyncTaskActor<C>>,
}

impl<C> SyncTaskAction for StateSyncTaskRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn activate(&self) {
        let address = self.address.clone();
        Arbiter::spawn(async move {
            let _ = address.send(SyncTaskRequest::ACTIVATE()).await;
        })
    }
}

#[async_trait::async_trait]
impl<C> StateSyncReset for StateSyncTaskRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    async fn reset(
        &self,
        state_root: HashValue,
        txn_accumulator_root: HashValue,
        block_accumulator_root: HashValue,
    ) {
        if let Err(e) = self
            .address
            .send(StateSyncEvent::RESET(ResetRoots {
                state_root,
                txn_accumulator_root,
                block_accumulator_root,
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
    TxnAccumulator,
    BlockAccumulator,
    TxnInfo,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncTaskEvent {
    peer_id: PeerId,
    node_key: HashValue,
    state_node: Option<StateNode>,
    accumulator_node: Option<AccumulatorNode>,
    txn_info: Option<TransactionInfo>,
    task_type: TaskType,
}

impl StateSyncTaskEvent {
    pub fn new_state(peer_id: PeerId, node_key: HashValue, state_node: Option<StateNode>) -> Self {
        StateSyncTaskEvent {
            peer_id,
            node_key,
            state_node,
            accumulator_node: None,
            txn_info: None,
            task_type: TaskType::STATE,
        }
    }

    pub fn new_accumulator(
        peer_id: PeerId,
        node_key: HashValue,
        accumulator_node: Option<AccumulatorNode>,
        accumulator_type: AccumulatorStoreType,
    ) -> Self {
        StateSyncTaskEvent {
            peer_id,
            node_key,
            state_node: None,
            accumulator_node,
            txn_info: None,
            task_type: match accumulator_type {
                AccumulatorStoreType::Block => TaskType::BlockAccumulator,
                AccumulatorStoreType::Transaction => TaskType::TxnAccumulator,
            },
        }
    }

    pub fn new_txn_info(
        peer_id: PeerId,
        node_key: HashValue,
        txn_info: Option<TransactionInfo>,
    ) -> Self {
        StateSyncTaskEvent {
            peer_id,
            node_key,
            state_node: None,
            accumulator_node: None,
            txn_info,
            task_type: TaskType::TxnInfo,
        }
    }
}

pub struct StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    self_peer_id: PeerId,
    roots: Roots,
    storage: Arc<dyn Store>,
    network_service: NetworkAsyncService,
    state_sync_task: StateSyncTask<(HashValue, bool)>,
    txn_accumulator_sync_task: StateSyncTask<HashValue>,
    txn_info_sync_task: StateSyncTask<HashValue>,
    block_accumulator_sync_task: StateSyncTask<HashValue>,
    block_sync_address: BlockSyncTaskRef<C>,
    state: SyncTaskState,
    download_address: Addr<DownloadActor<C>>,
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

impl<C> StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        self_peer_id: PeerId,
        root: (HashValue, HashValue, HashValue),
        storage: Arc<dyn Store>,
        network_service: NetworkAsyncService,
        block_sync_address: BlockSyncTaskRef<C>,
        download_address: Addr<DownloadActor<C>>,
    ) -> StateSyncTaskRef<C> {
        let roots = Roots::new(root.0, root.1, root.2);
        let mut state_sync_task = StateSyncTask::new();
        state_sync_task.push_back((*roots.state_root(), true));
        let mut txn_accumulator_sync_task = StateSyncTask::new();
        txn_accumulator_sync_task.push_back(*roots.txn_accumulator_root());
        let mut block_accumulator_sync_task = StateSyncTask::new();
        block_accumulator_sync_task.push_back(*roots.block_accumulator_root());
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            self_peer_id,
            roots,
            storage,
            network_service,
            state_sync_task,
            txn_accumulator_sync_task,
            txn_info_sync_task: StateSyncTask::new(),
            block_accumulator_sync_task,
            block_sync_address,
            state: SyncTaskState::Ready,
            download_address,
        });
        StateSyncTaskRef { address }
    }

    fn do_finish(&mut self) -> bool {
        if !self.state.is_finish() {
            info!(
                "state sync task info : {:?},\
             txn accumulator sync task info : {:?},\
             block accumulator sync task info : {:?},\
             txn info sync task info : {:?}.",
                self.state_sync_task.task_info(),
                self.txn_accumulator_sync_task.task_info(),
                self.block_accumulator_sync_task.task_info(),
                self.txn_info_sync_task.task_info(),
            );
            if self.state_sync_task.is_empty()
                && self.accumulator_sync_finish()
                && self.txn_info_sync_task.is_empty()
            {
                info!("State sync task finish.");
                self.state = SyncTaskState::Finish;
            }
        }

        self.state.is_finish()
    }

    fn accumulator_sync_finish(&self) -> bool {
        self.txn_accumulator_sync_task.is_empty() && self.block_accumulator_sync_task.is_empty()
    }

    fn exe_state_sync_task(&mut self, address: Addr<StateSyncTaskActor<C>>) {
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
                if let Err(err) = address.try_send(StateSyncTaskEvent::new_state(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(state_node),
                )) {
                    error!("Send state StateSyncTaskEvent failed : {:?}", err);
                };
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
                    let network_service = self.network_service.clone();
                    self.state_sync_task
                        .insert(best_peer.get_peer_id(), (node_key, is_global));
                    Arbiter::spawn(async move {
                        sync_state_node(
                            node_key,
                            best_peer.get_peer_id(),
                            network_service,
                            address,
                        )
                        .await;
                    });
                } else {
                    warn!("{:?}", "best peer is none, state sync may be failed.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_state_sync(&mut self, task_event: StateSyncTaskEvent) {
        if let Some((state_node_hash, is_global)) = self.state_sync_task.get(&task_event.peer_id) {
            let is_global = *is_global;
            //1. push back
            let current_node_key = task_event.node_key;
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
        address: Addr<StateSyncTaskActor<C>>,
        accumulator_type: AccumulatorStoreType,
    ) {
        let accumulator_sync_task = match accumulator_type {
            AccumulatorStoreType::Transaction => &mut self.txn_accumulator_sync_task,
            AccumulatorStoreType::Block => &mut self.block_accumulator_sync_task,
        };
        let value = accumulator_sync_task.pop_front();
        if let Some(node_key) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_ACCUMULATOR])
                .inc();
            if let Ok(Some(accumulator_node)) =
                self.storage.get_node(accumulator_type.clone(), node_key)
            {
                debug!("find accumulator_node {:?} in db.", node_key);
                accumulator_sync_task.insert(self.self_peer_id.clone(), node_key);
                if let Err(err) = address.try_send(StateSyncTaskEvent::new_accumulator(
                    self.self_peer_id.clone(),
                    node_key,
                    Some(accumulator_node),
                    accumulator_type,
                )) {
                    error!("Send accumulator StateSyncTaskEvent failed : {:?}", err);
                };
            } else {
                let best_peer_info = get_best_peer_info(self.network_service.clone());
                debug!(
                    "sync accumulator node {:?} from peer {:?}.",
                    node_key, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id != best_peer.get_peer_id() {
                        accumulator_sync_task.insert(best_peer.get_peer_id(), node_key);
                        let network_service = self.network_service.clone();
                        Arbiter::spawn(async move {
                            sync_accumulator_node(
                                node_key,
                                best_peer.get_peer_id(),
                                network_service,
                                address,
                                accumulator_type,
                            )
                            .await;
                        });
                    }
                } else {
                    warn!("{:?}", "best peer is none.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_accumulator_sync(
        &mut self,
        task_event: StateSyncTaskEvent,
        address: Addr<StateSyncTaskActor<C>>,
    ) {
        let accumulator_sync_task = match task_event.task_type {
            TaskType::TxnAccumulator => &mut self.txn_accumulator_sync_task,
            _ => &mut self.block_accumulator_sync_task,
        };
        if let Some(accumulator_node_hash) = accumulator_sync_task.get(&task_event.peer_id) {
            //1. push back
            let current_node_key = task_event.node_key;
            if accumulator_node_hash != &current_node_key {
                warn!(
                    "hash miss match {:} : {:?}",
                    accumulator_node_hash, current_node_key
                );
                return;
            }
            let _ = accumulator_sync_task.remove(&task_event.peer_id);
            if let Some(accumulator_node) = task_event.accumulator_node {
                if let Err(e) = self.storage.save_node(
                    match task_event.task_type {
                        TaskType::TxnAccumulator => AccumulatorStoreType::Transaction,
                        _ => AccumulatorStoreType::Block,
                    },
                    accumulator_node.clone(),
                ) {
                    debug!("{:?}", e);
                    accumulator_sync_task.push_back(current_node_key);
                } else {
                    debug!("receive accumulator_node: {:?}", accumulator_node);
                    accumulator_sync_task.do_one_task();
                    match accumulator_node {
                        AccumulatorNode::Leaf(leaf) => {
                            if let TaskType::TxnAccumulator = task_event.task_type {
                                self.txn_info_sync_task.push_back(leaf.value());
                                address.do_send(TxnInfoEvent {});
                            }
                        }
                        AccumulatorNode::Internal(n) => {
                            if n.left() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                accumulator_sync_task.push_back(n.left());
                            }
                            if n.right() != *ACCUMULATOR_PLACEHOLDER_HASH {
                                accumulator_sync_task.push_back(n.right());
                            }
                        }
                        _ => {
                            debug!("node {:?} is null.", current_node_key);
                        }
                    }
                }
            } else {
                accumulator_sync_task.push_back(current_node_key);
            }
        } else {
            debug!("discard state event : {:?}", task_event);
        }
    }

    fn exe_txn_info_sync_task(&mut self, address: Addr<StateSyncTaskActor<C>>) {
        let value = self.txn_info_sync_task.pop_front();
        if let Some(txn_info_hash) = value {
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_TXN_INFO])
                .inc();
            if let Ok(Some(txn_info)) = self.storage.get_transaction_info(txn_info_hash) {
                debug!("find txn info {:?} in db.", txn_info_hash);
                self.txn_info_sync_task
                    .insert(self.self_peer_id.clone(), txn_info_hash);
                if let Err(err) = address.try_send(StateSyncTaskEvent::new_txn_info(
                    self.self_peer_id.clone(),
                    txn_info_hash,
                    Some(txn_info),
                )) {
                    error!("Send txn info StateSyncTaskEvent failed : {:?}", err);
                };
            } else {
                let best_peer_info = get_best_peer_info(self.network_service.clone());
                debug!(
                    "sync txn info {:?} from peer {:?}.",
                    txn_info_hash, best_peer_info
                );
                if let Some(best_peer) = best_peer_info {
                    if self.self_peer_id == best_peer.get_peer_id() {
                        return;
                    }
                    let network_service = self.network_service.clone();
                    self.txn_info_sync_task
                        .insert(best_peer.get_peer_id(), txn_info_hash);
                    Arbiter::spawn(async move {
                        sync_txn_info(
                            txn_info_hash,
                            best_peer.get_peer_id(),
                            network_service,
                            address,
                        )
                        .await;
                    });
                } else {
                    warn!("{:?}", "best peer is none, state sync may be failed.");
                    self.state = SyncTaskState::Failed;
                }
            }
        }
    }

    fn handle_txn_info_sync(&mut self, task_event: StateSyncTaskEvent) {
        if let Some(txn_info_hash) = self.txn_info_sync_task.get(&task_event.peer_id) {
            //1. push back
            let current_node_key = task_event.node_key;
            if txn_info_hash != &current_node_key {
                debug!(
                    "hash miss match {:} : {:?}",
                    txn_info_hash, current_node_key
                );
                return;
            }
            let _ = self.txn_info_sync_task.remove(&task_event.peer_id);
            if let Some(txn_info) = task_event.txn_info {
                let mut infos = Vec::new();
                infos.push(txn_info);
                if let Err(e) = self.storage.save_transaction_infos(infos) {
                    debug!("{:?}, retry {:?}.", e, current_node_key);
                    self.txn_info_sync_task.push_back(current_node_key);
                } else {
                    self.txn_info_sync_task.do_one_task();
                }
            } else {
                self.txn_info_sync_task.push_back(current_node_key);
            }
        } else {
            debug!("discard state event : {:?}", task_event);
        }
    }

    pub fn reset(
        &mut self,
        state_root: &HashValue,
        txn_accumulator_root: &HashValue,
        block_accumulator_root: &HashValue,
        address: Addr<StateSyncTaskActor<C>>,
    ) {
        debug!("reset state sync task with state root : {:?}, txn accumulator root : {:?}, block accumulator root : {:?}.",
               state_root, txn_accumulator_root, block_accumulator_root);
        self.roots = Roots::new(*state_root, *txn_accumulator_root, *block_accumulator_root);

        let old_state_is_empty = self.state_sync_task.is_empty();
        self.state_sync_task.clear();
        self.state_sync_task
            .push_back((*self.roots.state_root(), true));

        let old_txn_accumulator_is_empty = self.txn_accumulator_sync_task.is_empty();
        self.txn_accumulator_sync_task.clear();
        self.txn_accumulator_sync_task
            .push_back(*self.roots.txn_accumulator_root());

        let old_block_accumulator_is_empty = self.block_accumulator_sync_task.is_empty();
        self.block_accumulator_sync_task.clear();
        self.block_accumulator_sync_task
            .push_back(*self.roots.block_accumulator_root());

        self.txn_info_sync_task.clear();

        if old_state_is_empty {
            self.exe_state_sync_task(address.clone());
        }
        if old_txn_accumulator_is_empty {
            self.exe_accumulator_sync_task(address.clone(), AccumulatorStoreType::Transaction);
        }
        if old_block_accumulator_is_empty {
            self.exe_accumulator_sync_task(address, AccumulatorStoreType::Block);
        }
    }

    fn activation_task(&mut self, address: Addr<StateSyncTaskActor<C>>) {
        if self.state.is_failed() {
            debug!("activation state sync task.");
            self.state = SyncTaskState::Syncing;
            self.exe_state_sync_task(address.clone());
            self.exe_accumulator_sync_task(address.clone(), AccumulatorStoreType::Transaction);
            self.exe_accumulator_sync_task(address, AccumulatorStoreType::Block);
        }
    }
}

impl<C> Actor for StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.exe_state_sync_task(ctx.address());
        self.exe_accumulator_sync_task(ctx.address(), AccumulatorStoreType::Transaction);
        self.exe_accumulator_sync_task(ctx.address(), AccumulatorStoreType::Block);
    }
}

impl<C> Handler<TxnInfoEvent> for StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: TxnInfoEvent, ctx: &mut Self::Context) -> Self::Result {
        self.exe_txn_info_sync_task(ctx.address());
        Ok(())
    }
}

impl<C> Handler<StateSyncTaskEvent> for StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, task_event: StateSyncTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let task_type = task_event.task_type.clone();
        match task_type.clone() {
            TaskType::STATE => self.handle_state_sync(task_event),
            TaskType::TxnInfo => self.handle_txn_info_sync(task_event),
            _ => self.handle_accumulator_sync(task_event, ctx.address()),
        }

        if self.accumulator_sync_finish() {
            self.block_sync_address.start();
        }

        if !self.do_finish() {
            match task_type {
                TaskType::STATE => self.exe_state_sync_task(ctx.address()),
                TaskType::TxnAccumulator => {
                    self.exe_accumulator_sync_task(ctx.address(), AccumulatorStoreType::Transaction)
                }
                TaskType::BlockAccumulator => {
                    self.exe_accumulator_sync_task(ctx.address(), AccumulatorStoreType::Block)
                }
                TaskType::TxnInfo => self.exe_txn_info_sync_task(ctx.address()),
            }
        } else {
            self.download_address.do_send(SyncTaskType::STATE);
            ctx.stop();
        }
        Ok(())
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
    txn_accumulator_root: HashValue,
    block_accumulator_root: HashValue,
}

impl<C> Handler<StateSyncEvent> for StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
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

impl<C> Handler<SyncTaskRequest> for StateSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
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

fn get_best_peer_info(network_service: NetworkAsyncService) -> Option<PeerInfo> {
    block_on(async move {
        if let Ok(peer_info) = network_service.best_peer().await {
            peer_info
        } else {
            None
        }
    })
}
