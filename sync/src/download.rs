/// Sync message which outbound
use crate::pool::TTLPool;
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::{format_err, Result};
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use futures::channel::mpsc;
use parking_lot::RwLock;
// use itertools;
use crate::helper::{get_block_by_hash, get_hash_by_number, get_header_by_hash};
use crate::state_sync::StateSyncTaskActor;
use crate::sync_metrics::{LABEL_BLOCK, LABEL_HASH, LABEL_STATE, SYNC_METRICS};
use config::NodeConfig;
use crypto::HashValue;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_storage::Store;
use starcoin_sync_api::sync_messages::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithNumber, SyncNotify,
};
use starcoin_sync_api::SyncMetadata;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use traits::{is_ok, ConnectBlockError, Consensus};
use types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState},
    peer_info::PeerId,
    system_events::SyncBegin,
};

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub enum SyncEvent {
    DoSync,
    DoPivot(Box<Block>, Box<BlockInfo>),
}

const MIN_PEER_SIZE: usize = 5;

#[derive(Clone)]
pub struct DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<C>>,
    self_peer_id: Arc<PeerId>,
    network: NetworkAsyncService,
    bus: Addr<BusActor>,
    sync_event_sender: mpsc::Sender<SyncEvent>,
    sync_duration: Duration,
    syncing: Arc<AtomicBool>,
    ready: Arc<AtomicBool>,
    storage: Arc<dyn Store>,
    sync_metadata: SyncMetadata,
    main_network: bool,
}

impl<C> DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        peer_id: Arc<PeerId>,
        chain_reader: ChainActorRef<C>,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
        storage: Arc<dyn Store>,
        sync_metadata: SyncMetadata,
    ) -> Result<Addr<DownloadActor<C>>> {
        let download_actor = DownloadActor::create(move |ctx| {
            let (sync_event_sender, sync_event_receiver) = mpsc::channel(100);
            ctx.add_message_stream(sync_event_receiver);
            DownloadActor {
                downloader: Arc::new(Downloader::new(chain_reader)),
                self_peer_id: peer_id,
                network,
                bus,
                sync_event_sender,
                sync_duration: Duration::from_secs(5),
                syncing: Arc::new(AtomicBool::new(false)),
                ready: Arc::new(AtomicBool::new(false)),
                storage,
                sync_metadata,
                main_network: node_config.base.net().is_main(),
            }
        });
        Ok(download_actor)
    }

    fn sync_task(&mut self) {
        if (!self.sync_metadata.fast_sync_mode()
            || (self.sync_metadata.fast_sync_mode() && self.sync_metadata.is_sync_done())
            || (self.sync_metadata.state_syncing()
                && (self.sync_metadata.get_address().is_some() || self.sync_metadata.state_done())))
            && !self.syncing.load(Ordering::Relaxed)
            && self.ready.load(Ordering::Relaxed)
        {
            self.syncing.store(true, Ordering::Relaxed);
            Self::sync_block_from_best_peer(
                self.sync_metadata.clone(),
                self.syncing.clone(),
                self.self_peer_id.as_ref().clone(),
                self.downloader.clone(),
                self.network.clone(),
            );
        }
    }
}

impl<C> Actor for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let sys_event_recipient = ctx.address().recipient::<SyncBegin>();
        self.bus
            .send(Subscription {
                recipient: sys_event_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        ctx.run_interval(self.sync_duration, move |act, _ctx| {
            if !act.syncing.load(Ordering::Relaxed) {
                if let Err(e) = act.sync_event_sender.try_send(SyncEvent::DoSync) {
                    error!("{:?}", e);
                }
            }
        });
    }
}

impl<C> Handler<SyncBegin> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ();

    fn handle(&mut self, _msg: SyncBegin, ctx: &mut Self::Context) -> Self::Result {
        self.ready.store(true, Ordering::Relaxed);

        let downloader = self.downloader.clone();
        let network = self.network.clone();
        let storage = self.storage.clone();
        let sync_metadata = self.sync_metadata.clone();
        let is_main = self.main_network;
        let self_peer_id = self.self_peer_id.as_ref().clone();
        Self::sync_state(
            self_peer_id,
            is_main,
            downloader,
            network,
            storage,
            sync_metadata,
            ctx.address(),
        );
    }
}

impl<C> Handler<SyncEvent> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, item: SyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        match item {
            SyncEvent::DoSync => self.sync_task(),
            SyncEvent::DoPivot(block, block_info) => {
                self.do_block_and_child(*block, Some(*block_info))
            }
        }

        Ok(())
    }
}

impl<C> Handler<SyncNotify> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ();

    fn handle(&mut self, msg: SyncNotify, ctx: &mut Self::Context) -> Self::Result {
        let downloader = self.downloader.clone();
        let network = self.network.clone();
        let storage = self.storage.clone();
        let sync_metadata = self.sync_metadata.clone();
        let is_main = self.main_network;
        let self_peer_id = self.self_peer_id.as_ref().clone();
        let ready = self.ready.load(Ordering::Relaxed);
        match msg {
            SyncNotify::NewPeerMsg(_peer_id) => {
                if ready {
                    Self::sync_state(
                        self_peer_id,
                        is_main,
                        downloader,
                        network,
                        storage,
                        sync_metadata,
                        ctx.address(),
                    );
                }
            }
            SyncNotify::NewHeadBlock(_peer_id, block) => self.do_block_and_child(*block, None),
            SyncNotify::ClosePeerMsg(peer_id) => {
                debug!("close peer: {:?}", peer_id);
            }
        }
    }
}

impl<C> DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn sync_state(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        storage: Arc<dyn Store>,
        sync_metadata: SyncMetadata,
        address: Addr<DownloadActor<C>>,
    ) {
        Arbiter::spawn(async move {
            SYNC_METRICS
                .sync_count
                .with_label_values(&[LABEL_STATE])
                .inc();
            if let Err(e) = Self::sync_state_inner(
                self_peer_id.clone(),
                main_network,
                downloader.clone(),
                network.clone(),
                storage.clone(),
                sync_metadata.clone(),
                address.clone(),
            )
            .await
            {
                debug!("error : {:?}", e);
                Self::sync_state(
                    self_peer_id,
                    main_network,
                    downloader,
                    network,
                    storage,
                    sync_metadata,
                    address,
                );
            } else {
                SYNC_METRICS
                    .sync_done_count
                    .with_label_values(&[LABEL_STATE])
                    .inc();
            }
        });
    }

    async fn sync_state_inner(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        storage: Arc<dyn Store>,
        sync_metadata: SyncMetadata,
        address: Addr<DownloadActor<C>>,
    ) -> Result<()> {
        if !sync_metadata.state_syncing() {
            debug!("not fast sync mode.");
            return Ok(());
        }

        if sync_metadata.state_done() {
            debug!("state sync already done.");
            return Ok(());
        }

        if main_network && network.get_peer_set_size().await? < MIN_PEER_SIZE {
            debug!("condition is not satisfied when sync state.");
            return Ok(());
        }

        if let Some(best_peer) = network.best_peer().await? {
            //1. ancestor
            let begin_number = downloader
                .chain_reader
                .clone()
                .master_head_header()
                .await
                .unwrap()
                .number();

            if let Some(hash_with_number) = Downloader::find_ancestor(
                downloader.clone(),
                best_peer.get_peer_id(),
                network.clone(),
                begin_number,
                false,
                true,
            )
            .await?
            {
                let ancestor = hash_with_number.number;

                // 2. pivot
                let latest_number = best_peer.get_block_number();
                let min_behind = if main_network {
                    MAIN_MIN_BLOCKS_BEHIND
                } else {
                    MIN_BLOCKS_BEHIND
                };
                if (ancestor + min_behind) > latest_number {
                    debug!(
                        "do not need sync state : {:?}, {:?}, {:?}",
                        ancestor, min_behind, latest_number
                    );
                    if sync_metadata.get_address().is_none() {
                        let _ = sync_metadata.state_sync_done();
                        let _ = sync_metadata.pivot_connected_succ();
                    }
                    return Ok(());
                }
                let pivot = latest_number - min_behind;

                // 3. StateSyncActor
                let root = Self::get_pivot(&network, best_peer.get_peer_id(), pivot).await?;
                let sync_pivot = sync_metadata.get_pivot()?;
                if !(sync_pivot.is_none() || sync_pivot.unwrap() < pivot) {
                    debug!("pivot {:?} : {}", sync_pivot, pivot);
                    return Ok(());
                }

                if sync_metadata.state_done() {
                    debug!("state sync already done during find_ancestor.");
                    return Ok(());
                }

                if sync_pivot.is_none() {
                    sync_metadata.clone().update_pivot(pivot, min_behind)?;
                    let state_sync_task_address = StateSyncTaskActor::launch(
                        self_peer_id,
                        (
                            root.state_root(),
                            root.accumulator_root(),
                            root.parent_block_accumulator_root(),
                        ),
                        storage,
                        network.clone(),
                        sync_metadata.clone(),
                        address,
                    );
                    sync_metadata.update_address(&state_sync_task_address)?
                } else if let Some(_tmp) = sync_pivot {
                    // if tmp < pivot {
                    //     if let Some(address) = sync_metadata.get_address() {
                    //         address
                    //             .reset(
                    //                 root.state_root(),
                    //                 root.accumulator_root(),
                    //                 root.parent_block_accumulator_root(),
                    //             )
                    //             .await;
                    //     } else {
                    //         debug!("state sync reset address is none.");
                    //     }
                    // }
                }
            } else {
                return Err(format_err!("find_ancestor return none."));
            }
        } else {
            Delay::new(Duration::from_secs(5)).await;
            return Err(format_err!("best peer is none."));
        }

        if sync_metadata.is_failed() {
            if let Some(address) = sync_metadata.get_address() {
                address.act().await;
            }
        }

        Ok(())
    }

    async fn get_pivot(
        network: &NetworkAsyncService,
        peer_id: PeerId,
        pivot: BlockNumber,
    ) -> Result<BlockHeader> {
        // 1. pivot hash
        let mut numbers: Vec<BlockNumber> = Vec::new();
        numbers.push(pivot);
        let mut batch_hash_by_number_msg =
            get_hash_by_number(&network, peer_id.clone(), GetHashByNumberMsg { numbers }).await?;
        if let Some(hash_number) = batch_hash_by_number_msg.hashs.pop() {
            // 2. pivot header
            let mut hashs = Vec::new();
            hashs.push(hash_number.hash);
            let mut headers = get_header_by_hash(&network, peer_id.clone(), hashs).await?;
            if let Some(header) = headers.headers.pop() {
                Ok(header)
            } else {
                Err(format_err!("{:?}", "pivot header is none."))
            }
        } else {
            Err(format_err!("{:?}", "pivot hash is none."))
        }
    }

    fn sync_block_from_best_peer(
        sync_metadata: SyncMetadata,
        syncing: Arc<AtomicBool>,
        _self_peer_id: PeerId,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
    ) {
        Arbiter::spawn(async move {
            SYNC_METRICS
                .sync_count
                .with_label_values(&[LABEL_BLOCK])
                .inc();
            let full_mode = sync_metadata.state_syncing();
            if let Err(e) =
                Self::sync_block_from_best_peer_inner(downloader, network, full_mode).await
            {
                error!("sync block from best peer failed : {:?}", e);
            } else {
                let _ = sync_metadata.block_sync_done();
                SYNC_METRICS
                    .sync_done_count
                    .with_label_values(&[LABEL_BLOCK])
                    .inc();
            };

            syncing.store(false, Ordering::Relaxed);
        });
    }

    async fn sync_block_from_best_peer_inner(
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        full_mode: bool,
    ) -> Result<()> {
        if let Some(best_peer) = network.best_peer().await? {
            if let Some(header) = downloader.chain_reader.clone().master_head_header().await {
                let mut begin_number = header.number();
                let head_executed = if let Some(head_state) = downloader
                    .chain_reader
                    .clone()
                    .get_block_state_by_hash(&header.id())
                    .await?
                {
                    head_state == BlockState::Executed
                } else {
                    false
                };

                if let Some(hash_number) = Downloader::find_ancestor(
                    downloader.clone(),
                    best_peer.get_peer_id(),
                    network.clone(),
                    begin_number,
                    full_mode,
                    head_executed,
                )
                .await?
                {
                    begin_number = hash_number.number + 1;
                    //1. sync hash
                    while let Some((get_hash_by_number_msg, end, next_number)) =
                        Downloader::<C>::get_hash_by_number_msg_forward(
                            network.clone(),
                            best_peer.get_peer_id(),
                            begin_number,
                        )
                        .await?
                    {
                        begin_number = next_number;
                        SYNC_METRICS
                            .sync_total_count
                            .with_label_values(&[LABEL_HASH])
                            .inc_by(get_hash_by_number_msg.numbers.len() as i64);
                        let hash_timer = SYNC_METRICS
                            .sync_done_time
                            .with_label_values(&[LABEL_HASH])
                            .start_timer();
                        let batch_hash_by_number_msg = get_hash_by_number(
                            &network,
                            best_peer.get_peer_id(),
                            get_hash_by_number_msg,
                        )
                        .await?;
                        hash_timer.observe_duration();

                        Downloader::put_hash_2_hash_pool(
                            downloader.clone(),
                            best_peer.get_peer_id(),
                            batch_hash_by_number_msg,
                        );

                        let hashs = Downloader::take_task_from_hash_pool(downloader.clone());
                        if !hashs.is_empty() {
                            SYNC_METRICS
                                .sync_total_count
                                .with_label_values(&[LABEL_BLOCK])
                                .inc_by(hashs.len() as i64);
                            let block_timer = SYNC_METRICS
                                .sync_done_time
                                .with_label_values(&[LABEL_BLOCK])
                                .start_timer();
                            let (headers, bodies, infos) =
                                get_block_by_hash(&network, best_peer.get_peer_id(), hashs).await?;
                            block_timer.observe_duration();
                            SYNC_METRICS
                                .sync_succ_count
                                .with_label_values(&[LABEL_BLOCK])
                                .inc_by(headers.headers.len() as i64);
                            Downloader::do_blocks(
                                downloader.clone(),
                                headers.headers,
                                bodies.bodies,
                                infos.infos,
                            )
                            .await;
                        } else {
                            debug!("{:?}", "hash pool is empty.");
                        }

                        if end {
                            break;
                        }
                    }
                }
            } else {
                return Err(format_err!("{:?}", "block header is none."));
            }
        } else {
            //return Err(format_err!("{:?}", "best peer is none."));
            debug!("{:?}", "best peer is none when sync block.");
        }

        Ok(())
    }

    pub fn do_block_and_child(&self, block: Block, block_info: Option<BlockInfo>) {
        let downloader = self.downloader.clone();
        Arbiter::spawn(async move {
            Downloader::do_block_and_child(downloader, block, block_info).await;
        });
    }
}

struct FutureBlockPool {
    child: Arc<RwLock<HashMap<HashValue, HashSet<HashValue>>>>,
    blocks: Arc<RwLock<HashMap<HashValue, (Block, Option<BlockInfo>)>>>,
}

impl FutureBlockPool {
    pub fn new() -> Self {
        FutureBlockPool {
            child: Arc::new(RwLock::new(HashMap::new())),
            blocks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_future_block(&self, block: Block, block_info: Option<BlockInfo>) {
        let block_id = block.header().id();
        let parent_id = block.header().parent_hash();
        if !self.blocks.read().contains_key(&block_id) {
            self.blocks.write().insert(block_id, (block, block_info));
        }
        let mut lock = self.child.write();
        if lock.contains_key(&parent_id) {
            lock.get_mut(&parent_id)
                .expect("parent not exist.")
                .insert(block_id);
        } else {
            let mut child = HashSet::new();
            child.insert(block_id);
            lock.insert(parent_id, child);
        }
    }

    fn descendants(&self, parent_id: &HashValue) -> Vec<HashValue> {
        let mut child = Vec::new();
        let lock = self.child.read();
        if lock.contains_key(parent_id) {
            lock.get(parent_id)
                .expect("parent not exist.")
                .iter()
                .for_each(|id| {
                    child.push(*id);
                });

            if !child.is_empty() {
                child.clone().iter().for_each(|new_parent_id| {
                    let mut new_child = self.descendants(new_parent_id);
                    if !new_child.is_empty() {
                        child.append(&mut new_child);
                    }
                })
            }
        };

        child
    }

    pub fn take_child(&self, parent_id: &HashValue) -> Option<Vec<(Block, Option<BlockInfo>)>> {
        let descendants = self.descendants(parent_id);
        if !descendants.is_empty() {
            let mut child = Vec::new();
            let mut child_lock = self.child.write();
            let mut block_lock = self.blocks.write();
            descendants.iter().for_each(|id| {
                let _ = child_lock.remove(id);
                if let Some((block, block_info)) = block_lock.remove(id) {
                    child.push((block, block_info));
                }
            });
            Some(child)
        } else {
            None
        }
    }
}

/// Send download message
pub struct Downloader<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    hash_pool: TTLPool<HashWithNumber>,
    _header_pool: TTLPool<BlockHeader>,
    _body_pool: TTLPool<BlockBody>,
    chain_reader: ChainActorRef<C>,
    future_blocks: FutureBlockPool,
}

const HEAD_CT: u64 = 10;
const MIN_BLOCKS_BEHIND: u64 = 10;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl<C> Downloader<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>) -> Self {
        Downloader {
            hash_pool: TTLPool::new(),
            _header_pool: TTLPool::new(),
            _body_pool: TTLPool::new(),
            chain_reader,
            future_blocks: FutureBlockPool::new(),
        }
    }

    pub fn get_chain_reader(&self) -> ChainActorRef<C> {
        self.chain_reader.clone()
    }

    /// for ancestors
    pub async fn get_hash_by_number_msg_backward(
        network: NetworkAsyncService,
        peer_id: PeerId,
        begin_number: u64,
    ) -> Result<Option<(GetHashByNumberMsg, bool, u64)>> {
        //todo：binary search
        if let Some(peer_info) = network.get_peer(&peer_id.clone().into()).await? {
            let number = peer_info.get_block_number();

            if begin_number < number {
                let mut numbers = Vec::new();
                let mut end = false;
                let mut next_number = 0;
                if begin_number < HEAD_CT {
                    for i in 0..(begin_number + 1) {
                        numbers.push(i);
                        end = true;
                    }
                } else {
                    for i in (begin_number - HEAD_CT + 1)..(begin_number + 1) {
                        numbers.push(i);
                    }
                    next_number = begin_number - HEAD_CT;
                };
                Ok(Some((GetHashByNumberMsg { numbers }, end, next_number)))
            } else {
                Ok(None)
            }
        } else {
            Err(format_err!("peer {:?} not exist.", peer_id))
        }
    }

    pub async fn get_hash_by_number_msg_forward(
        network: NetworkAsyncService,
        peer_id: PeerId,
        begin_number: u64,
    ) -> Result<Option<(GetHashByNumberMsg, bool, u64)>> {
        if let Some(peer_info) = network.get_peer(&peer_id.clone().into()).await? {
            let number = peer_info.get_block_number();
            if begin_number < number {
                let mut numbers = Vec::new();
                let mut end = false;
                let next_number = if (number - begin_number) < HEAD_CT {
                    for i in begin_number..(number + 1) {
                        numbers.push(i);
                        end = true;
                    }
                    number + 1
                } else {
                    for i in begin_number..(begin_number + HEAD_CT) {
                        numbers.push(i);
                    }
                    begin_number + HEAD_CT
                };

                Ok(Some((GetHashByNumberMsg { numbers }, end, next_number)))
            } else {
                Ok(None)
            }
        } else {
            Err(format_err!("peer {:?} not exist.", peer_id))
        }
    }

    pub async fn find_ancestor(
        downloader: Arc<Downloader<C>>,
        peer_id: PeerId,
        network: NetworkAsyncService,
        block_number: BlockNumber,
        full_mode: bool,
        head_executed: bool,
    ) -> Result<Option<HashWithNumber>> {
        let mut hash_with_number = None;
        let mut begin_number = block_number;
        let need_executed = if head_executed { false } else { full_mode };
        while let Some((get_hash_by_number_msg, end, next_number)) =
            Downloader::<C>::get_hash_by_number_msg_backward(
                network.clone(),
                peer_id.clone(),
                begin_number,
            )
            .await?
        {
            begin_number = next_number;
            let batch_hash_by_number_msg =
                get_hash_by_number(&network, peer_id.clone(), get_hash_by_number_msg).await?;
            hash_with_number = Downloader::do_ancestor(
                downloader.clone(),
                peer_id.clone(),
                batch_hash_by_number_msg,
                need_executed,
            )
            .await;

            if end || hash_with_number.is_some() {
                if end && hash_with_number.is_none() {
                    return Err(format_err!("{:?}", "find ancestor is none."));
                }
                break;
            }
        }

        Ok(hash_with_number)
    }

    pub async fn do_ancestor(
        downloader: Arc<Downloader<C>>,
        peer: PeerId,
        batch_hash_by_number_msg: BatchHashByNumberMsg,
        need_executed: bool,
    ) -> Option<HashWithNumber> {
        let mut exist_ancestor = false;
        let mut ancestor = None;
        let mut hashs = batch_hash_by_number_msg.hashs.clone();
        let mut not_exist_hash = Vec::new();
        hashs.reverse();
        for hash in hashs {
            if let Some(block_state) = downloader
                .chain_reader
                .clone()
                .get_block_state_by_hash(&hash.hash)
                .await
                .unwrap()
            {
                if !need_executed || block_state == BlockState::Executed {
                    exist_ancestor = true;
                    ancestor = Some(hash);
                    break;
                }
            } else {
                not_exist_hash.push(hash);
            }
        }

        if exist_ancestor {
            for hash in not_exist_hash {
                downloader.hash_pool.insert(peer.clone(), hash.number, hash);
            }
        }
        ancestor
    }

    pub fn put_hash_2_hash_pool(
        downloader: Arc<Downloader<C>>,
        peer: PeerId,
        batch_hash_by_number_msg: BatchHashByNumberMsg,
    ) {
        for hash in batch_hash_by_number_msg.hashs {
            downloader.hash_pool.insert(peer.clone(), hash.number, hash);
            SYNC_METRICS
                .sync_succ_count
                .with_label_values(&[LABEL_HASH])
                .inc();
        }
    }

    pub fn take_task_from_hash_pool(downloader: Arc<Downloader<C>>) -> Vec<HashValue> {
        let hash_vec = downloader.hash_pool.take(100);
        if !hash_vec.is_empty() {
            hash_vec.iter().map(|hash| hash.hash).collect()
        } else {
            Vec::new()
        }
    }

    pub async fn _put_header_2_header_pool(
        downloader: Arc<Downloader<C>>,
        peer: PeerId,
        batch_header_msg: BatchHeaderMsg,
    ) {
        if !batch_header_msg.headers.is_empty() {
            for header in batch_header_msg.headers {
                downloader
                    ._header_pool
                    .insert(peer.clone(), header.number(), header);
            }
        }
    }

    pub async fn _take_task_from_header_pool(
        downloader: Arc<Downloader<C>>,
    ) -> Option<GetDataByHashMsg> {
        let header_vec = downloader._header_pool.take(100);
        if !header_vec.is_empty() {
            let hashs = header_vec.iter().map(|header| header.id()).collect();
            Some(GetDataByHashMsg {
                hashs,
                data_type: DataType::BODY,
            })
        } else {
            None
        }
    }

    pub async fn do_blocks(
        downloader: Arc<Downloader<C>>,
        headers: Vec<BlockHeader>,
        bodies: Vec<BlockBody>,
        infos: Vec<BlockInfo>,
    ) {
        assert_eq!(headers.len(), bodies.len());
        for i in 0..headers.len() {
            let block = Block::new(
                headers.get(i).unwrap().clone(),
                bodies.get(i).unwrap().clone().transactions,
            );
            let block_info = infos.get(i).unwrap().clone();
            Self::do_block_and_child(downloader.clone(), block, Some(block_info)).await;
        }
    }

    pub async fn do_block_and_child(
        downloader: Arc<Downloader<C>>,
        block: Block,
        block_info: Option<BlockInfo>,
    ) {
        let block_id = block.header().id();
        if Self::do_block(downloader.clone(), block, block_info).await {
            if let Some(child) = downloader.future_blocks.take_child(&block_id) {
                for (son_block, son_block_info) in child {
                    let _ = Self::do_block(downloader.clone(), son_block, son_block_info).await;
                }
            }
        }
    }

    async fn do_block(
        downloader: Arc<Downloader<C>>,
        block: Block,
        block_info: Option<BlockInfo>,
    ) -> bool {
        let connect_result = if block_info.is_some() {
            downloader
                .chain_reader
                .clone()
                .try_connect_with_block_info(block.clone(), block_info.clone().unwrap())
                .await
        } else {
            downloader
                .chain_reader
                .clone()
                .try_connect(block.clone())
                .await
        };

        let block_id = block.id();
        match connect_result {
            Ok(connect) => {
                if is_ok(&connect) {
                    return true;
                } else if let Err(err) = connect {
                    match err {
                        ConnectBlockError::FutureBlock => {
                            downloader.future_blocks.add_future_block(block, block_info)
                        }
                        _ => debug!("Connect block {:?} succ, but : {:?}", block_id, err),
                    }
                }
            }
            Err(e) => error!("Connect block {:?} failed : {:?}", block_id, e),
        }

        false
    }
}
