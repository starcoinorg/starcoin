use crate::block_connector::BlockConnector;
/// Sync message which outbound
use crate::block_sync::BlockSyncTaskActor;
use crate::helper::{get_headers_by_number, get_headers_msg_for_ancestor, get_headers_with_peer};
use crate::state_sync::StateSyncTaskActor;
use crate::sync_metrics::{LABEL_BLOCK, LABEL_STATE, SYNC_METRICS};
use crate::sync_task::{SyncTask, SyncTaskType};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::{format_err, Result};
use bus::{Broadcast, BusActor, Subscription};
use chain::ChainActorRef;
use config::NodeConfig;
use crypto::HashValue;
use futures::channel::mpsc;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_storage::Store;
use starcoin_sync_api::{BlockBody, GetBlockHeaders, SyncNotify};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use traits::Consensus;
use types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState},
    peer_info::PeerId,
    system_events::{SyncBegin, SyncDone},
};

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub enum SyncEvent {
    DoSync,
}

const _MIN_PEER_SIZE: usize = 5;

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
    ready: Arc<AtomicBool>,
    syncing: Arc<AtomicBool>,
    storage: Arc<dyn Store>,
    sync_task: SyncTask,
    need_sync_state: Arc<AtomicBool>,
    node_config: Arc<NodeConfig>,
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
                sync_task: SyncTask::new_empty(),
                need_sync_state: Arc::new(AtomicBool::new(
                    if node_config.clone().network.disable_seed {
                        false
                    } else {
                        node_config.clone().sync.is_state_sync()
                    },
                )),
                node_config,
            }
        });

        Ok(download_actor)
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

        ctx.run_interval(self.sync_duration, move |download, _ctx| {
            if !download.ready.load(Ordering::Relaxed) {
                return;
            }
            if download.sync_task.is_finish() {
                if let Err(e) = download.sync_event_sender.try_send(SyncEvent::DoSync) {
                    error!("{:?}", e);
                }
            }
        });
    }
}

impl<C> Handler<SyncTaskType> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, task_type: SyncTaskType, _ctx: &mut Self::Context) -> Self::Result {
        self.sync_task.drop_task(&task_type);
        if self.sync_task.is_finish() {
            self.bus.do_send(Broadcast { msg: SyncDone });
            self.need_sync_state.store(false, Ordering::Relaxed);
            self.syncing.store(false, Ordering::Relaxed);
            self.downloader.set_pivot(None);
        }
        Ok(())
    }
}

impl<C> Handler<SyncBegin> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ();

    fn handle(&mut self, _msg: SyncBegin, _ctx: &mut Self::Context) -> Self::Result {
        self.ready.store(true, Ordering::Relaxed);
    }
}

impl<C> Handler<SyncEvent> for DownloadActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, item: SyncEvent, ctx: &mut Self::Context) -> Self::Result {
        match item {
            SyncEvent::DoSync => {
                if !self.sync_task.is_finish() {
                    return Ok(());
                }

                let sync_task = self.sync_task.clone();
                if self.need_sync_state.load(Ordering::Relaxed) {
                    Self::sync_state_and_block(
                        self.self_peer_id.as_ref().clone(),
                        self.node_config.clone().base.net().is_main(),
                        self.downloader.clone(),
                        self.network.clone(),
                        self.storage.clone(),
                        sync_task,
                        self.syncing.clone(),
                        ctx.address(),
                    );
                } else {
                    Self::sync_block_from_best_peer(
                        self.downloader.clone(),
                        self.network.clone(),
                        sync_task,
                        self.syncing.clone(),
                        ctx.address(),
                    );
                }
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

    fn handle(&mut self, msg: SyncNotify, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SyncNotify::NewPeerMsg(peer_id) => {
                self.sync_task.activate_tasks();
                debug!("new peer: {:?}", peer_id);
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
    fn sync_state_and_block(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: Addr<DownloadActor<C>>,
    ) {
        Arbiter::spawn(async move {
            SYNC_METRICS
                .sync_count
                .with_label_values(&[LABEL_STATE])
                .inc();
            syncing.store(true, Ordering::Relaxed);
            match Self::sync_state_and_block_inner(
                self_peer_id.clone(),
                main_network,
                downloader.clone(),
                network.clone(),
                storage.clone(),
                sync_task.clone(),
                download_address.clone(),
            )
            .await
            {
                Err(e) => {
                    debug!("state sync error : {:?}", e);
                    syncing.store(false, Ordering::Relaxed);
                    Self::sync_state_and_block(
                        self_peer_id.clone(),
                        main_network,
                        downloader.clone(),
                        network.clone(),
                        storage.clone(),
                        sync_task,
                        syncing.clone(),
                        download_address,
                    );
                }
                Ok(flag) => {
                    SYNC_METRICS
                        .sync_done_count
                        .with_label_values(&[LABEL_STATE])
                        .inc();
                    if flag {
                        syncing.store(false, Ordering::Relaxed);
                    }
                }
            }
        });
    }

    async fn sync_state_and_block_inner(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        download_address: Addr<DownloadActor<C>>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            //1. ancestor
            let begin_number = downloader
                .chain_reader
                .clone()
                .master_head_header()
                .await?
                .ok_or_else(|| format_err!("Master head is none."))?
                .number();

            if let Some(ancestor_header) = downloader
                .find_ancestor_header(
                    best_peer.get_peer_id(),
                    network.clone(),
                    begin_number,
                    false,
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

                    return Ok(true);
                }

                // 3. sync task
                let root = Downloader::<C>::get_pivot(
                    &network,
                    best_peer.get_peer_id(),
                    (latest_block_id, latest_number),
                    min_behind as usize,
                )
                .await?;

                let block_sync_task = BlockSyncTaskActor::launch(
                    &ancestor_header,
                    latest_number,
                    downloader.clone(),
                    network.clone(),
                    false,
                    download_address.clone(),
                );
                sync_task.push_task(SyncTaskType::BLOCK, Box::new(block_sync_task.clone()));

                let state_sync_task_address = StateSyncTaskActor::launch(
                    self_peer_id,
                    (
                        root.state_root(),
                        root.accumulator_root(),
                        root.parent_block_accumulator_root(),
                    ),
                    storage,
                    network.clone(),
                    block_sync_task,
                    download_address,
                );
                sync_task.push_task(SyncTaskType::STATE, Box::new(state_sync_task_address));
                downloader.set_pivot(Some(root.number()));

            // address
            //     .reset(
            //         root.state_root(),
            //         root.accumulator_root(),
            //         root.parent_block_accumulator_root(),
            //     )
            //     .await;
            } else {
                return Err(format_err!("find_ancestor return none."));
            }
        } else {
            Delay::new(Duration::from_secs(5)).await;
            return Err(format_err!("best peer is none."));
        }

        Ok(false)
    }

    fn sync_block_from_best_peer(
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: Addr<DownloadActor<C>>,
    ) {
        if !syncing.load(Ordering::Relaxed) {
            syncing.store(true, Ordering::Relaxed);
            Arbiter::spawn(async move {
                SYNC_METRICS
                    .sync_count
                    .with_label_values(&[LABEL_BLOCK])
                    .inc();
                match Self::sync_block_from_best_peer_inner(
                    downloader,
                    network,
                    sync_task,
                    download_address,
                )
                .await
                {
                    Err(e) => {
                        error!("sync block from best peer failed : {:?}", e);
                        syncing.store(false, Ordering::Relaxed);
                    }
                    Ok(flag) => {
                        if flag {
                            syncing.store(false, Ordering::Relaxed);
                        }
                    }
                }
            });
        }
    }

    async fn sync_block_from_best_peer_inner(
        downloader: Arc<Downloader<C>>,
        network: NetworkAsyncService,
        sync_task: SyncTask,
        download_address: Addr<DownloadActor<C>>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            if let Some(header) = downloader.chain_reader.clone().master_head_header().await? {
                let end_number = best_peer.get_block_number();
                match downloader
                    .find_ancestor_header(
                        best_peer.get_peer_id(),
                        network.clone(),
                        header.number(),
                        true,
                    )
                    .await
                {
                    Ok(ancestor) => {
                        if let Some(ancestor_header) = ancestor {
                            let block_sync_task = BlockSyncTaskActor::launch(
                                &ancestor_header,
                                end_number,
                                downloader.clone(),
                                network.clone(),
                                true,
                                download_address,
                            );
                            sync_task.push_task(SyncTaskType::BLOCK, Box::new(block_sync_task));
                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    }
                    Err(e) => Err(e),
                }
            } else {
                Err(format_err!(
                    "{:?}",
                    "block header is none when create sync task."
                ))
            }
        } else {
            // Err(format_err!(
            //     "{:?}",
            //     "best peer is none when create sync task."
            // ))
            Ok(true)
        }
    }

    pub fn do_block_and_child(&self, block: Block, block_info: Option<BlockInfo>) {
        let downloader = self.downloader.clone();
        Arbiter::spawn(async move {
            downloader.connect_block_and_child(block, block_info).await;
        });
    }
}

/// Send download message
pub struct Downloader<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<C>,
    block_connector: BlockConnector<C>,
}

const MIN_BLOCKS_BEHIND: u64 = 10;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl<C> Downloader<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>) -> Self {
        Downloader {
            block_connector: BlockConnector::new(chain_reader.clone()),
            chain_reader,
        }
    }

    pub fn get_chain_reader(&self) -> ChainActorRef<C> {
        self.chain_reader.clone()
    }

    pub async fn find_ancestor_header(
        &self,
        peer_id: PeerId,
        network: NetworkAsyncService,
        block_number: BlockNumber,
        is_full_mode: bool,
    ) -> Result<Option<BlockHeader>> {
        let mut ancestor_header = None;
        let peer_info = network
            .get_peer(&peer_id.clone().into())
            .await?
            .ok_or_else(|| format_err!("get peer {:?} not exist.", peer_id))?;
        if peer_info.latest_header.number() <= block_number {
            return Ok(ancestor_header);
        }
        let mut need_executed = is_full_mode;
        let mut latest_block_number = block_number;
        let mut continue_none = false;
        loop {
            let get_block_headers_by_number_req =
                get_headers_msg_for_ancestor(latest_block_number, 1);
            let headers =
                get_headers_by_number(&network, peer_id.clone(), get_block_headers_by_number_req)
                    .await?;
            if !headers.is_empty() {
                latest_block_number = headers
                    .last()
                    .expect("get_headers_by_number is empty.")
                    .clone()
                    .number();
                continue_none = false;
            } else {
                if continue_none {
                    break;
                }
                continue_none = true;
            }

            let (need_executed_new, ancestor) = self.do_ancestor(headers, need_executed).await?;

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
        &self,
        block_headers: Vec<BlockHeader>,
        need_executed: bool,
    ) -> Result<(bool, Option<BlockHeader>)> {
        let mut ancestor = None;
        let mut need_executed = need_executed;
        for header in block_headers {
            if let Some(block_state) = self
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

    async fn get_pivot(
        network: &NetworkAsyncService,
        peer_id: PeerId,
        latest_block: (HashValue, BlockNumber),
        step: usize,
    ) -> Result<BlockHeader> {
        let get_headers_req = GetBlockHeaders::new(latest_block.0, step, true, 1);
        let mut headers = get_headers_with_peer(&network, peer_id, get_headers_req).await?;
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

    pub async fn do_blocks(
        &self,
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
                        self.connect_block_and_child(block, Some(info.clone()))
                            .await;
                    }
                }
            }
        }
    }

    pub async fn connect_block_and_child(&self, block: Block, block_info: Option<BlockInfo>) {
        self.block_connector
            .do_block_and_child(block, block_info)
            .await;
    }

    fn set_pivot(&self, pivot: Option<BlockNumber>) {
        self.block_connector.update_pivot(pivot);
    }
}
