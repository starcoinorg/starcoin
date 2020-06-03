use crate::helper::{get_body_by_hash, get_headers, get_info_by_hash};
/// Sync message which outbound
use crate::state_sync::StateSyncTaskActor;
use crate::sync_metrics::{LABEL_BLOCK, LABEL_HASH, LABEL_STATE, SYNC_METRICS};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::{format_err, Result};
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use config::NodeConfig;
use crypto::HashValue;
use futures::channel::mpsc;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use parking_lot::RwLock;
use starcoin_storage::Store;
use starcoin_sync_api::sync_messages::{BlockBody, GetBlockHeaders, SyncNotify};
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

        // TODO: why only sync state for main network?
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
                .await?
                .ok_or_else(|| format_err!("Master head is none."))?
                .number();

            if let Some(ancestor_header) = Downloader::<C>::find_ancestor_header(
                downloader.clone(),
                best_peer.get_peer_id(),
                network.clone(),
                begin_number,
                false,
                true,
            )
            .await?
            {
                let ancestor = ancestor_header.number();

                // 2. pivot
                let latest_number = best_peer.get_block_number();
                let latest_block_id = best_peer.get_block_id();
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

                // 3. StateSyncActor
                let root = Self::get_pivot(
                    &network,
                    best_peer.get_peer_id(),
                    (latest_block_id, latest_number),
                    min_behind as usize,
                )
                .await?;
                let sync_pivot = sync_metadata.get_pivot()?;
                let pivot = root.number();
                if !(sync_pivot.is_none() || sync_pivot.expect("sync pivot is none.") < pivot) {
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
                    // TODO: don't update pivot if sync_pivot < pivot?
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
            // TODO: why delay 5s ?
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
        latest_block: (HashValue, BlockNumber),
        step: usize,
    ) -> Result<BlockHeader> {
        let get_headers_req = GetBlockHeaders::new(latest_block.0, step, true, 1);
        let mut headers = get_headers(&network, peer_id.clone(), get_headers_req).await?;
        if let Some(pivot) = headers.pop() {
            let number = latest_block.1 - step as u64;
            if pivot.number() == number {
                Ok(pivot)
            } else {
                Err(format_err!(
                    "pivot number miss match : {:?} , {:?}",
                    pivot.number(),
                    number
                ))
            }
        } else {
            Err(format_err!("{:?}", "pivot header is none."))
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
            if let Some(header) = downloader.chain_reader.clone().master_head_header().await? {
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

                let end_number = best_peer.get_block_number();
                if let Some(ancestor_header) = Downloader::<C>::find_ancestor_header(
                    downloader.clone(),
                    best_peer.get_peer_id(),
                    network.clone(),
                    header.number(),
                    full_mode,
                    head_executed,
                )
                .await?
                {
                    let mut latest_block_id = ancestor_header.id();
                    let mut latest_number = ancestor_header.number();
                    //1. sync hash
                    loop {
                        if end_number <= latest_number {
                            break;
                        }
                        let get_headers_req =
                            Downloader::<C>::get_headers_msg_for_common(latest_block_id);
                        let hash_timer = SYNC_METRICS
                            .sync_done_time
                            .with_label_values(&[LABEL_HASH])
                            .start_timer();
                        let headers =
                            get_headers(&network, best_peer.get_peer_id(), get_headers_req).await?;
                        hash_timer.observe_duration();
                        SYNC_METRICS
                            .sync_total_count
                            .with_label_values(&[LABEL_HASH])
                            .inc_by(headers.len() as i64);

                        let block_timer = SYNC_METRICS
                            .sync_done_time
                            .with_label_values(&[LABEL_BLOCK])
                            .start_timer();
                        if headers.is_empty() {
                            break;
                        } else {
                            let latest_header = headers.last().expect("headers is empty.");
                            latest_block_id = latest_header.id();
                            latest_number = latest_header.number();
                            let hashs: Vec<HashValue> =
                                headers.iter().map(|header| header.id()).collect();
                            let bodies =
                                get_body_by_hash(&network, best_peer.get_peer_id(), hashs.clone())
                                    .await?;
                            SYNC_METRICS
                                .sync_total_count
                                .with_label_values(&[LABEL_BLOCK])
                                .inc_by(bodies.len() as i64);

                            let infos =
                                get_info_by_hash(&network, best_peer.get_peer_id(), hashs).await?;
                            block_timer.observe_duration();
                            SYNC_METRICS
                                .sync_succ_count
                                .with_label_values(&[LABEL_BLOCK])
                                .inc_by(infos.len() as i64);
                            Downloader::do_blocks(downloader.clone(), headers, bodies, infos).await;
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
    chain_reader: ChainActorRef<C>,
    future_blocks: FutureBlockPool,
}

const HEAD_CT: usize = 10;
const MIN_BLOCKS_BEHIND: u64 = 10;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl<C> Downloader<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>) -> Self {
        Downloader {
            chain_reader,
            future_blocks: FutureBlockPool::new(),
        }
    }

    pub fn get_chain_reader(&self) -> ChainActorRef<C> {
        self.chain_reader.clone()
    }

    /// for ancestor
    pub fn get_headers_msg_for_ancestor(block_id: HashValue, step: usize) -> GetBlockHeaders {
        //todo：binary search
        GetBlockHeaders::new(block_id, step, true, HEAD_CT)
    }

    /// for common
    pub fn get_headers_msg_for_common(block_id: HashValue) -> GetBlockHeaders {
        GetBlockHeaders::new(block_id, 1, false, HEAD_CT)
    }

    pub async fn find_ancestor_header(
        downloader: Arc<Downloader<C>>,
        peer_id: PeerId,
        network: NetworkAsyncService,
        block_number: BlockNumber,
        full_mode: bool,
        head_executed: bool,
    ) -> Result<Option<BlockHeader>> {
        let mut ancestor_header = None;
        let peer_info = network
            .get_peer(&peer_id.clone().into())
            .await?
            .ok_or_else(|| format_err!("get peer {:?} not exist.", peer_id))?;
        if peer_info.latest_header.number() <= block_number {
            return Ok(ancestor_header);
        }
        let mut need_executed = if head_executed { false } else { full_mode };
        let mut latest_block_id = peer_info.latest_header.id();
        let mut continue_none = false;
        loop {
            let get_block_headers_req =
                Downloader::<C>::get_headers_msg_for_ancestor(latest_block_id, 1);
            let get_headers = get_headers(&network, peer_id.clone(), get_block_headers_req).await?;
            if !get_headers.is_empty() {
                latest_block_id = get_headers
                    .last()
                    .expect("get_headers is empty.")
                    .clone()
                    .id();
                continue_none = false;
            } else {
                if continue_none {
                    break;
                }
                continue_none = true;
            }

            let (need_executed_new, ancestor) =
                Downloader::do_ancestor(downloader.clone(), get_headers, need_executed).await?;

            need_executed = need_executed_new;

            if ancestor.is_some() {
                ancestor_header = ancestor;
                break;
            }
        }

        if ancestor_header.is_none() {
            return Err(format_err!("{:?}", "find ancestor is none."));
        }
        Ok(ancestor_header)
    }

    pub async fn do_ancestor(
        downloader: Arc<Downloader<C>>,
        block_headers: Vec<BlockHeader>,
        need_executed: bool,
    ) -> Result<(bool, Option<BlockHeader>)> {
        let mut ancestor = None;
        let mut need_executed = need_executed;
        for header in block_headers {
            if let Some(block_state) = downloader
                .chain_reader
                .clone()
                .get_block_state_by_hash(&header.id())
                .await?
            {
                if !need_executed || block_state == BlockState::Executed {
                    if need_executed && block_state == BlockState::Executed {
                        need_executed = false;
                    }
                    ancestor = Some(header);
                    break;
                }
            }
        }

        Ok((need_executed, ancestor))
    }

    pub async fn do_blocks(
        downloader: Arc<Downloader<C>>,
        headers: Vec<BlockHeader>,
        bodies: Vec<BlockBody>,
        infos: Vec<BlockInfo>,
    ) {
        assert_eq!(headers.len(), bodies.len());
        assert_eq!(headers.len(), infos.len());
        for i in 0..headers.len() {
            if let Some(header) = headers.get(i) {
                if let Some(body) = bodies.get(i) {
                    if let Some(info) = infos.get(i) {
                        let block = Block::new(header.clone(), body.clone().transactions);
                        Self::do_block_and_child(downloader.clone(), block, Some(info.clone()))
                            .await;
                    }
                }
            }
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
                .try_connect_with_block_info(
                    block.clone(),
                    block_info.clone().expect("block info can not be none."),
                )
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
                        _ => debug!("Connect block {:?} failed, because : {:?}", block_id, err),
                    }
                }
            }
            Err(e) => error!("Connect block {:?} failed : {:?}", block_id, e),
        }

        false
    }
}
