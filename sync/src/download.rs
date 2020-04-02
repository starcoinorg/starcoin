/// Sync message which outbound
use crate::pool::TTLPool;
use crate::{do_duration, DELAY_TIME};
use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::Result;
use bus::BusActor;
use chain::ChainActorRef;
use futures::channel::mpsc;
use parking_lot::RwLock;
// use itertools;
use crate::state_sync::StateSyncActor;
use consensus::Consensus;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::{NetworkAsyncService, RPCRequest, RPCResponse};
use network_p2p_api::sync_messages::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithNumber, LatestStateMsg, ProcessMessage,
};
use starcoin_state_tree::StateNodeStore;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;
use types::{
    block::{Block, BlockHeader, BlockNumber},
    peer_info::PeerInfo,
};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
struct SyncEvent {}

#[derive(Clone)]
pub struct DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    downloader: Arc<Downloader<E, C>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService,
    bus: Addr<BusActor>,
    sync_event_sender: mpsc::Sender<SyncEvent>,
    sync_duration: Duration,
    syncing: Arc<AtomicBool>,
    state_node_storage: Arc<dyn StateNodeStore>,
}

impl<E, C> DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: ChainActorRef<E, C>,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) -> Result<Addr<DownloadActor<E, C>>> {
        let download_actor = DownloadActor::create(move |ctx| {
            let (sync_event_sender, sync_event_receiver) = mpsc::channel(100);
            ctx.add_message_stream(sync_event_receiver);
            DownloadActor {
                downloader: Arc::new(Downloader::new(chain_reader)),
                peer_info,
                network,
                bus,
                sync_event_sender,
                sync_duration: Duration::from_secs(5),
                syncing: Arc::new(AtomicBool::new(false)),
                state_node_storage,
            }
        });
        Ok(download_actor)
    }
}

impl<E, C> Actor for DownloadActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
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
        if !self.syncing.load(Ordering::Relaxed) {
            self.syncing.store(true, Ordering::Relaxed);
            Self::sync_block_from_best_peer(self.downloader.clone(), self.network.clone());
            self.syncing.store(false, Ordering::Relaxed);
        }
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
        let fut = async move {
            match msg {
                DownloadMessage::LatestStateMsg(peer_info, latest_state_msg) => {
                    debug!(
                        "latest_state_msg number: {:?}",
                        &latest_state_msg.header.number()
                    );
                    Downloader::handle_latest_state_msg(
                        downloader.clone(),
                        peer_info.clone(),
                        latest_state_msg,
                    )
                    .await;
                }
                DownloadMessage::NewHeadBlock(peer_info, block) => {
                    info!(
                        "receive new block: {:?} from {:?}",
                        block.header().id(),
                        peer_info.id
                    );
                    //1. update latest block
                    let latest_state_msg = LatestStateMsg {
                        header: block.header().clone(),
                    };
                    Downloader::handle_latest_state_msg(
                        downloader.clone(),
                        peer_info.clone(),
                        latest_state_msg,
                    )
                    .await;

                    //2. connect block
                    Downloader::do_block(downloader.clone(), block).await;
                }
                DownloadMessage::ClosePeerMsg(peer_info) => {
                    debug!("close peer: {:?}", peer_info,);

                    Downloader::close_peer(downloader.clone(), peer_info).await;
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
    fn sync_state(
        downloader: Arc<Downloader<E, C>>,
        network: NetworkAsyncService,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) {
        Arbiter::spawn(async move {
            if let Some(best_peer) = Downloader::best_peer(downloader.clone()).await {
                //1. ancestor
                let mut begin_number = downloader
                    .chain_reader
                    .clone()
                    .master_head_header()
                    .await
                    .unwrap()
                    .number();

                let ancestor = if let Some(hash_with_number) = Downloader::find_ancestor(
                    downloader.clone(),
                    best_peer.clone(),
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
                let latest_number = downloader.get_latest_header_with_peer(&best_peer).number();
                if (ancestor + MIN_BLOCKS_BEHIND) <= latest_number {
                    let pivot = latest_number - MIN_BLOCKS_BEHIND;
                    downloader.update_pivot(pivot);

                    // 3. get pivot hash
                    let mut numbers: Vec<BlockNumber> = Vec::new();
                    numbers.push(pivot);
                    let get_hash_by_number_req = RPCRequest::GetHashByNumberMsg(
                        ProcessMessage::GetHashByNumberMsg(GetHashByNumberMsg { numbers }),
                    );
                    if let RPCResponse::BatchHashByNumberMsg(mut batch_hash_by_number_msg) = network
                        .clone()
                        .send_request(
                            best_peer.id.clone().into(),
                            get_hash_by_number_req.clone(),
                            do_duration(DELAY_TIME),
                        )
                        .await
                        .expect("send_request 2 err.")
                    {
                        // 4. get pivot header
                        let hash_with_number = batch_hash_by_number_msg.hashs.pop().unwrap();
                        let mut hashs = Vec::new();
                        hashs.push(hash_with_number.hash);
                        let get_header_msg = GetDataByHashMsg {
                            hashs,
                            data_type: DataType::HEADER,
                        };
                        let get_data_by_hash_req = RPCRequest::GetDataByHashMsg(
                            ProcessMessage::GetDataByHashMsg(get_header_msg),
                        );
                        if let RPCResponse::BatchHeaderAndBodyMsg(mut headers, bodies) = network
                            .clone()
                            .send_request(
                                best_peer.id.clone().into(),
                                get_data_by_hash_req.clone(),
                                do_duration(DELAY_TIME),
                            )
                            .await
                            .expect("send_request 3 err.")
                        {
                            // 5. StateSyncActor
                            let root = headers.headers.pop().unwrap();
                            let _ = StateSyncActor::launch(
                                root.state_root(),
                                network.clone(),
                                state_node_storage,
                                downloader.clone(),
                            );
                        }
                    }
                }
            }
        });
    }

    fn sync_block_from_best_peer(downloader: Arc<Downloader<E, C>>, network: NetworkAsyncService) {
        Arbiter::spawn(async move {
            debug!("begin sync.");
            if let Some(best_peer) = Downloader::best_peer(downloader.clone()).await {
                let mut begin_number = downloader
                    .chain_reader
                    .clone()
                    .master_head_header()
                    .await
                    .unwrap()
                    .number();

                let mut hash_with_number = Downloader::find_ancestor(
                    downloader.clone(),
                    best_peer.clone(),
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
                                Downloader::send_get_hash_by_number_msg_forward(
                                    downloader.clone(),
                                    best_peer.clone(),
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

                                    let get_hash_by_number_req = RPCRequest::GetHashByNumberMsg(
                                        ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg),
                                    );

                                    if let RPCResponse::BatchHashByNumberMsg(
                                        batch_hash_by_number_msg,
                                    ) = network
                                        .clone()
                                        .send_request(
                                            best_peer.id.clone().into(),
                                            get_hash_by_number_req.clone(),
                                            do_duration(DELAY_TIME),
                                        )
                                        .await
                                        .expect("send_request 2 err.")
                                    {
                                        Downloader::handle_batch_hash_by_number_msg(
                                            downloader.clone(),
                                            best_peer.clone(),
                                            batch_hash_by_number_msg,
                                        );
                                    }

                                    if let Some(get_data_by_hash_msg) =
                                        Downloader::send_get_header_by_hash_msg(downloader.clone())
                                            .await
                                    {
                                        let get_data_by_hash_req = RPCRequest::GetDataByHashMsg(
                                            ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg),
                                        );

                                        if let RPCResponse::BatchHeaderAndBodyMsg(headers, bodies) =
                                            network
                                                .clone()
                                                .send_request(
                                                    best_peer.id.clone().into(),
                                                    get_data_by_hash_req.clone(),
                                                    do_duration(DELAY_TIME),
                                                )
                                                .await
                                                .expect("send_request 3 err.")
                                        {
                                            Downloader::do_blocks(
                                                downloader.clone(),
                                                headers.headers,
                                                bodies.bodies,
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
            };
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
    peers: Arc<RwLock<HashMap<PeerInfo, LatestStateMsg>>>,
    chain_reader: ChainActorRef<E, C>,
    pivot: RwLock<Option<BlockNumber>>,
}

const HEAD_CT: u64 = 10;
const MIN_BLOCKS_BEHIND: u64 = 100;

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
            peers: Arc::new(RwLock::new(HashMap::new())),
            chain_reader,
            pivot: RwLock::new(None),
        }
    }

    pub fn get_latest_header_with_peer(&self, peer: &PeerInfo) -> BlockHeader {
        self.peers.read().get(&peer).unwrap().header.clone()
    }

    pub fn update_pivot(&self, pivot: BlockNumber) {
        *self.pivot.write() = Some(pivot);
    }

    pub fn get_pivot(&self) -> BlockNumber {
        self.pivot.read().clone().unwrap()
    }

    pub async fn handle_latest_state_msg(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerInfo,
        latest_state_msg: LatestStateMsg,
    ) {
        // let hash_num = HashWithNumber {
        //     hash: latest_state_msg.hash_header.hash.clone(),
        //     number: latest_state_msg.hash_header.header.number(),
        // };
        //        self.hash_pool
        //            .insert(peer.clone(), latest_state_msg.header.number(), hash_num);
        let mut lock = downloader.peers.write();
        if lock.get(&peer).is_none()
            || (lock.get(&peer).unwrap().header.number() < latest_state_msg.header.number())
        {
            info!(
                "peer {:?} : latest number: {} , hash : {:?}",
                peer.id,
                latest_state_msg.header.number(),
                latest_state_msg.header.id()
            );
            lock.insert(peer, latest_state_msg.clone());
        }
    }

    pub async fn best_peer(downloader: Arc<Downloader<E, C>>) -> Option<PeerInfo> {
        let lock = downloader.peers.read();
        for p in lock.keys() {
            return Some(p.clone());
        }

        info!("best peer return none.");
        None
    }

    /// for ancestors
    pub async fn send_get_hash_by_number_msg_backward(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerInfo,
        begin_number: u64,
    ) -> Option<(GetHashByNumberMsg, bool, u64)> {
        //todo：binary search

        let number = downloader
            .peers
            .read()
            .get(&peer)
            .expect("Latest state is none.")
            .header
            .number();

        info!(
            "sync with peer {:?} : latest number: {} , begin number : {}",
            peer.id, number, begin_number
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
            Some((GetHashByNumberMsg { numbers }, end, next_number))
        } else {
            warn!("GetHashByNumberMsg is none.");
            None
        }
    }

    pub async fn send_get_hash_by_number_msg_forward(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerInfo,
        begin_number: u64,
    ) -> Option<(GetHashByNumberMsg, bool, u64)> {
        let number = downloader
            .peers
            .read()
            .get(&peer)
            .expect("Latest state is none.")
            .header
            .number();

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
            Some((GetHashByNumberMsg { numbers }, end, next_number))
        } else {
            None
        }
    }

    pub async fn find_ancestor(
        downloader: Arc<Downloader<E, C>>,
        peer: PeerInfo,
        network: NetworkAsyncService,
        block_number: BlockNumber,
    ) -> Option<HashWithNumber> {
        let mut hash_with_number = None;
        let mut begin_number = block_number;
        loop {
            let send_get_hash_by_number_msg = Downloader::send_get_hash_by_number_msg_backward(
                downloader.clone(),
                peer.clone(),
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
                        peer.clone(),
                        get_hash_by_number_msg.numbers.len()
                    );
                    let get_hash_by_number_req = RPCRequest::GetHashByNumberMsg(
                        ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg),
                    );

                    if let RPCResponse::BatchHashByNumberMsg(batch_hash_by_number_msg) = network
                        .clone()
                        .send_request(
                            peer.id.clone().into(),
                            get_hash_by_number_req.clone(),
                            do_duration(DELAY_TIME),
                        )
                        .await
                        .expect("send_request 1 err.")
                    {
                        debug!("batch_hash_by_number_msg:{:?}", batch_hash_by_number_msg);
                        hash_with_number = Downloader::handle_hash_by_number_msg(
                            downloader.clone(),
                            peer.clone(),
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
        peer: PeerInfo,
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
        peer: PeerInfo,
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
        peer: PeerInfo,
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

    pub async fn do_block(downloader: Arc<Downloader<E, C>>, block: Block) {
        info!("do block {:?}", block.header().id());
        //todo:verify block
        let _ = downloader.chain_reader.clone().try_connect(block).await;
    }

    pub async fn close_peer(downloader: Arc<Downloader<E, C>>, peer: PeerInfo) {
        let mut lock = downloader.peers.write();
        let _ = lock.remove(&peer);
    }
}
