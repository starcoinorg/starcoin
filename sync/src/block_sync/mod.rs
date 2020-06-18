use crate::download::DownloadActor;
use crate::helper::{get_body_by_hash, get_headers, get_headers_msg_for_common, get_info_by_hash};
use crate::sync_metrics::{LABEL_BLOCK_BODY, LABEL_BLOCK_INFO, LABEL_HASH, SYNC_METRICS};
use crate::sync_task::{
    SyncTaskAction, SyncTaskRequest, SyncTaskResponse, SyncTaskState, SyncTaskType,
};
use crate::Downloader;
use actix::prelude::*;
use actix::{Actor, ActorContext, Addr, Context, Handler};
use anyhow::Result;
use crypto::hash::HashValue;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use traits::Consensus;
use types::block::{Block, BlockHeader, BlockInfo, BlockNumber};

const MAX_LEN: usize = 100;
const MAX_SIZE: usize = 10;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct BlockSyncBeginEvent;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct NextTimeEvent;

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

    pub fn push_hashs(&mut self, hashs: Vec<HashValue>) {
        for hash in hashs {
            self.wait_2_sync.push_back(hash)
        }
    }

    fn take_hashs(&mut self) -> Option<Vec<HashValue>> {
        let mut hashs = Vec::new();
        for _ in 0..MAX_SIZE {
            if let Some(hash) = self.wait_2_sync.pop_front() {
                hashs.push(hash);
            } else {
                break;
            }
        }

        if hashs.is_empty() {
            None
        } else {
            Some(hashs)
        }
    }
}

pub struct BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    ancestor_number: BlockNumber,
    target_number: BlockNumber,
    next: (HashValue, BlockNumber),
    headers: HashMap<HashValue, BlockHeader>,
    info_task: BlockSyncTask,
    infos: HashMap<HashValue, BlockInfo>,
    body_task: BlockSyncTask,
    downloader: Arc<Downloader<C>>,
    network: NetworkAsyncService,
    state: SyncTaskState,
    download_address: Addr<DownloadActor<C>>,
}

impl<C> Debug for BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("BlockSyncTask")
            .field(&self.ancestor_number)
            .field(&self.target_number)
            .field(&self.next.clone())
            .field(&self.headers.len())
            .field(&self.info_task.len())
            .field(&self.infos.len())
            .field(&self.body_task.len())
            .finish()
    }
}

impl<C> BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        ancestor_header: &BlockHeader,
        target_number: BlockNumber,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        start: bool,
        download_address: Addr<DownloadActor<C>>,
    ) -> BlockSyncTaskRef<C> {
        assert!(ancestor_header.number() < target_number);
        let address = BlockSyncTaskActor::create(move |_ctx| Self {
            ancestor_number: ancestor_header.number(),
            target_number,
            next: (ancestor_header.id(), ancestor_header.number()),
            headers: HashMap::new(),
            info_task: BlockSyncTask::new(),
            infos: HashMap::new(),
            body_task: BlockSyncTask::new(),
            downloader,
            network,
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
                && self.info_task.is_empty()
                && self.infos.is_empty()
                && self.body_task.is_empty()
            {
                info!("Block sync task finish.");
                self.state = SyncTaskState::Finish;
            }
        }

        self.state.is_finish()
    }

    async fn sync_headers(&mut self) {
        if self.info_task.len() > MAX_LEN
            || self.body_task.len() > MAX_LEN
            || self.next.1 >= self.target_number
        {
            return;
        }

        let get_headers_req = get_headers_msg_for_common(self.next.0);
        let hash_timer = SYNC_METRICS
            .sync_done_time
            .with_label_values(&[LABEL_HASH])
            .start_timer();
        match get_headers(&self.network, get_headers_req).await {
            Ok(headers) => {
                let len = headers.len();
                for block_header in headers {
                    self.info_task.push_back(block_header.id());
                    self.next = (block_header.id(), block_header.number());
                    self.headers.insert(block_header.id(), block_header);
                }

                SYNC_METRICS
                    .sync_total_count
                    .with_label_values(&[LABEL_HASH])
                    .inc_by(len as i64);
            }
            Err(e) => {
                error!("Sync headers err: {:?}", e);
            }
        }

        hash_timer.observe_duration();
    }

    async fn sync_infos(&mut self) {
        if let Some(hashs) = self.info_task.take_hashs() {
            let block_info_timer = SYNC_METRICS
                .sync_done_time
                .with_label_values(&[LABEL_BLOCK_INFO])
                .start_timer();
            match get_info_by_hash(&self.network, hashs.clone()).await {
                Ok(infos) => {
                    let len = infos.len();
                    for block_info in infos {
                        let block_id = *block_info.block_id();
                        self.body_task.push_back(block_id.clone());
                        self.infos.insert(block_id, block_info);
                    }

                    SYNC_METRICS
                        .sync_total_count
                        .with_label_values(&[LABEL_BLOCK_INFO])
                        .inc_by(len as i64);
                }
                Err(e) => {
                    error!("Sync infos err: {:?}", e);
                    self.info_task.push_hashs(hashs);
                }
            }
            block_info_timer.observe_duration();
        }
    }

    async fn sync_bodies(&mut self) {
        if let Some(hashs) = self.body_task.take_hashs() {
            let block_body_timer = SYNC_METRICS
                .sync_done_time
                .with_label_values(&[LABEL_BLOCK_BODY])
                .start_timer();
            match get_body_by_hash(&self.network, hashs.clone()).await {
                Ok(bodies) => {
                    let len = bodies.len();
                    for block_body in bodies {
                        let (block_id, transactions) = block_body.into();
                        let block_header = self.headers.remove(&block_id);
                        let block_info = self.infos.remove(&block_id);

                        if block_info.is_some() && block_header.is_some() {
                            let block = Block::new(
                                block_header.expect("block_header is none."),
                                transactions,
                            );
                            self.downloader
                                .connect_block_and_child(
                                    block,
                                    Some(block_info.expect("block_info is none.")),
                                )
                                .await;
                        }
                    }

                    SYNC_METRICS
                        .sync_total_count
                        .with_label_values(&[LABEL_BLOCK_BODY])
                        .inc_by(len as i64);
                }
                Err(e) => {
                    error!("Sync bodies err: {:?}", e);
                    self.body_task.push_hashs(hashs);
                }
            }

            block_body_timer.observe_duration();
        }
    }

    async fn block_sync(&mut self, address: Addr<BlockSyncTaskActor<C>>) {
        self.sync_headers().await;
        self.sync_infos().await;
        self.sync_bodies().await;
        if let Err(err) = address.try_send(NextTimeEvent {}) {
            error!("Send NextTimeEvent failed when sync : {:?}", err);
        };
    }

    fn start_sync_task(&mut self, address: Addr<BlockSyncTaskActor<C>>) {
        self.state = SyncTaskState::Syncing;
        if let Err(err) = address.try_send(NextTimeEvent {}) {
            error!("Send NextTimeEvent failed when start : {:?}", err);
        };
    }
}

impl<C> Actor for BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if self.state.is_ready() {
            self.start_sync_task(ctx.address());
        }
    }
}

impl<C> Handler<NextTimeEvent> for BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: NextTimeEvent, ctx: &mut Self::Context) -> Self::Result {
        let finish = self.do_finish();
        if !finish {
            block_on(async { self.block_sync(ctx.address()).await });
        } else {
            self.download_address.do_send(SyncTaskType::BLOCK);
            ctx.stop();
        }

        Ok(())
    }
}

impl<C> Handler<BlockSyncBeginEvent> for BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: BlockSyncBeginEvent, ctx: &mut Self::Context) -> Self::Result {
        if !self.state.is_ready() {
            self.state = SyncTaskState::Ready;
            self.start_sync_task(ctx.address());
        }

        Ok(())
    }
}

impl<C> Handler<SyncTaskRequest> for BlockSyncTaskActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<SyncTaskResponse>;

    fn handle(&mut self, action: SyncTaskRequest, _ctx: &mut Self::Context) -> Self::Result {
        match action {
            SyncTaskRequest::ACTIVATE() => Ok(SyncTaskResponse::None),
        }
    }
}

#[derive(Clone)]
pub struct BlockSyncTaskRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    address: Addr<BlockSyncTaskActor<C>>,
}

impl<C> BlockSyncTaskRef<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn start(&self) {
        let address = self.address.clone();
        Arbiter::spawn(async move {
            let _ = address.send(BlockSyncBeginEvent {}).await;
        })
    }
}

impl<C> SyncTaskAction for BlockSyncTaskRef<C> where C: Consensus + Sync + Send + 'static + Clone {}
