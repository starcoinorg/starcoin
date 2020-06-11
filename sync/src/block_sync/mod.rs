use crate::download::DownloadActor;
use crate::download::SyncEvent;
use crate::helper::{get_body_by_hash, get_headers, get_headers_msg_for_common, get_info_by_hash};
use crate::sync_metrics::{LABEL_BLOCK_BODY, LABEL_BLOCK_INFO, LABEL_HASH, SYNC_METRICS};
use crate::Downloader;
use actix::{Addr, Arbiter};
use crypto::hash::HashValue;
use logger::prelude::*;
use network::NetworkAsyncService;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter, Result};
use std::sync::Arc;
use traits::Consensus;
use types::block::{Block, BlockHeader, BlockInfo, BlockNumber};

const MAX_LEN: usize = 100;
const MAX_SIZE: usize = 10;

struct SyncTask {
    wait_2_sync: VecDeque<HashValue>,
}

impl SyncTask {
    pub fn new() -> Self {
        SyncTask {
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

pub struct BlockSyncTask<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    ancestor_number: BlockNumber,
    target_number: BlockNumber,
    next: (HashValue, BlockNumber),
    headers: Arc<Mutex<HashMap<HashValue, BlockHeader>>>,
    info_task: Arc<Mutex<SyncTask>>,
    infos: Arc<Mutex<HashMap<HashValue, BlockInfo>>>,
    body_task: Arc<Mutex<SyncTask>>,
    downloader: Arc<Downloader<C>>,
    network: NetworkAsyncService,
    download_address: Addr<DownloadActor<C>>,
}

impl<C> Debug for BlockSyncTask<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("BlockSyncTask")
            .field(&self.ancestor_number)
            .field(&self.target_number)
            .field(&self.next)
            .field(&self.headers.lock().len())
            .field(&self.info_task.lock().len())
            .field(&self.infos.lock().len())
            .field(&self.body_task.lock().len())
            .finish()
    }
}

impl<C> BlockSyncTask<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn finish(&self) -> bool {
        info!("Block sync task info : {:?}", &self);
        self.next.1 >= self.target_number
            && self.headers.lock().is_empty()
            && self.info_task.lock().is_empty()
            && self.infos.lock().is_empty()
            && self.body_task.lock().is_empty()
    }

    async fn sync_headers(&mut self) {
        if self.info_task.lock().len() > MAX_LEN
            || self.body_task.lock().len() > MAX_LEN
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
                    self.info_task.lock().push_back(block_header.id());
                    self.next = (block_header.id(), block_header.number());
                    self.headers.lock().insert(block_header.id(), block_header);
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

    async fn sync_infos(&self) {
        let mut info_lock = self.info_task.lock();
        if let Some(hashs) = info_lock.take_hashs() {
            let block_info_timer = SYNC_METRICS
                .sync_done_time
                .with_label_values(&[LABEL_BLOCK_INFO])
                .start_timer();
            match get_info_by_hash(&self.network, hashs.clone()).await {
                Ok(infos) => {
                    let len = infos.len();
                    for block_info in infos {
                        let block_id = *block_info.block_id();
                        self.body_task.lock().push_back(block_id.clone());
                        self.infos.lock().insert(block_id, block_info);
                    }

                    SYNC_METRICS
                        .sync_total_count
                        .with_label_values(&[LABEL_BLOCK_INFO])
                        .inc_by(len as i64);
                }
                Err(e) => {
                    error!("Sync infos err: {:?}", e);
                    info_lock.push_hashs(hashs);
                }
            }
            block_info_timer.observe_duration();
        }
    }

    async fn sync_bodies(&self) {
        let mut body_lock = self.body_task.lock();
        if let Some(hashs) = body_lock.take_hashs() {
            let block_body_timer = SYNC_METRICS
                .sync_done_time
                .with_label_values(&[LABEL_BLOCK_BODY])
                .start_timer();
            match get_body_by_hash(&self.network, hashs.clone()).await {
                Ok(bodies) => {
                    let len = bodies.len();
                    for block_body in bodies {
                        let (block_id, transactions) = block_body.into();
                        let block_header = self.headers.lock().remove(&block_id);
                        let block_info = self.infos.lock().remove(&block_id);

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
                    body_lock.push_hashs(hashs);
                }
            }

            block_body_timer.observe_duration();
        }
    }

    async fn block_sync(&mut self) {
        loop {
            if self.finish() {
                self.download_address.do_send(SyncEvent::BlockSyncDone);
                break;
            }

            self.sync_headers().await;
            self.sync_infos().await;
            self.sync_bodies().await;
        }
    }
}

pub fn do_block_sync_task<C>(
    ancestor_header: &BlockHeader,
    target_number: BlockNumber,
    downloader: Arc<Downloader<C>>,
    network: NetworkAsyncService,
    download_address: Addr<DownloadActor<C>>,
) where
    C: Consensus + Sync + Send + 'static + Clone,
{
    assert!(ancestor_header.number() < target_number);
    let mut task = BlockSyncTask {
        ancestor_number: ancestor_header.number(),
        target_number,
        next: (ancestor_header.id(), ancestor_header.number()),
        headers: Arc::new(Mutex::new(HashMap::new())),
        info_task: Arc::new(Mutex::new(SyncTask::new())),
        infos: Arc::new(Mutex::new(HashMap::new())),
        body_task: Arc::new(Mutex::new(SyncTask::new())),
        downloader,
        network,
        download_address,
    };

    Arbiter::spawn(async move {
        task.block_sync().await;
    });
}
