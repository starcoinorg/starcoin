use crate::block_connector::{BlockConnector, PivotBlock};
/// Sync message which outbound
use crate::block_sync::BlockSyncTaskActor;
use crate::helper::{
    get_headers_by_number, get_headers_msg_for_ancestor, get_headers_with_peer, get_info_by_hash,
};
use crate::state_sync::StateSyncTaskActor;
use crate::sync_metrics::{LABEL_BLOCK, LABEL_STATE, SYNC_METRICS};
use crate::sync_task::{SyncTask, SyncTaskType};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::{format_err, Result};
use bus::{Broadcast, BusActor, Subscription};
use config::NodeConfig;
use crypto::HashValue;
use futures::channel::mpsc;
use futures_timer::Delay;
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_chain_service::ChainReaderService;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetBlockHeaders, RemoteChainStateReader,
};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::Store;
use starcoin_sync_api::SyncNotify;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use txpool::TxPoolService;
use types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState},
    peer_info::PeerId,
    startup_info::StartupInfo,
    system_events::{MinedBlock, SyncDone, SystemStarted},
    U256,
};

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub enum SyncEvent {
    DoSync,
}

const _MIN_PEER_SIZE: usize = 5;

pub struct DownloadActor<N>
where
    N: NetworkService + 'static,
{
    downloader: Arc<Downloader<N>>,
    self_peer_id: Arc<PeerId>,
    rpc_client: NetworkRpcClient,
    network: N,
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

impl<N> DownloadActor<N>
where
    N: NetworkService + 'static,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        peer_id: Arc<PeerId>,
        chain_reader: ServiceRef<ChainReaderService>,
        network: N,
        bus: Addr<BusActor>,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
        startup_info: StartupInfo,
    ) -> Result<Addr<DownloadActor<N>>> {
        let download_actor = DownloadActor::create(move |ctx| {
            let (sync_event_sender, sync_event_receiver) = mpsc::channel(100);
            ctx.add_message_stream(sync_event_receiver);
            DownloadActor {
                downloader: Arc::new(Downloader::new(
                    chain_reader,
                    node_config.clone(),
                    startup_info,
                    storage.clone(),
                    txpool,
                    bus.clone(),
                    None,
                )),
                self_peer_id: peer_id,
                rpc_client: NetworkRpcClient::new(network.clone()),
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

impl<N> Actor for DownloadActor<N>
where
    N: NetworkService + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(1024);
        let recipient = ctx.address().recipient::<MinedBlock>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        let sys_event_recipient = ctx.address().recipient::<SystemStarted>();
        self.bus
            .send(Subscription {
                recipient: sys_event_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

impl<N> Handler<SyncTaskType> for DownloadActor<N>
where
    N: NetworkService + 'static,
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

impl<N> Handler<MinedBlock> for DownloadActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: MinedBlock, _ctx: &mut Self::Context) -> Self::Result {
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

impl<N> Handler<SystemStarted> for DownloadActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, _msg: SystemStarted, ctx: &mut Self::Context) -> Self::Result {
        if !self.ready.load(Ordering::Relaxed) {
            ctx.run_interval(self.sync_duration, move |download, _ctx| {
                debug!("Send sync event.");
                if download.sync_task.is_finish() {
                    if let Err(e) = download.sync_event_sender.try_send(SyncEvent::DoSync) {
                        error!("{:?}", e);
                    }
                }
            });
        }
        self.ready.store(true, Ordering::Relaxed);
        info!("Sync Ready.");
    }
}

impl<N> Handler<SyncEvent> for DownloadActor<N>
where
    N: NetworkService + 'static,
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
                        self.rpc_client.clone(),
                        self.network.clone(),
                        self.storage.clone(),
                        sync_task,
                        self.syncing.clone(),
                        ctx.address(),
                    );
                } else {
                    Self::sync_block_from_best_peer(
                        self.downloader.clone(),
                        self.rpc_client.clone(),
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

impl<N> Handler<SyncNotify> for DownloadActor<N>
where
    N: NetworkService + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: SyncNotify, _ctx: &mut Self::Context) -> Self::Result {
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

impl<N> DownloadActor<N>
where
    N: NetworkService + 'static,
{
    fn sync_state_and_block(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<N>>,
        rpc_client: NetworkRpcClient,
        network: N,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: Addr<DownloadActor<N>>,
    ) {
        Arbiter::spawn(async move {
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
                        error!("state sync error : {:?}", e);
                        syncing.store(false, Ordering::Relaxed);
                        Self::sync_state_and_block(
                            self_peer_id.clone(),
                            main_network,
                            downloader.clone(),
                            rpc_client,
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
                            download_address.do_send(SyncTaskType::STATE);
                            syncing.store(false, Ordering::Relaxed);
                        }
                    }
                }
            }
        });
    }

    async fn sync_state_and_block_inner(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<N>>,
        rpc_client: NetworkRpcClient,
        network: N,
        storage: Arc<dyn Store>,
        sync_task: SyncTask,
        download_address: Addr<DownloadActor<N>>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            //1. ancestor
            let master_header = downloader
                .chain_reader
                .clone()
                .master_head_header()
                .await?
                .ok_or_else(|| format_err!("Master head is none."))?;
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
                    best_peer.get_peer_id(),
                    &rpc_client,
                    network.clone(),
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
                let (root, block_info) = Downloader::<N>::get_pivot(
                    &rpc_client,
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
                        root.parent_block_accumulator_root(),
                        root.id(),
                    ),
                    storage.clone(),
                    network.clone(),
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

    fn sync_block_from_best_peer(
        downloader: Arc<Downloader<N>>,
        rpc_client: NetworkRpcClient,
        network: N,
        sync_task: SyncTask,
        syncing: Arc<AtomicBool>,
        download_address: Addr<DownloadActor<N>>,
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
            });
        }
    }

    async fn sync_block_from_best_peer_inner(
        downloader: Arc<Downloader<N>>,
        rpc_client: NetworkRpcClient,
        network: N,
        sync_task: SyncTask,
        download_address: Addr<DownloadActor<N>>,
    ) -> Result<bool> {
        if let Some(best_peer) = network.best_peer().await? {
            if let Some(header) = downloader.chain_reader.clone().master_head_header().await? {
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
                        best_peer.get_peer_id(),
                        &rpc_client,
                        network.clone(),
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
                Err(format_err!("block header is none when create sync task."))
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
pub struct Downloader<N>
where
    N: NetworkService + 'static,
{
    chain_reader: ServiceRef<ChainReaderService>,
    block_connector: BlockConnector<N>,
}

const MIN_BLOCKS_BEHIND: u64 = 50;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl<N> Downloader<N>
where
    N: NetworkService + 'static,
{
    pub fn new(
        chain_reader: ServiceRef<ChainReaderService>,
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
        bus: Addr<BusActor>,
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
        peer_id: PeerId,
        rpc_client: &NetworkRpcClient,
        network: N,
        block_number: BlockNumber,
        total_difficulty: U256,
        is_full_mode: bool,
    ) -> Result<Option<BlockHeader>> {
        let mut ancestor_header = None;
        let peer_info = network
            .get_peer(peer_id.clone())
            .await?
            .ok_or_else(|| format_err!("get peer {:?} not exist.", peer_id))?;

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
        step: usize,
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
                    get_info_by_hash(&rpc_client, peer_id, vec![pivot.parent_hash()]).await?;
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

    fn set_pivot(&self, pivot: Option<PivotBlock<N>>) {
        self.block_connector.update_pivot(pivot);
    }
}
