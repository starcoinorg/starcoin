use crate::download::DownloadActor;
use crate::helper::{get_body_by_hash, get_headers, get_headers_msg_for_common};
use crate::sync_event_handle::SendSyncEventHandler;
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
use network_api::{NetworkService, PeerId};
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, BlockBody};
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use std::time::Duration;
use types::block::{Block, BlockBody as RealBlockBody, BlockHeader, BlockNumber};

#[cfg(test)]
mod test_block_sync;

const MAX_LEN: usize = 100;
const MAX_SIZE: usize = 10;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct BlockSyncBeginEvent;

#[derive(Default, Debug, Message)]
#[rtype(result = "()")]
pub struct NextTimeEvent;

#[derive(Debug, Clone)]
enum DataType {
    Header,
    Body,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct BlockIdAndNumber {
    pub id: HashValue,
    pub number: BlockNumber,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct SyncDataEvent {
    data_type: DataType,
    body_taskes: Option<Vec<BlockIdAndNumber>>,
    headers: Option<Vec<BlockHeader>>,
    bodies: Option<Vec<BlockBody>>,
    peer_id: PeerId,
}

impl SyncDataEvent {
    fn new_header_event(headers: Vec<BlockHeader>, peer_id: PeerId) -> Self {
        SyncDataEvent {
            data_type: DataType::Header,
            body_taskes: None,
            headers: Some(headers),
            bodies: None,
            peer_id,
        }
    }

    fn new_body_event(
        bodies: Vec<BlockBody>,
        body_taskes: Vec<BlockIdAndNumber>,
        peer_id: PeerId,
    ) -> Self {
        SyncDataEvent {
            data_type: DataType::Body,
            body_taskes: Some(body_taskes),
            headers: None,
            bodies: Some(bodies),
            peer_id,
        }
    }
}

#[derive(Debug)]
struct BlockSyncTask {
    wait_2_sync: VecDeque<BlockIdAndNumber>,
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

    pub fn push_back(&mut self, height: BlockNumber, id: HashValue) {
        self.wait_2_sync
            .push_back(BlockIdAndNumber { number: height, id })
    }

    pub fn push_tasks(&mut self, hashes: Vec<BlockIdAndNumber>) {
        self.wait_2_sync.extend(hashes);
    }

    fn take_tasks(&mut self) -> Option<Vec<BlockIdAndNumber>> {
        let total_len = self.wait_2_sync.len();
        if total_len == 0 {
            return None;
        }
        Some(self.wait_2_sync.drain(..MAX_SIZE.min(total_len)).collect())
    }
}

pub struct BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    inner: Inner<N>,
    downloader: Arc<Downloader<N>>,
    download_address: Addr<DownloadActor<N>>,
}

pub struct Inner<N>
where
    N: NetworkService + 'static,
{
    ancestor_number: BlockNumber,
    target_number: BlockNumber,
    next: BlockIdAndNumber,
    headers: HashMap<HashValue, BlockHeader>,
    body_task: BlockSyncTask,
    network: N,
    rpc_client: NetworkRpcClient,
    state: SyncTaskState,
}

impl<N> Debug for Inner<N>
where
    N: NetworkService + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Inner")
            .field(&self.ancestor_number)
            .field(&self.target_number)
            .field(&self.next)
            .field(&self.headers.len())
            .field(&self.body_task.len())
            .finish()
    }
}

impl<N> Inner<N>
where
    N: NetworkService + 'static,
{
    fn new(
        ancestor_number: BlockNumber,
        target_number: BlockNumber,
        next: BlockIdAndNumber,
        network: N,
        state: SyncTaskState,
    ) -> Self {
        Self {
            ancestor_number,
            target_number,
            next,
            headers: HashMap::new(),
            body_task: BlockSyncTask::new(),
            network: network.clone(),
            rpc_client: NetworkRpcClient::new(network),
            state,
        }
    }

    fn _header_size(&self) -> usize {
        self.headers.len()
    }

    fn _body_task_size(&self) -> usize {
        self.body_task.len()
    }

    fn do_finish(&mut self) -> bool {
        if !self.state.is_finish() {
            info!("Block sync task info : {:?}", &self);
            if self.next.number >= self.target_number
                && self.headers.is_empty()
                && self.body_task.is_empty()
            {
                info!("Block sync task finish.");
                self.state = SyncTaskState::Finish;
            }
        }

        self.state.is_finish()
    }

    fn sync_blocks(
        &mut self,
        sync_data_handler: Box<dyn SendSyncEventHandler<SyncDataEvent>>,
        next_time_handler: Box<dyn SendSyncEventHandler<NextTimeEvent>>,
    ) {
        let sync_header_flag =
            !(self.body_task.len() > MAX_LEN || self.next.number >= self.target_number);

        let body_tasks = self.body_task.take_tasks();

        let next = self.next.id;
        let next_number = self.next.number;
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

                let event =
                    match get_headers(&network, &rpc_client, get_headers_req, next_number).await {
                        Ok((headers, peer_id)) => SyncDataEvent::new_header_event(headers, peer_id),
                        Err(e) => {
                            error!("Sync headers err: {:?}", e);
                            Delay::new(Duration::from_secs(1)).await;
                            SyncDataEvent::new_header_event(Vec::new(), PeerId::random())
                        }
                    };

                sync_data_handler.send_event(event);
                hash_timer.observe_duration();
            }

            // sync body
            if let Some(tasks) = body_tasks {
                let block_body_timer = SYNC_METRICS
                    .sync_done_time
                    .with_label_values(&[LABEL_BLOCK_BODY])
                    .start_timer();
                debug_assert!(!tasks.is_empty());
                let max_height = tasks
                    .iter()
                    .map(|t| t.number)
                    .max()
                    .expect("body tasks is not empty");
                let block_idlist = tasks.iter().map(|t| t.id).collect();

                let event =
                    match get_body_by_hash(&rpc_client, &network, block_idlist, max_height).await {
                        Ok((bodies, peer_id)) => {
                            SyncDataEvent::new_body_event(bodies, Vec::new(), peer_id)
                        }
                        Err(e) => {
                            error!("Sync bodies err: {:?}", e);
                            Delay::new(Duration::from_secs(1)).await;
                            SyncDataEvent::new_body_event(Vec::new(), tasks, PeerId::random())
                        }
                    };

                sync_data_handler.send_event(event);
                block_body_timer.observe_duration();
            }

            next_time_handler.send_event(NextTimeEvent {});
        });
    }

    fn update_next(&mut self, last_header: &BlockHeader) {
        self.next = BlockIdAndNumber {
            id: last_header.id(),
            number: last_header.number(),
        };
    }

    fn handle_headers(&mut self, headers: Vec<BlockHeader>) {
        if !headers.is_empty() {
            let last_header = headers.last().unwrap();
            self.update_next(last_header);
            let len = headers.len();
            for block_header in headers {
                self.body_task
                    .push_back(block_header.number(), block_header.id());
                self.headers.insert(block_header.id(), block_header);
            }
            SYNC_METRICS
                .sync_total_count
                .with_label_values(&[LABEL_HASH])
                .inc_by(len as i64);
        }
    }

    fn handle_bodies(
        &mut self,
        bodies: Vec<BlockBody>,
        hashes: Vec<BlockIdAndNumber>,
    ) -> Option<Vec<Block>> {
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
            Some(blocks)
        } else {
            self.body_task.push_tasks(hashes);
            None
        }
    }
}

impl<N> Debug for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("BlockSyncTask").field(&self.inner).finish()
    }
}

impl<N> BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    pub fn launch(
        ancestor_header: &BlockHeader,
        target_number: BlockNumber,
        downloader: Arc<Downloader<N>>,
        network: N,
        start: bool,
        download_address: Addr<DownloadActor<N>>,
    ) -> BlockSyncTaskRef<N> {
        debug_assert!(ancestor_header.number() < target_number);
        let inner = Inner::new(
            ancestor_header.number(),
            target_number,
            BlockIdAndNumber {
                id: ancestor_header.id(),
                number: ancestor_header.number(),
            },
            network,
            if start {
                SyncTaskState::Ready
            } else {
                SyncTaskState::NotReady
            },
        );
        let address = BlockSyncTaskActor::create(move |_ctx| Self {
            inner,
            downloader,
            download_address,
        });
        BlockSyncTaskRef { address }
    }

    fn connect_blocks(&self, blocks: Vec<Block>, peer_id: PeerId) {
        let mut blocks = blocks;
        blocks.reverse();
        loop {
            let block = blocks.pop();
            if let Some(b) = block {
                self.downloader.connect_block_and_child(b, peer_id.clone());
            } else {
                break;
            }
        }
    }

    fn block_sync(&mut self, address: Addr<BlockSyncTaskActor<N>>) {
        self.inner
            .sync_blocks(Box::new(address.clone()), Box::new(address));
    }

    fn start_sync_task(&mut self, address: Addr<BlockSyncTaskActor<N>>) {
        self.inner.state = SyncTaskState::Syncing;
        if let Err(err) = address.try_send(NextTimeEvent {}) {
            error!("Send NextTimeEvent failed when start : {:?}", err);
        };
    }
}

impl<N> Actor for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.inner.state.is_ready() {
            self.start_sync_task(ctx.address());
        }
    }
}

impl<N> Handler<SyncDataEvent> for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, data: SyncDataEvent, _ctx: &mut Self::Context) -> Self::Result {
        match data.data_type {
            DataType::Header => {
                if let Some(headers) = data.headers {
                    self.inner.handle_headers(headers);
                }
            }
            DataType::Body => {
                if let (Some(bodies), Some(body_taskes)) = (data.bodies, data.body_taskes) {
                    if let Some(blocks) = self.inner.handle_bodies(bodies, body_taskes) {
                        self.connect_blocks(blocks, data.peer_id);
                    }
                }
            }
        }
    }
}

impl<N> Handler<NextTimeEvent> for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, _event: NextTimeEvent, ctx: &mut Self::Context) -> Self::Result {
        let finish = self.inner.do_finish();
        if !finish {
            self.block_sync(ctx.address());
        } else {
            self.download_address.do_send(SyncTaskType::BLOCK);
            ctx.stop();
        }
    }
}

impl<N> Handler<BlockSyncBeginEvent> for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: BlockSyncBeginEvent, ctx: &mut Self::Context) -> Self::Result {
        if !self.inner.state.is_ready() {
            self.inner.state = SyncTaskState::Ready;
            self.start_sync_task(ctx.address());
        }

        Ok(())
    }
}

impl<N> Handler<SyncTaskRequest> for BlockSyncTaskActor<N>
where
    N: NetworkService + 'static,
{
    type Result = Result<SyncTaskResponse>;

    fn handle(&mut self, action: SyncTaskRequest, _ctx: &mut Self::Context) -> Self::Result {
        match action {
            SyncTaskRequest::ACTIVATE() => Ok(SyncTaskResponse::None),
        }
    }
}

#[derive(Clone)]
pub struct BlockSyncTaskRef<N>
where
    N: NetworkService + 'static,
{
    address: Addr<BlockSyncTaskActor<N>>,
}

impl<N> BlockSyncTaskRef<N>
where
    N: NetworkService + 'static,
{
    pub fn start(&self) {
        let address = self.address.clone();
        Arbiter::spawn(async move {
            let _ = address.send(BlockSyncBeginEvent {}).await;
        })
    }
}

impl<N> SyncTaskAction for BlockSyncTaskRef<N> where N: NetworkService + 'static {}
