use crate::download::Downloader;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use actix::{Actor, Addr, Context, Handler};
use anyhow::Result;
use chain::SyncMetadata;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use forkable_jellyfish_merkle::node_type::Node;
use logger::prelude::*;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use starcoin_state_tree::{StateNode, StateNodeStore};
use std::collections::VecDeque;
use std::sync::Arc;
use traits::Consensus;

async fn sync_state_node<E, C>(
    node_key: HashValue,
    downloader: Arc<Downloader<E, C>>,
    network_service: NetworkAsyncService,
    state_node_storage: Arc<dyn StateNodeStore>,
    address: Addr<StateSyncTaskActor<E, C>>,
) where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    debug!("sync_state_node : {:?}", node_key);
    let state_node = match state_node_storage.get(&node_key).unwrap() {
        Some(node) => StateSyncTaskEvent {
            node_key,
            state_node: Some(node),
        },
        None => {
            let best_peer = Downloader::best_peer(downloader.clone()).await.unwrap();
            let get_state_node_by_node_hash_req = RPCRequest::GetStateNodeByNodeHash(node_key);
            if let RPCResponse::GetStateNodeByNodeHash(state_node) = network_service
                .send_request(
                    best_peer.get_peer_id().clone().into(),
                    get_state_node_by_node_hash_req.clone(),
                    do_duration(DELAY_TIME),
                )
                .await
                .unwrap()
            {
                debug!("get_state_node_by_node_hash_resp:{:?}", state_node);
                StateSyncTaskEvent {
                    node_key,
                    state_node: Some(state_node),
                }
            } else {
                warn!("{:?}", "error RPCResponse type.");
                StateSyncTaskEvent {
                    node_key,
                    state_node: None,
                }
            }
        }
    };

    if let Err(err) = address.try_send(state_node) {
        warn!("err:{:?}", err);
    };
}

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct StateSyncTaskEvent {
    node_key: HashValue,
    state_node: Option<StateNode>,
}

pub struct StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    root: HashValue,
    state_node_storage: Arc<dyn StateNodeStore>,
    network_service: NetworkAsyncService,
    downloader: Arc<Downloader<E, C>>,
    wait_2_sync: VecDeque<HashValue>,
    sync_metadata: SyncMetadata,
}

impl<E, C> StateSyncTaskActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        root: HashValue,
        state_node_storage: Arc<dyn StateNodeStore>,
        network_service: NetworkAsyncService,
        downloader: Arc<Downloader<E, C>>,
        sync_metadata: SyncMetadata,
    ) -> Addr<StateSyncTaskActor<E, C>> {
        let mut wait_2_sync: VecDeque<HashValue> = VecDeque::new();
        wait_2_sync.push_back(root.clone());
        StateSyncTaskActor::create(move |_ctx| Self {
            root,
            state_node_storage,
            network_service,
            downloader,
            wait_2_sync,
            sync_metadata,
        })
    }

    fn exe_task(&mut self, address: Addr<StateSyncTaskActor<E, C>>) {
        let node_key = self.wait_2_sync.pop_front().unwrap();
        let downloader = self.downloader.clone();
        let network_service = self.network_service.clone();
        let state_node_storage = self.state_node_storage.clone();
        Arbiter::spawn(async move {
            sync_state_node(
                node_key,
                downloader,
                network_service,
                state_node_storage,
                address,
            )
            .await;
        });
    }

    pub fn _reset(&mut self, root: &HashValue) {
        self.wait_2_sync.clear();
        std::mem::swap(&mut self.root, &mut root.clone());
        self.wait_2_sync.push_back(root.clone());
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
        //1. push back
        let current_node_key = task_event.node_key.clone();
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
                ctx.stop();
            }
        } else {
            self.exe_task(ctx.address());
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
    fn handle(&mut self, _msg: StateSyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        unimplemented!()
    }
}
