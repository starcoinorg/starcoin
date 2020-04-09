use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{Broadcast, BusActor};
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use forkable_jellyfish_merkle::node_type::Node;
use logger::prelude::*;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use parking_lot::Mutex;
use starcoin_state_tree::{StateNode, StateNodeStore};
use starcoin_sync_api::{StateSyncReset, SyncMetadata};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use traits::Consensus;
use types::{peer_info::PeerId, system_events::SystemEvents};

async fn sync_state_node<E, C>(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor<E, C>>,
) where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    debug!("sync_state_node : {:?}", node_key);
    let get_state_node_by_node_hash_req = RPCRequest::GetStateNodeByNodeHash(node_key);
    let state_node = if let RPCResponse::GetStateNodeByNodeHash(state_node) = network_service
        .send_request(
            peer_id.clone().into(),
            get_state_node_by_node_hash_req.clone(),
            do_duration(DELAY_TIME),
        )
        .await
        .unwrap()
    {
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
    } else {
        warn!("{:?}", "error RPCResponse type.");
        None
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
pub struct StateSyncTaskRef<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    address: Addr<StateSyncTaskActor<E, C>>,
}

#[async_trait::async_trait]
impl<E, C> StateSyncReset for StateSyncTaskRef<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    async fn reset(&self, root: HashValue) {
        if let Err(e) = self.address.send(StateSyncEvent { root }).await {
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

pub struct StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    peer_id: PeerId,
    root: HashValue,
    state_node_storage: Arc<dyn StateNodeStore>,
    network_service: NetworkAsyncService,
    downloader: Arc<Downloader<E, C>>,
    wait_2_sync: VecDeque<HashValue>,
    sync_metadata: SyncMetadata,
    bus: Addr<BusActor>,
    syncing_nodes: Mutex<HashMap<PeerId, HashValue>>,
}

impl<E, C> StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        my_peer_id: PeerId,
        root: HashValue,
        state_node_storage: Arc<dyn StateNodeStore>,
        network_service: NetworkAsyncService,
        downloader: Arc<Downloader<E, C>>,
        sync_metadata: SyncMetadata,
        bus: Addr<BusActor>,
    ) -> StateSyncTaskRef<E, C> {
        let mut wait_2_sync: VecDeque<HashValue> = VecDeque::new();
        wait_2_sync.push_back(root.clone());
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            peer_id: my_peer_id,
            root,
            state_node_storage,
            network_service,
            downloader,
            wait_2_sync,
            sync_metadata,
            bus,
            syncing_nodes: Mutex::new(HashMap::new()),
        });
        StateSyncTaskRef { address }
    }

    fn exe_task(&mut self, address: Addr<StateSyncTaskActor<E, C>>) {
        let node_key = self.wait_2_sync.pop_front().unwrap();
        if let Some(state_node) = self.state_node_storage.get(&node_key).unwrap() {
            self.syncing_nodes
                .lock()
                .insert(self.peer_id.clone(), node_key.clone());
            if let Err(err) = address.try_send(StateSyncTaskEvent {
                peer_id: self.peer_id.clone(),
                node_key,
                state_node: Some(state_node),
            }) {
                warn!("err:{:?}", err);
            };
        } else {
            let downloader = self.downloader.clone();
            let network_service = self.network_service.clone();
            let best_peer = Downloader::best_peer(downloader.clone())
                .unwrap()
                .get_peer_id();
            self.syncing_nodes
                .lock()
                .insert(best_peer.clone(), node_key.clone());
            Arbiter::spawn(async move {
                sync_state_node(node_key, best_peer, network_service, address).await;
            });
        }
    }

    pub fn reset(&mut self, root: &HashValue) {
        self.wait_2_sync.clear();
        self.root = root.clone();
        self.wait_2_sync.push_back(root.clone());
        self.syncing_nodes.lock().clear();
    }
}

impl<E, C> Actor for StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor started.");
        self.exe_task(ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor stopped.");
    }
}

impl<E, C> Handler<StateSyncTaskEvent> for StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, task_event: StateSyncTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let mut lock = self.syncing_nodes.lock();
        if let Some(state_node_hash) = lock.get(&task_event.peer_id) {
            //1. push back
            let current_node_key = task_event.node_key;
            if state_node_hash == &current_node_key {
                let _ = lock.remove(&task_event.peer_id);
                drop(lock);
                if let Some(state_node) = task_event.state_node {
                    match state_node.inner() {
                        Node::Leaf(_) => {}
                        Node::Internal(n) => {
                            for child in n.all_child() {
                                self.wait_2_sync.push_back(child);
                            }
                        }
                        _ => {
                            warn!("node {:?} is null.", current_node_key);
                        }
                    }
                } else {
                    self.wait_2_sync.push_back(current_node_key);
                }

                //2. exe_task
                if self.wait_2_sync.is_empty() {
                    if let Err(e) = self.sync_metadata.sync_done() {
                        warn!("err:{:?}", e);
                    } else {
                        info!("sync_done : {:?}", self.sync_metadata.get_pivot());
                        let bus = self.bus.clone();
                        Arbiter::spawn(async move {
                            let _ = bus
                                .send(Broadcast {
                                    msg: SystemEvents::StateSyncDone(),
                                })
                                .await;
                        });
                        ctx.stop();
                    }
                } else {
                    self.exe_task(ctx.address());
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
    root: HashValue,
}

impl<E, C> Handler<StateSyncEvent> for StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.reset(&msg.root);
        Ok(())
    }
}
