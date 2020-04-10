use crate::helper::send_sync_request;
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{Broadcast, BusActor};
use crypto::hash::HashValue;
use forkable_jellyfish_merkle::node_type::Node;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{SyncRpcRequest, SyncRpcResponse};
use parking_lot::Mutex;
use starcoin_state_tree::{StateNode, StateNodeStore};
use starcoin_sync_api::{StateSyncReset, SyncMetadata};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use types::{peer_info::PeerId, system_events::SystemEvents};

async fn sync_state_node(
    node_key: HashValue,
    peer_id: PeerId,
    network_service: NetworkAsyncService,
    address: Addr<StateSyncTaskActor>,
) {
    debug!("sync_state_node : {:?}", node_key);
    let get_state_node_by_node_hash_req = SyncRpcRequest::GetStateNodeByNodeHash(node_key);
    let state_node = if let SyncRpcResponse::GetStateNodeByNodeHash(state_node) = send_sync_request(
        &network_service,
        peer_id.clone(),
        get_state_node_by_node_hash_req.clone(),
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
pub struct StateSyncTaskRef {
    address: Addr<StateSyncTaskActor>,
}

#[async_trait::async_trait]
impl StateSyncReset for StateSyncTaskRef {
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

pub struct StateSyncTaskActor {
    self_peer_id: PeerId,
    root: HashValue,
    state_node_storage: Arc<dyn StateNodeStore>,
    network_service: NetworkAsyncService,
    wait_2_sync: VecDeque<HashValue>,
    sync_metadata: SyncMetadata,
    bus: Addr<BusActor>,
    syncing_nodes: Mutex<HashMap<PeerId, HashValue>>,
}

impl StateSyncTaskActor {
    pub fn launch(
        self_peer_id: PeerId,
        root: HashValue,
        state_node_storage: Arc<dyn StateNodeStore>,
        network_service: NetworkAsyncService,
        sync_metadata: SyncMetadata,
        bus: Addr<BusActor>,
    ) -> StateSyncTaskRef {
        let mut wait_2_sync: VecDeque<HashValue> = VecDeque::new();
        wait_2_sync.push_back(root.clone());
        let address = StateSyncTaskActor::create(move |_ctx| Self {
            self_peer_id,
            root,
            state_node_storage,
            network_service,
            wait_2_sync,
            sync_metadata,
            bus,
            syncing_nodes: Mutex::new(HashMap::new()),
        });
        StateSyncTaskRef { address }
    }

    fn exe_task(&mut self, address: Addr<StateSyncTaskActor>) {
        let node_key = self.wait_2_sync.pop_front().unwrap();
        if let Some(state_node) = self.state_node_storage.get(&node_key).unwrap() {
            self.syncing_nodes
                .lock()
                .insert(self.self_peer_id.clone(), node_key.clone());
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
            if let Some(best_peer) = best_peer_info {
                let network_service = self.network_service.clone();
                self.syncing_nodes
                    .lock()
                    .insert(best_peer.get_peer_id(), node_key.clone());
                Arbiter::spawn(async move {
                    sync_state_node(node_key, best_peer.get_peer_id(), network_service, address)
                        .await;
                });
            } else {
                warn!("{:?}", "best peer is none.");
            }
        }
    }

    pub fn reset(&mut self, root: &HashValue) {
        self.wait_2_sync.clear();
        self.root = root.clone();
        self.wait_2_sync.push_back(root.clone());
        self.syncing_nodes.lock().clear();
    }
}

impl Actor for StateSyncTaskActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor started.");
        self.exe_task(ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("StateSyncTaskActor actor stopped.");
    }
}

impl Handler<StateSyncTaskEvent> for StateSyncTaskActor {
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
                                    msg: SystemEvents::SyncDone(),
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

impl Handler<StateSyncEvent> for StateSyncTaskActor {
    type Result = Result<()>;

    /// This method is called for every message received by this actor.
    fn handle(&mut self, msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.reset(&msg.root);
        Ok(())
    }
}
