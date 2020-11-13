use crate::block_connector::{BlockConnector, PivotBlock};
/// Sync message which outbound
use crate::block_sync::BlockSyncTaskActor;
use crate::helper::{
    get_block_infos, get_headers_by_number, get_headers_msg_for_ancestor, get_headers_with_peer,
};
use crate::state_sync::StateSyncTaskActor;
use crate::sync_metrics::{LABEL_BLOCK, LABEL_STATE, SYNC_METRICS};
use crate::sync_task::{SyncTask, SyncTaskType};
use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::{format_err, Result};
use config::NodeConfig;
use futures_timer::Delay;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::PeerProvider;
use starcoin_chain_service::ChainReaderService;
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetBlockHeaders, RemoteChainStateReader,
};
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_sync_api::SyncNotify;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState},
    peer_info::PeerId,
    startup_info::StartupInfo,
    system_events::{MinedBlock, SystemStarted},
    U256,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use txpool::TxPoolService;

#[derive(Debug, Clone)]
pub enum SyncEvent {
    DoSync,
}

const _MIN_PEER_SIZE: usize = 5;

pub struct DownloadService {
    downloader: Arc<Downloader>,
    self_peer_id: PeerId,
    rpc_client: NetworkRpcClient,
    network: Arc<dyn PeerProvider>,
    sync_duration: Duration,
    ready: Arc<AtomicBool>,
    syncing: Arc<AtomicBool>,
    storage: Arc<dyn Store>,
    sync_task: SyncTask,
    need_sync_state: Arc<AtomicBool>,
    node_config: Arc<NodeConfig>,
}

impl DownloadService {
    pub fn new(
        node_config: Arc<NodeConfig>,
        peer_id: PeerId,
        chain_reader: ServiceRef<ChainReaderService>,
        network: NetworkAsyncService,
        bus: ServiceRef<BusService>,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
        startup_info: StartupInfo,
    ) -> Self {
        Self {
            downloader: Arc::new(Downloader::new(
                chain_reader,
                node_config.clone(),
                startup_info,
                storage.clone(),
                txpool,
                bus,
                None,
            )),
            self_peer_id: peer_id,
            rpc_client: NetworkRpcClient::new(network.clone()),
            network: Arc::new(network),
            sync_duration: Duration::from_secs(5),
            syncing: Arc::new(AtomicBool::new(false)),
            ready: Arc::new(AtomicBool::new(false)),
            storage,
            sync_task: SyncTask::new_empty(),
            need_sync_state: Arc::new(AtomicBool::new(if node_config.network.disable_seed {
                false
            } else {
                node_config.sync.is_state_sync()
            })),
            node_config,
        }
    }
}

impl ServiceFactory<Self> for DownloadService {
    fn create(ctx: &mut ServiceContext<DownloadService>) -> Result<DownloadService> {
        let chain_reader = ctx.service_ref::<ChainReaderService>()?.clone();
        let node_config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let bus = ctx.bus_ref().clone();
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist."))?;
        let network = ctx.get_shared::<NetworkAsyncService>()?;
        let peer_id = node_config.network.self_peer_id()?;
        Ok(Self::new(
            node_config,
            peer_id,
            chain_reader,
            network,
            bus,
            storage,
            txpool,
            startup_info,
        ))
    }
}

impl ActorService for DownloadService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<MinedBlock>();
        ctx.subscribe::<SystemStarted>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<SystemStarted>();
        Ok(())
    }
}

impl EventHandler<Self, SyncTaskType> for DownloadService {
    fn handle_event(
        &mut self,
        task_type: SyncTaskType,
        _ctx: &mut ServiceContext<DownloadService>,
    ) {
        self.sync_task.drop_task(&task_type);
        if self.sync_task.is_finish() {
            //ctx.broadcast(SyncDone);
            self.need_sync_state.store(false, Ordering::Relaxed);
            self.syncing.store(false, Ordering::Relaxed);
            self.downloader.set_pivot(None);
        }
    }
}

impl EventHandler<Self, MinedBlock> for DownloadService {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<DownloadService>) {
        debug!("try connect mined block.");
        let MinedBlock(new_block) = msg;
        match self.downloader.connect_block(new_block.as_ref().clone()) {
            Ok(_) => debug!("Process mined block success."),
            Err(e) => {
                warn!("Process mined block fail, error: {:?}", e);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct CheckDone;

impl EventHandler<Self, CheckDone> for DownloadService {
    fn handle_event(&mut self, _msg: CheckDone, ctx: &mut ServiceContext<DownloadService>) {
        debug!("Check sync is finish.");
        if self.sync_task.is_finish() {
            ctx.notify(SyncEvent::DoSync);
        }
    }
}

impl EventHandler<Self, SystemStarted> for DownloadService {
    fn handle_event(&mut self, _msg: SystemStarted, ctx: &mut ServiceContext<DownloadService>) {
        if !self.ready.load(Ordering::Relaxed) {
            ctx.run_interval(self.sync_duration, move |ctx| {
                ctx.notify(CheckDone);
            });
        }
        self.ready.store(true, Ordering::Relaxed);
        info!("Sync Ready.");
    }
}

impl EventHandler<Self, SyncEvent> for DownloadService {
    fn handle_event(&mut self, item: SyncEvent, ctx: &mut ServiceContext<DownloadService>) {
        match item {
            SyncEvent::DoSync => {
                if !self.sync_task.is_finish() {
                    return;
                }

                let sync_task = self.sync_task.clone();
                if self.need_sync_state.load(Ordering::Relaxed) {
                    let self_peer_id = self.self_peer_id.clone();
                    let is_main = self.node_config.net().is_main();
                    let downloader = self.downloader.clone();
                    let rpc_client = self.rpc_client.clone();
                    let network = self.network.clone();
                    let storage = self.storage.clone();
                    let syncing = self.syncing.clone();
                    let self_ref = ctx.self_ref();
                    ctx.spawn(async move {
                        Self::sync_state_and_block(
                            self_peer_id,
                            is_main,
                            downloader,
                            rpc_client,
                            network,
                            storage,
                            sync_task,
                            syncing,
                            self_ref,
                        )
                        .await;
                    });
                } else {
                    let downloader = self.downloader.clone();
                    let rpc_client = self.rpc_client.clone();
                    let network = self.network.clone();
                    let syncing = self.syncing.clone();
                    let self_ref = ctx.self_ref();
                    ctx.spawn(async move {
                        Self::sync_block_from_best_peer(
                            downloader, rpc_client, network, sync_task, syncing, self_ref,
                        )
                        .await;
                    });
                }
            }
        }
    }
}

impl EventHandler<Self, SyncNotify> for DownloadService {
    fn handle_event(&mut self, msg: SyncNotify, _ctx: &mut ServiceContext<DownloadService>) {
        match msg {
            SyncNotify::NewPeerMsg(peer_id) => {
                self.sync_task.activate_tasks();
                debug!("new peer: {:?}", peer_id);
            }
            SyncNotify::NewHeadBlock(peer_id, block) => self.do_block_and_child(*block, peer_id),
            SyncNotify::ClosePeerMsg(peer_id) => {
                debug!("close peer: {:?}", peer_id);
            }
        }
    }
}

impl DownloadService {
    async fn sync_state_and_block(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader>,
        rpc_client: NetworkRpcClient,
        network: Arc<dyn PeerProvider>,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: ServiceRef<DownloadService>,
    ) {
        SYNC_METRICS
            .sync_count
            .with_label_values(&[LABEL_STATE])
            .inc();
        if !syncing.load(Ordering::Relaxed) {
            syncing.store(true, Ordering::Relaxed);
            match Self::sync_state_and_block_inner(
                self_peer_id.clone(),
                main_network,
                downloader.clone(),
                rpc_client.clone(),
                network.clone(),
                storage.clone(),
                sync_task.clone(),
                download_address.clone(),
            )
            .await
            {
                Err(e) => {
                    error!("state sync error : {:?}, delay and retry.", e);
                    syncing.store(false, Ordering::Relaxed);
                    Delay::new(Duration::from_millis(1000)).await;
                    if let Err(e) = download_address.notify(SyncEvent::DoSync) {
                        error!("Send DoSync event error: {:?}", e);
                    }
                }
                Ok(flag) => {
                    SYNC_METRICS
                        .sync_done_count
                        .with_label_values(&[LABEL_STATE])
                        .inc();
                    if flag {
                        if let Err(e) = download_address.notify(SyncTaskType::STATE) {
                            error!("Notify error: {:?}", e)
                        }
                        syncing.store(false, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    async fn sync_state_and_block_inner(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader>,
        rpc_client: NetworkRpcClient,
        network: Arc<dyn PeerProvider>,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        download_address: ServiceRef<DownloadService>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            //1. ancestor
            let master_header = downloader.chain_reader.clone().master_head_header().await?;
            let begin_number = master_header.number();
            let total_difficulty = downloader
                .chain_reader
                .clone()
                .get_block_info_by_hash(&master_header.id())
                .await?
                .ok_or_else(|| format_err!("Master head block info is none."))?
                .total_difficulty;
            if let Some(ancestor_header) = downloader
                .find_ancestor_header(
                    best_peer.clone(),
                    &rpc_client,
                    begin_number,
                    total_difficulty,
                    false,
                )
                .await?
            {
                let ancestor = ancestor_header.number();

                // 2. pivot
                let latest_header = best_peer.get_latest_header();
                let latest_block_id = latest_header.id();
                let latest_number = latest_header.number();

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
                let (root, block_info) = Downloader::get_pivot(
                    &rpc_client,
                    best_peer.get_peer_id(),
                    (latest_block_id, latest_number),
                    min_behind,
                )
                .await?;
                let peer_selector = network
                    .peer_selector()
                    .await?
                    .selector()
                    .filter_by_block_number(latest_number)
                    .into_selector();
                let verified_rpc_client =
                    VerifiedRpcClient::new_with_client(peer_selector, rpc_client.clone());
                let block_sync_task = BlockSyncTaskActor::launch(
                    &ancestor_header,
                    latest_number,
                    downloader.clone(),
                    false,
                    download_address.clone(),
                    verified_rpc_client.clone(),
                );
                sync_task.push_task(SyncTaskType::BLOCK, Box::new(block_sync_task.clone()));

                let state_sync_task_address = StateSyncTaskActor::launch(
                    self_peer_id,
                    (
                        root.state_root(),
                        root.parent_block_accumulator_root(),
                        root.id(),
                    ),
                    storage.clone(),
                    verified_rpc_client,
                    block_sync_task,
                    download_address,
                );
                sync_task.push_task(
                    SyncTaskType::STATE,
                    Box::new(state_sync_task_address.clone()),
                );
                let pivot_block =
                    PivotBlock::new(root.number(), block_info, state_sync_task_address, storage);
                downloader.set_pivot(Some(pivot_block));

            // address
            //     .reset(
            //         root.state_root(),
            //         root.accumulator_root(),
            //         root.id(),
            //     )
            //     .await;
            } else {
                return Ok(true);
            }
        } else {
            Delay::new(Duration::from_secs(5)).await;
            return Err(format_err!("best peer is none."));
        }

        Ok(false)
    }

    async fn sync_block_from_best_peer(
        downloader: Arc<Downloader>,
        rpc_client: NetworkRpcClient,
        network: Arc<dyn PeerProvider>,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: ServiceRef<DownloadService>,
    ) {
        if !syncing.load(Ordering::Relaxed) {
            syncing.store(true, Ordering::Relaxed);
            SYNC_METRICS
                .sync_count
                .with_label_values(&[LABEL_BLOCK])
                .inc();
            match Self::sync_block_from_best_peer_inner(
                downloader,
                rpc_client,
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
        }
    }

    async fn sync_block_from_best_peer_inner(
        downloader: Arc<Downloader>,
        rpc_client: NetworkRpcClient,
        network: Arc<dyn PeerProvider>,
        sync_task: SyncTask,
        download_address: ServiceRef<DownloadService>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            let header = downloader.chain_reader.clone().master_head_header().await?;
            let end_number = best_peer.get_block_number();
            let total_difficulty = downloader
                .chain_reader
                .clone()
                .get_block_info_by_hash(&header.id())
                .await?
                .ok_or_else(|| format_err!("Master head block info is none."))?
                .total_difficulty;
            match downloader
                .find_ancestor_header(
                    best_peer,
                    &rpc_client,
                    header.number(),
                    total_difficulty,
                    true,
                )
                .await
            {
                Ok(ancestor) => {
                    if let Some(ancestor_header) = ancestor {
                        if ancestor_header.number() >= end_number {
                            return Ok(true);
                        }
                        let peer_selector = network
                            .peer_selector()
                            .await?
                            .selector()
                            .filter_by_block_number(end_number)
                            .into_selector();
                        let verified_rpc_client =
                            VerifiedRpcClient::new_with_client(peer_selector, rpc_client.clone());
                        let block_sync_task = BlockSyncTaskActor::launch(
                            &ancestor_header,
                            end_number,
                            downloader.clone(),
                            true,
                            download_address,
                            verified_rpc_client,
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
            // Err(format_err!(
            //     "best peer is none when create sync task."
            // ))
            Ok(true)
        }
    }

    fn do_block_and_child(&mut self, block: Block, peer_id: PeerId) {
        self.downloader.connect_block_and_child(block, peer_id);
    }
}

/// Send download message
pub struct Downloader {
    chain_reader: ServiceRef<ChainReaderService>,
    block_connector: BlockConnector,
}

const MIN_BLOCKS_BEHIND: u64 = 50;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl Downloader {
    pub fn new(
        chain_reader: ServiceRef<ChainReaderService>,
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
        bus: ServiceRef<BusService>,
        remote_chain_state: Option<RemoteChainStateReader>,
    ) -> Self {
        Downloader {
            block_connector: BlockConnector::new(
                config,
                startup_info,
                storage,
                txpool,
                bus,
                remote_chain_state,
            ),
            chain_reader,
        }
    }

    pub fn get_chain_reader(&self) -> ServiceRef<ChainReaderService> {
        self.chain_reader.clone()
    }

    pub async fn find_ancestor_header(
        &self,
        peer_info: PeerInfo,
        rpc_client: &NetworkRpcClient,
        block_number: BlockNumber,
        total_difficulty: U256,
        is_full_mode: bool,
    ) -> Result<Option<BlockHeader>> {
        let peer_id = peer_info.get_peer_id();
        let mut ancestor_header = None;

        if peer_info.total_difficulty <= total_difficulty {
            return Ok(ancestor_header);
        }
        info!("Sync begin, find ancestor.");
        let mut need_executed = is_full_mode;
        let mut latest_block_number = if block_number > peer_info.latest_header.number() {
            peer_info.latest_header.number()
        } else {
            block_number
        };
        let mut continue_none = false;
        loop {
            let get_block_headers_by_number_req =
                get_headers_msg_for_ancestor(latest_block_number, 1);
            let headers =
                get_headers_by_number(rpc_client, peer_id.clone(), get_block_headers_by_number_req)
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
            return Err(format_err!("find ancestor is none."));
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

    fn verify_pivot(block_header: &BlockHeader, block_info: &BlockInfo) -> bool {
        &block_header.parent_hash() == block_info.block_id()
    }

    async fn get_pivot(
        rpc_client: &NetworkRpcClient,
        peer_id: PeerId,
        latest_block: (HashValue, BlockNumber),
        step: u64,
    ) -> Result<(BlockHeader, BlockInfo)> {
        let get_headers_req = GetBlockHeaders::new(latest_block.0, step, true, 1);
        let mut headers = get_headers_with_peer(
            &rpc_client,
            peer_id.clone(),
            get_headers_req,
            latest_block.1,
        )
        .await?;
        if let Some(pivot) = headers.pop() {
            let number = latest_block.1 - step as u64;
            if pivot.number() == number {
                let mut infos =
                    get_block_infos(&rpc_client, peer_id, vec![pivot.parent_hash()]).await?;
                if let Some(block_info) = infos.pop() {
                    if Self::verify_pivot(&pivot, &block_info) {
                        Ok((pivot, block_info))
                    } else {
                        Err(format_err!(
                            "pivot header and info miss match : {:?} , {:?}",
                            pivot,
                            block_info
                        ))
                    }
                } else {
                    Err(format_err!("pivot block info is none."))
                }
            } else {
                Err(format_err!(
                    "pivot number miss match : {:?} , {:?}",
                    pivot.number(),
                    number
                ))
            }
        } else {
            Err(format_err!("pivot header is none."))
        }
    }

    pub fn do_blocks(&self, headers: Vec<BlockHeader>, bodies: Vec<BlockBody>, peer_id: PeerId) {
        debug_assert_eq!(headers.len(), bodies.len());
        for i in 0..headers.len() {
            if let Some(header) = headers.get(i) {
                if let Some(body) = bodies.get(i) {
                    let block = Block::new(header.clone(), body.clone().transactions);
                    self.connect_block_and_child(block, peer_id.clone());
                }
            }
        }
    }

    pub fn connect_block_and_child(&self, block: Block, peer_id: PeerId) {
        self.block_connector.do_block_and_child(block, peer_id)
    }

    fn connect_block(&self, block: Block) -> Result<()> {
        self.block_connector.connect_block(block)
    }

    fn set_pivot(&self, pivot: Option<PivotBlock>) {
        self.block_connector.update_pivot(pivot);
    }
}
