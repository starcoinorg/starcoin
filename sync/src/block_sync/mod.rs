use crate::download::DownloadActor;
use crate::helper::{get_body_by_hash, get_headers, get_headers_msg_for_common};
use crate::sync_metrics::{LABEL_BLOCK_BODY, LABEL_HASH, SYNC_METRICS};
use crate::sync_task::{
    SyncTaskAction, SyncTaskRequest, SyncTaskResponse, SyncTaskState, SyncTaskType,
};
use crate::Downloader;
use actix::prelude::*;
use actix::{Actor, ActorContext, Addr, Context, Handler};
use anyhow::Result;
use crypto::hash::HashValue;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkAsyncService;
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, BlockBody};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use std::time::Duration;
use types::block::{Block, BlockBody as RealBlockBody, BlockHeader, BlockNumber};

const MAX_LEN: usize = 100;
const MAX_SIZE: usize = 10;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct BlockSyncBeginEvent;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct NextTimeEvent;

#[derive(Debug, Clone)]
enum DataType {
    Header,
    Body,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct SyncDataEvent {
    data_type: DataType,
    hashes: Vec<HashValue>,
    headers: Vec<BlockHeader>,
    bodies: Vec<BlockBody>,
}

impl SyncDataEvent {
    fn new_header_event(headers: Vec<BlockHeader>) -> Self {
        SyncDataEvent {
            data_type: DataType::Header,
            hashes: Vec::new(),
            headers,
            bodies: Vec::new(),
        }
    }

    fn new_body_event(bodies: Vec<BlockBody>, hashes: Vec<HashValue>) -> Self {
        SyncDataEvent {
            data_type: DataType::Body,
            hashes,
            headers: Vec::new(),
            bodies,
        }
    }
}

struct BlockSyncTask {
    wait_2_sync: VecDeque<HashValue>,
}

impl BlockSyncTask {
    pub fn new() -> Self {
        BlockSyncTask {
            wait_2_sync: VecDeque::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.wait_2_sync.is_empty()
    }

    fn len(&self) -> usize {
        self.wait_2_sync.len()
    }

    pub fn push_back(&mut self, hash: HashValue) {
        self.wait_2_sync.push_back(hash)
    }

    pub fn push_hashes(&mut self, hashes: Vec<HashValue>) {
        for hash in hashes {
            self.wait_2_sync.push_back(hash)
        }
    }

    fn take_hashes(&mut self) -> Option<Vec<HashValue>> {
        let mut hashes = Vec::new();
        for _ in 0..MAX_SIZE {
            if let Some(hash) = self.wait_2_sync.pop_front() {
                hashes.push(hash);
            } else {
                break;
            }
        }

        if hashes.is_empty() {
            None
        } else {
            Some(hashes)
        }
    }
}

pub struct BlockSyncTaskActor {
    ancestor_number: BlockNumber,
    target_number: BlockNumber,
    next: (HashValue, BlockNumber),
    headers: HashMap<HashValue, BlockHeader>,
    body_task: BlockSyncTask,
    downloader: Arc<Downloader>,
    network: NetworkAsyncService,
    rpc_client: NetworkRpcClient<NetworkAsyncService>,
    state: SyncTaskState,
    download_address: Addr<DownloadActor>,
}

impl Debug for BlockSyncTaskActor {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("BlockSyncTask")
            .field(&self.ancestor_number)
            .field(&self.target_number)
            .field(&self.next.clone())
            .field(&self.headers.len())
            .field(&self.body_task.len())
            .finish()
    }
}

impl BlockSyncTaskActor {
    pub fn launch(
        ancestor_header: &BlockHeader,
        target_number: BlockNumber,
        downloader: Arc<Downloader>,
        network: NetworkAsyncService,
        start: bool,
        download_address: Addr<DownloadActor>,
    ) -> BlockSyncTaskRef {
        debug_assert!(ancestor_header.number() < target_number);
        let address = BlockSyncTaskActor::create(move |_ctx| Self {
            ancestor_number: ancestor_header.number(),
            target_number,
            next: (ancestor_header.id(), ancestor_header.number()),
            headers: HashMap::new(),
            body_task: BlockSyncTask::new(),
            downloader,
            network: network.clone(),
            rpc_client: NetworkRpcClient::new(network),
            state: if start {
                SyncTaskState::Ready
            } else {
                SyncTaskState::NotReady
            },
            download_address,
        });
        BlockSyncTaskRef { address }
    }

    fn do_finish(&mut self) -> bool {
        if !self.state.is_finish() {
            info!("Block sync task info : {:?}", &self);
            if self.next.1 >= self.target_number
                && self.headers.is_empty()
                && self.body_task.is_empty()
            {
                info!("Block sync task finish.");
                self.state = SyncTaskState::Finish;
            }
        }

        self.state.is_finish()
    }

    fn sync_blocks(&mut self, address: Addr<BlockSyncTaskActor>) {
        let sync_header_flag =
            !(self.body_task.len() > MAX_LEN || self.next.1 >= self.target_number);

        let body_hashes = self.body_task.take_hashes();

        let next = self.next.0;
        let network = self.network.clone();
        let rpc_client = self.rpc_client.clone();
        Arbiter::spawn(async move {
            // sync header
            if sync_header_flag {
                let get_headers_req = get_headers_msg_for_common(next);
                let hash_timer = SYNC_METRICS
                    .sync_done_time
                    .with_label_values(&[LABEL_HASH])
                    .start_timer();
                let event = match get_headers(&network, &rpc_client, get_headers_req).await {
                    Ok(headers) => SyncDataEvent::new_header_event(headers),
                    Err(e) => {
                        error!("Sync headers err: {:?}", e);
                        Delay::new(Duration::from_secs(1)).await;
                        SyncDataEvent::new_header_event(Vec::new())
                    }
                };

                address.clone().do_send(event);
                hash_timer.observe_duration();
            }

            // sync body
            if let Some(hashes) = body_hashes {
                let block_body_timer = SYNC_METRICS
                    .sync_done_time
                    .with_label_values(&[LABEL_BLOCK_BODY])
                    .start_timer();
                let event = match get_body_by_hash(&rpc_client, &network, hashes.clone()).await {
                    Ok(bodies) => SyncDataEvent::new_body_event(bodies, Vec::new()),
                    Err(e) => {
                        error!("Sync bodies err: {:?}", e);
                        Delay::new(Duration::from_secs(1)).await;
                        SyncDataEvent::new_body_event(Vec::new(), hashes)
                    }
                };

                address.clone().do_send(event);
                block_body_timer.observe_duration();
            }

            if let Err(err) = address.try_send(NextTimeEvent {}) {
                error!("Send NextTimeEvent failed when sync : {:?}", err);
            };
        });
    }

    fn _sync_headers(&mut self, address: Addr<BlockSyncTaskActor>) {
        if self.body_task.len() > MAX_LEN || self.next.1 >= self.target_number {
            return;
        }

        let next = self.next.0;
        let network = self.network.clone();
        let rpc_client = self.rpc_client.clone();
        Arbiter::spawn(async move {
            let get_headers_req = get_headers_msg_for_common(next);
            let hash_timer = SYNC_METRICS
                .sync_done_time
                .with_label_values(&[LABEL_HASH])
                .start_timer();
            let event = match get_headers(&network, &rpc_client, get_headers_req).await {
                Ok(headers) => SyncDataEvent::new_header_event(headers),
                Err(e) => {
                    error!("Sync headers err: {:?}", e);
                    Delay::new(Duration::from_secs(1)).await;
                    SyncDataEvent::new_header_event(Vec::new())
                }
            };

            address.clone().do_send(event);
            hash_timer.observe_duration();
        });
    }

    fn handle_headers(&mut self, headers: Vec<BlockHeader>) {
        if !headers.is_empty() {
            let len = headers.len();
            for block_header in headers {
                self.body_task.push_back(block_header.id());
                self.next = (block_header.id(), block_header.number());
                self.headers.insert(block_header.id(), block_header);
            }
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_HASH])
                .inc_by(len as i64);
        }
    }

    fn _sync_bodies(&mut self, address: Addr<BlockSyncTaskActor>) {
        if let Some(hashes) = self.body_task.take_hashes() {
            let network = self.network.clone();
            let rpc_client = self.rpc_client.clone();
            Arbiter::spawn(async move {
                let block_body_timer = SYNC_METRICS
                    .sync_done_time
                    .with_label_values(&[LABEL_BLOCK_BODY])
                    .start_timer();
                let event = match get_body_by_hash(&rpc_client, &network, hashes.clone()).await {
                    Ok(bodies) => SyncDataEvent::new_body_event(bodies, Vec::new()),
                    Err(e) => {
                        error!("Sync bodies err: {:?}", e);
                        Delay::new(Duration::from_secs(1)).await;
                        SyncDataEvent::new_body_event(Vec::new(), hashes)
                    }
                };

                address.do_send(event);
                block_body_timer.observe_duration();
            });
        }
    }

    fn handle_bodies(
        &mut self,
        bodies: Vec<BlockBody>,
        hashes: Vec<HashValue>,
    ) -> Option<Box<impl Future<Output = ()>>> {
        if !bodies.is_empty() {
            let len = bodies.len();
            let mut blocks: Vec<Block> = Vec::new();
            for block_body in bodies {
                let block_header = self.headers.remove(&block_body.hash);
                let body = RealBlockBody::new(block_body.transactions, block_body.uncles);
                let block =
                    Block::new_with_body(block_header.expect("block_header is none."), body);
                blocks.push(block);
            }

            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_BLOCK_BODY])
                .inc_by(len as i64);

            Some(self.connect_blocks(blocks))
        } else {
            self.body_task.push_hashes(hashes);
            None
        }
    }

    fn connect_blocks(&self, blocks: Vec<Block>) -> Box<impl Future<Output = ()>> {
        let downloader = self.downloader.clone();
        let fut = async move {
            let mut blocks = blocks;
            blocks.reverse();
            loop {
                let block = blocks.pop();
                if let Some(b) = block {
                    downloader.connect_block_and_child(b).await;
                } else {
                    break;
                }
            }
        };
        Box::new(fut)
    }

    fn block_sync(&mut self, address: Addr<BlockSyncTaskActor>) {
        // self.sync_headers(address.clone());
        // self.sync_bodies(address);
        self.sync_blocks(address);
    }

    fn start_sync_task(&mut self, address: Addr<BlockSyncTaskActor>) {
        self.state = SyncTaskState::Syncing;
        if let Err(err) = address.try_send(NextTimeEvent {}) {
            error!("Send NextTimeEvent failed when start : {:?}", err);
        };
    }
}

impl Actor for BlockSyncTaskActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.state.is_ready() {
            self.start_sync_task(ctx.address());
        }
    }
}

impl Handler<SyncDataEvent> for BlockSyncTaskActor {
    type Result = ();

    fn handle(&mut self, data: SyncDataEvent, ctx: &mut Self::Context) -> Self::Result {
        match data.data_type {
            DataType::Header => {
                self.handle_headers(data.headers);
            }
            DataType::Body => {
                if let Some(fut) = self.handle_bodies(data.bodies, data.hashes) {
                    (*fut)
                        .into_actor(self)
                        .then(|_result, act, _ctx| async {}.into_actor(act))
                        .wait(ctx);
                }
            }
        }
    }
}

impl Handler<NextTimeEvent> for BlockSyncTaskActor {
    type Result = Result<()>;

    fn handle(&mut self, _event: NextTimeEvent, ctx: &mut Self::Context) -> Self::Result {
        let finish = self.do_finish();
        if !finish {
            self.block_sync(ctx.address());
        } else {
            self.download_address.do_send(SyncTaskType::BLOCK);
            ctx.stop();
        }

        Ok(())
    }
}

impl Handler<BlockSyncBeginEvent> for BlockSyncTaskActor {
    type Result = Result<()>;

    fn handle(&mut self, _event: BlockSyncBeginEvent, ctx: &mut Self::Context) -> Self::Result {
        if !self.state.is_ready() {
            self.state = SyncTaskState::Ready;
            self.start_sync_task(ctx.address());
        }

        Ok(())
    }
}

impl Handler<SyncTaskRequest> for BlockSyncTaskActor {
    type Result = Result<SyncTaskResponse>;

    fn handle(&mut self, action: SyncTaskRequest, _ctx: &mut Self::Context) -> Self::Result {
        match action {
            SyncTaskRequest::ACTIVATE() => Ok(SyncTaskResponse::None),
        }
    }
}

#[derive(Clone)]
pub struct BlockSyncTaskRef {
    address: Addr<BlockSyncTaskActor>,
}

impl BlockSyncTaskRef {
    pub fn start(&self) {
        let address = self.address.clone();
        Arbiter::spawn(async move {
            let _ = address.send(BlockSyncBeginEvent {}).await;
        })
    }
}

impl SyncTaskAction for BlockSyncTaskRef {}
