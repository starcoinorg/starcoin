/// Sync message which outbound
use crate::pool::TTLPool;
use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::Result;
use bus::{Broadcast, BusActor};
use chain::ChainActorRef;
use futures::channel::mpsc;
use parking_lot::RwLock;
// use itertools;
use crate::helper::send_sync_request;
use crate::state_sync::StateSyncTaskActor;
use config::NodeConfig;
use crypto::HashValue;
use executor::TransactionExecutor;
use futures::executor::block_on;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithNumber, ProcessMessage, SyncRpcRequest, SyncRpcResponse,
};
use starcoin_state_tree::StateNodeStore;
use starcoin_sync_api::SyncMetadata;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use traits::Consensus;
use types::{
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    system_events::SystemEvents,
};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct SyncEvent {}

const MIN_PEER_SIZE: usize = 5;

#[derive(Clone)]
pub struct DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<E, C>>,
    self_peer_id: Arc<PeerId>,
    network: NetworkAsyncService,
    bus: Addr<BusActor>,
    sync_event_sender: mpsc::Sender<SyncEvent>,
    sync_duration: Duration,
    syncing: Arc<AtomicBool>,
    state_node_storage: Arc<dyn StateNodeStore>,
    sync_metadata: SyncMetadata,
    main_network: bool,
    future_blocks: Arc<RwLock<HashMap<HashValue, BlockDetail>>>,
}

impl<E, C> DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        node_config: Arc<NodeConfig>,
        peer_id: Arc<PeerId>,
        chain_reader: ChainActorRef<E, C>,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
        state_node_storage: Arc<dyn StateNodeStore>,
        sync_metadata: SyncMetadata,
    ) -> Result<Addr<DownloadActor<E, C>>> {
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
                state_node_storage,
                sync_metadata,
                main_network: node_config.base.net().is_main(),
                future_blocks: Arc::new(RwLock::new(HashMap::new())),
            }
        });
        Ok(download_actor)
    }

    fn sync_task(&mut self) {
        if !self.syncing.load(Ordering::Relaxed) {
            self.syncing.store(true, Ordering::Relaxed);
            Self::sync_block_from_best_peer(
                self.self_peer_id.as_ref().clone(),
                self.downloader.clone(),
                self.network.clone(),
                self.bus.clone(),
            );
            self.syncing.store(false, Ordering::Relaxed);
        }
    }
}

impl<E, C> Actor for DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.sync_task();
        ctx.run_interval(self.sync_duration, move |act, _ctx| {
            if !act.syncing.load(Ordering::Relaxed) {
                if let Err(e) = act.sync_event_sender.try_send(SyncEvent {}) {
                    warn!("err:{:?}", e);
                }
            }
        });
        info!("download actor started.")
    }
}

impl<E, C> Handler<SyncEvent> for DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;
    fn handle(&mut self, _item: SyncEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.sync_task();
        Ok(())
    }
}

impl<E, C> Handler<DownloadMessage> for DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: DownloadMessage, _ctx: &mut Self::Context) -> Self::Result {
        let downloader = self.downloader.clone();
        let network = self.network.clone();
        let state_node_storage = self.state_node_storage.clone();
        let sync_metadata = self.sync_metadata.clone();
        let is_main = self.main_network;
        let bus = self.bus.clone();
        let self_peer_id = self.self_peer_id.as_ref().clone();
        let fut = async move {
            match msg {
                DownloadMessage::NewPeerMsg(peer_id) => {
                    info!("new peer msg: {:?}", peer_id);
                    Self::sync_state(
                        self_peer_id,
                        is_main,
                        downloader.clone(),
                        network,
                        state_node_storage,
                        sync_metadata,
                        bus,
                    )
                    .await;
                }
                DownloadMessage::NewHeadBlock(peer_id, block) => {
                    info!(
                        "receive new block: {:?} from {:?}",
                        block.header().id(),
                        peer_id
                    );
                    // connect block
                    Downloader::do_block(downloader.clone(), block).await;
                }
                DownloadMessage::ClosePeerMsg(peer_id) => {
                    warn!("close peer: {:?}", peer_id,);
                }
                _ => {}
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

impl<E, C> DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    async fn sync_state(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<E, C>>,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        sync_metadata: SyncMetadata,
        bus: Addr<BusActor>,
    ) {
        if let Err(e) = Self::sync_state_inner(
            self_peer_id,
            main_network,
            downloader,
            network,
            state_node_storage,
            sync_metadata,
            bus,
        )
        .await
        {
            error!("error : {:?}", e);
        }
    }

    async fn sync_state_inner(
        self_peer_id: PeerId,
        main_network: bool,
        downloader: Arc<Downloader<E, C>>,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
        sync_metadata: SyncMetadata,
        bus: Addr<BusActor>,
    ) -> Result<()> {
        if (main_network && network.get_peer_set_size().await? >= MIN_PEER_SIZE) || !main_network {
            if sync_metadata.is_state_sync()? {
                if let Some(best_peer) = network.best_peer().await? {
                    if self_peer_id != best_peer.get_peer_id() {
                        //1. ancestor
                        let begin_number = downloader
                            .chain_reader
                            .clone()
                            .master_head_header()
                            .await
                            .unwrap()
                            .number();

                        let ancestor = if let Some(hash_with_number) = Downloader::find_ancestor(
                            downloader.clone(),
                            best_peer.get_peer_id(),
                            network.clone(),
                            begin_number,
                        )
                        .await
                        {
                            hash_with_number.number
                        } else {
                            0
                        };

                        // 2. pivot
                        let latest_number = best_peer.get_block_number();
                        let min_behind = if main_network {
                            MAIN_MIN_BLOCKS_BEHIND
                        } else {
                            MIN_BLOCKS_BEHIND
                        };
                        if (ancestor + min_behind) <= latest_number {
                            let pivot = latest_number - min_behind;

                            // 3. get pivot hash
                            let mut numbers: Vec<BlockNumber> = Vec::new();
                            numbers.push(pivot);
                            let get_hash_by_number_req = SyncRpcRequest::GetHashByNumberMsg(
                                ProcessMessage::GetHashByNumberMsg(GetHashByNumberMsg { numbers }),
                            );
                            if let SyncRpcResponse::BatchHashByNumberMsg(
                                mut batch_hash_by_number_msg,
                            ) = send_sync_request(
                                &network,
                                best_peer.get_peer_id(),
                                get_hash_by_number_req.clone(),
                            )
                            .await?
                            {
                                // 4. get pivot header
                                let hash_with_number =
                                    batch_hash_by_number_msg.hashs.pop().unwrap();
                                let mut hashs = Vec::new();
                                hashs.push(hash_with_number.hash);
                                let get_header_msg = GetDataByHashMsg {
                                    hashs,
                                    data_type: DataType::HEADER,
                                };
                                let get_data_by_hash_req = SyncRpcRequest::GetDataByHashMsg(
                                    ProcessMessage::GetDataByHashMsg(get_header_msg),
                                );
                                if let SyncRpcResponse::BatchHeaderAndBodyMsg(
                                    mut headers,
                                    _bodies,
                                    _infos,
                                ) = send_sync_request(
                                    &network,
                                    best_peer.get_peer_id(),
                                    get_data_by_hash_req.clone(),
                                )
                                .await?
                                {
                                    // 5. StateSyncActor
                                    let root = headers.headers.pop().unwrap();
                                    let sync_pivot = sync_metadata.get_pivot()?;
                                    if sync_metadata.is_state_sync()? {
                                        if sync_pivot.is_none() || sync_pivot.unwrap() < pivot {
                                            sync_metadata.clone().update_pivot(pivot)?;
                                            if sync_pivot.is_none() {
                                                let state_sync_task_address =
                                                    StateSyncTaskActor::launch(
                                                        self_peer_id,
                                                        root.state_root(),
                                                        state_node_storage,
                                                        network.clone(),
                                                        sync_metadata.clone(),
                                                        bus,
                                                    );
                                                sync_metadata
                                                    .update_address(&state_sync_task_address)?
                                            } else if sync_pivot.unwrap() < pivot {
                                                //todo:reset
                                                if let Some(address) = sync_metadata.get_address() {
                                                    &address.reset(root.state_root());
                                                } else {
                                                    warn!(
                                                        "{:?}",
                                                        "state sync reset address is none."
                                                    );
                                                }
                                            }
                                        } else {
                                            warn!("pivot {:?} : {}", sync_pivot, pivot);
                                        }
                                    } else {
                                        warn!("{:?}", "not state sync mode.");
                                    }
                                }
                            }
                        }
                    }
                } else {
                    warn!("{:?}", "best peer is none.");
                }
            } else {
                warn!("{:?}", "not state sync mode.");
            }
        } else {
            warn!("{:?}", "nothing todo when sync state.");
        }

        Ok(())
    }

    fn sync_block_from_best_peer(
        self_peer_id: PeerId,
        downloader: Arc<Downloader<E, C>>,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
    ) {
        Arbiter::spawn(async move {
            debug!("begin sync.");
            if let Some(best_peer) = network.best_peer().await.unwrap() {
                info!("peers: {:?}, {:?}", self_peer_id, best_peer.get_peer_id());
                if self_peer_id != best_peer.get_peer_id() {
                    let mut begin_number = downloader
                        .chain_reader
                        .clone()
                        .master_head_header()
                        .await
                        .unwrap()
                        .number();

                    let hash_with_number = Downloader::find_ancestor(
                        downloader.clone(),
                        best_peer.get_peer_id(),
                        network.clone(),
                        begin_number,
                    )
                    .await;

                    debug!("hash_with_number:{:?}", hash_with_number);
                    match hash_with_number {
                        Some(hash_number) => {
                            begin_number = hash_number.number + 1;
                            loop {
                                //1. sync hash
                                let send_get_hash_by_number_msg =
                                    Downloader::<E, C>::send_get_hash_by_number_msg_forward(
                                        network.clone(),
                                        best_peer.get_peer_id(),
                                        begin_number,
                                    )
                                    .await;

                                info!(
                                    "get_hash_by_number_msg:{:?}, forward",
                                    send_get_hash_by_number_msg
                                );

                                match send_get_hash_by_number_msg {
                                    Some((get_hash_by_number_msg, end, next_number)) => {
                                        begin_number = next_number;

                                        let get_hash_by_number_req =
                                            SyncRpcRequest::GetHashByNumberMsg(
                                                ProcessMessage::GetHashByNumberMsg(
                                                    get_hash_by_number_msg,
                                                ),
                                            );

                                        if let SyncRpcResponse::BatchHashByNumberMsg(
                                            batch_hash_by_number_msg,
                                        ) = send_sync_request(
                                            &network,
                                            best_peer.get_peer_id(),
                                            get_hash_by_number_req.clone(),
                                        )
                                        .await
                                        .expect("send_request 1 err.")
                                        {
                                            Downloader::handle_batch_hash_by_number_msg(
                                                downloader.clone(),
                                                best_peer.get_peer_id(),
                                                batch_hash_by_number_msg,
                                            );
                                        }

                                        if let Some(get_data_by_hash_msg) =
                                            Downloader::send_get_header_by_hash_msg(
                                                downloader.clone(),
                                            )
                                            .await
                                        {
                                            let get_data_by_hash_req =
                                                SyncRpcRequest::GetDataByHashMsg(
                                                    ProcessMessage::GetDataByHashMsg(
                                                        get_data_by_hash_msg,
                                                    ),
                                                );

                                            if let SyncRpcResponse::BatchHeaderAndBodyMsg(
                                                headers,
                                                bodies,
                                                infos,
                                            ) = send_sync_request(
                                                &network,
                                                best_peer.get_peer_id(),
                                                get_data_by_hash_req.clone(),
                                            )
                                            .await
                                            .expect("send_request 2 err.")
                                            {
                                                Downloader::do_blocks(
                                                    downloader.clone(),
                                                    headers.headers,
                                                    bodies.bodies,
                                                    infos.infos,
                                                )
                                                .await;
                                            }
                                        }

                                        if end {
                                            break;
                                        }
                                    }
                                    _ => {
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {
                            warn!("find ancestor is none.");
                        }
                    }
                }
            };
            let _ = bus
                .send(Broadcast {
                    msg: SystemEvents::SyncDone(),
                })
                .await;
            debug!("end sync.");
        });
    }
}

/// Send download message
pub struct Downloader<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    hash_pool: TTLPool<HashWithNumber>,
    _header_pool: TTLPool<BlockHeader>,
    _body_pool: TTLPool<BlockBody>,
    chain_reader: ChainActorRef<E, C>,
}

const HEAD_CT: u64 = 10;
const MIN_BLOCKS_BEHIND: u64 = 10;
const MAIN_MIN_BLOCKS_BEHIND: u64 = 100;

impl<E, C> Downloader<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<E, C>) -> Self {
        Downloader {
            hash_pool: TTLPool::new(),
            _header_pool: TTLPool::new(),
            _body_pool: TTLPool::new(),
            //            _network: network,
            chain_reader,
        }
    }

    /// for ancestors
    pub async fn send_get_hash_by_number_msg_backward(
        network: NetworkAsyncService,
        peer_id: PeerId,
        begin_number: u64,
    ) -> Option<(GetHashByNumberMsg, bool, u64)> {
        //todo：binary search

        if let Some(peer_info) =
            block_on(async move { network.get_peer(&peer_id.into()).await.unwrap() })
        {
            let number = peer_info.get_block_number();
            info!(
                "sync with peer {:?} : latest number: {} , begin number : {}",
                peer_info.get_peer_id(),
                number,
                begin_number
            );

            if begin_number < number {
                let mut numbers = Vec::new();
                let mut end = false;
                let mut next_number = 0;
                if begin_number < HEAD_CT {
                    for i in 0..(begin_number + 1) {
                        info!("best peer backward number : {}, number : {}", number, i);
                        numbers.push(i);
                        end = true;
                    }
                } else {
                    for i in (begin_number - HEAD_CT + 1)..(begin_number + 1) {
                        info!("best peer backward number : {}, number : {}, ", number, i);
                        numbers.push(i);
                    }
                    next_number = begin_number - HEAD_CT;
                };
                info!(
                    "best peer backward number : {}, next number : {}",
                    number, next_number
                );
                return Some((GetHashByNumberMsg { numbers }, end, next_number));
            }
        }
        warn!("GetHashByNumberMsg is none.");
        None
    }

    pub async fn send_get_hash_by_number_msg_forward(
        network: NetworkAsyncService,
        peer_id: PeerId,
        begin_number: u64,
    ) -> Option<(GetHashByNumberMsg, bool, u64)> {
        if let Some(peer_info) =
            block_on(async move { network.get_peer(&peer_id.into()).await.unwrap() })
        {
            let number = peer_info.get_block_number();
            if begin_number < number {
                let mut numbers = Vec::new();
                let mut end = false;
                let mut next_number = 0;
                if (number - begin_number) < HEAD_CT {
                    for i in begin_number..(number + 1) {
                        info!("best peer forward number : {}, next number : {}", number, i,);
                        numbers.push(i);
                        end = true;
                    }
                } else {
                    for i in begin_number..(begin_number + HEAD_CT) {
                        info!("best peer forward number : {}, next number : {}", number, i,);
                        numbers.push(i);
                    }
                    next_number = begin_number + HEAD_CT;
                };

                info!(
                    "best peer forward number : {}, next number : {}",
                    number, next_number
                );
                return Some((GetHashByNumberMsg { numbers }, end, next_number));
            }
        }
        None
    }

    pub async fn find_ancestor(
        downloader: Arc<Downloader<E, C>>,
        peer_id: PeerId,
        network: NetworkAsyncService,
        block_number: BlockNumber,
    ) -> Option<HashWithNumber> {
        let mut hash_with_number = None;
        let mut begin_number = block_number;
        loop {
            let send_get_hash_by_number_msg =
                Downloader::<E, C>::send_get_hash_by_number_msg_backward(
                    network.clone(),
                    peer_id.clone(),
                    begin_number,
                )
                .await;

            info!(
                "get_hash_by_number_msg:{:?}, backward",
                send_get_hash_by_number_msg
            );
            match send_get_hash_by_number_msg {
                Some((get_hash_by_number_msg, end, next_number)) => {
                    begin_number = next_number;
                    info!(
                        "peer: {:?} , numbers : {}",
                        peer_id.clone(),
                        get_hash_by_number_msg.numbers.len()
                    );
                    let get_hash_by_number_req = SyncRpcRequest::GetHashByNumberMsg(
                        ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg),
                    );

                    if let SyncRpcResponse::BatchHashByNumberMsg(batch_hash_by_number_msg) =
                        send_sync_request(&network, peer_id.clone(), get_hash_by_number_req.clone())
                            .await
                            .expect("send_request 1 err.")
                    {
                        debug!("batch_hash_by_number_msg:{:?}", batch_hash_by_number_msg);
                        hash_with_number = Downloader::handle_hash_by_number_msg(
                            downloader.clone(),
                            peer_id.clone(),
                            batch_hash_by_number_msg,
                        )
                        .await;
                    }

                    if end || hash_with_number.is_some() {
                        break;
                    }
                }
                _ => {
                    break;
                }
            }
        }

        hash_with_number
    }

    pub async fn handle_hash_by_number_msg(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerId,
        batch_hash_by_number_msg: BatchHashByNumberMsg,
    ) -> Option<HashWithNumber> {
        //TODO
        let mut exist_ancestor = false;
        let mut ancestor = None;
        let mut hashs = batch_hash_by_number_msg.hashs.clone();
        let mut not_exist_hash = Vec::new();
        hashs.reverse();
        for hash in hashs {
            if downloader
                .chain_reader
                .clone()
                .get_block_by_hash(&hash.hash)
                .await
                .is_some()
            {
                exist_ancestor = true;
                info!("find ancestor hash : {:?}", hash);
                ancestor = Some(hash);
                break;
            } else {
                not_exist_hash.push(hash);
            }
        }

        if exist_ancestor {
            for hash in not_exist_hash {
                downloader
                    .hash_pool
                    .insert(peer.clone(), hash.number.clone(), hash);
            }
        }
        ancestor
    }

    fn handle_batch_hash_by_number_msg(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerId,
        batch_hash_by_number_msg: BatchHashByNumberMsg,
    ) {
        for hash in batch_hash_by_number_msg.hashs {
            downloader
                .hash_pool
                .insert(peer.clone(), hash.number.clone(), hash);
        }
    }

    pub async fn send_get_header_by_hash_msg(
        downloader: Arc<Downloader<E, C>>,
    ) -> Option<GetDataByHashMsg> {
        let hash_vec = downloader.hash_pool.take(100);
        if !hash_vec.is_empty() {
            let hashs = hash_vec.iter().map(|hash| hash.hash).collect();
            Some(GetDataByHashMsg {
                hashs,
                data_type: DataType::HEADER,
            })
        } else {
            None
        }
    }

    pub async fn _handle_batch_header_msg(
        downloader: Arc<Downloader<E, C>>,
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

    pub async fn _send_get_body_by_hash_msg(
        downloader: Arc<Downloader<E, C>>,
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
        downloader: Arc<Downloader<E, C>>,
        headers: Vec<BlockHeader>,
        bodies: Vec<BlockBody>,
        _infos: Vec<BlockInfo>,
    ) {
        assert_eq!(headers.len(), bodies.len());
        for i in 0..headers.len() {
            let block = Block::new(
                headers.get(i).unwrap().clone(),
                bodies.get(i).unwrap().clone().transactions,
            );
            //todo:verify block
            let _ = Self::do_block(downloader.clone(), block).await;
        }
    }

    pub async fn _do_block_with_info(
        downloader: Arc<Downloader<E, C>>,
        block: Block,
        block_info: BlockInfo,
    ) {
        info!("do block info {:?}", block.header().id());
        //todo:verify block
        let _ = downloader
            .chain_reader
            .clone()
            .try_connect_with_block_info(block, block_info)
            .await;
    }

    pub async fn do_block(downloader: Arc<Downloader<E, C>>, block: Block) {
        info!("do block {:?}", block.header().id());
        //todo:verify block
        let _ = downloader.chain_reader.clone().try_connect(block).await;
    }
}
