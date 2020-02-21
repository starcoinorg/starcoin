use crate::message::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg, ProcessMessage,
};
/// Sync message which outbound
use crate::pool::TTLPool;
use actix::prelude::*;
use actix::{
    fut::wrap_future, fut::FutureWrap, Actor, Addr, AsyncContext, Context, Handler,
    ResponseActFuture,
};
use anyhow::{Error, Result};
use atomic_refcell::AtomicRefCell;
use chain::{mem_chain::MemChainActor, ChainActorRef};
use futures::compat::Future01CompatExt;
use futures_locks::{Mutex, RwLock};
use itertools;
use network::NetworkActor;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use traits::Chain;
use types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
};

#[derive(Clone)]
pub struct DownloadActor {
    downloader: Arc<RwLock<Downloader>>,
    peer_info: Arc<PeerInfo>,
}

impl DownloadActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: ChainActorRef<MemChainActor>,
    ) -> Result<Addr<DownloadActor>> {
        let download_actor = DownloadActor {
            downloader: Arc::new(RwLock::new(Downloader::new(chain_reader))),
            peer_info,
        };
        Ok(download_actor.start())
    }
}

impl Actor for DownloadActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("download actor started.")
    }
}

impl Handler<DownloadMessage> for DownloadActor {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: DownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        let downloader = self.downloader.clone();
        let my_addr = ctx.address();
        let fut = async move {
            match msg {
                DownloadMessage::LatestStateMsg(addr, peer_info, latest_state_msg) => {
                    Downloader::handle_latest_state_msg(
                        downloader.clone(),
                        peer_info,
                        latest_state_msg,
                    )
                    .await;
                    match addr {
                        Some(address) => {
                            let send_get_hash_by_number_msg =
                                Downloader::send_get_hash_by_number_msg(downloader.clone()).await;
                            match send_get_hash_by_number_msg {
                                Some(get_hash_by_number_msg) => {
                                    address
                                        .send(ProcessMessage::GetHashByNumberMsg(
                                            Some(my_addr),
                                            get_hash_by_number_msg,
                                        ))
                                        .await;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                DownloadMessage::BatchHashByNumberMsg(
                    addr,
                    peer_info,
                    batch_hash_by_number_msg,
                ) => {
                    let hash_with_number = Downloader::find_ancestor(
                        downloader.clone(),
                        peer_info,
                        batch_hash_by_number_msg,
                    )
                    .await;
                    match hash_with_number {
                        Some(_) => {
                            let send_get_header_by_hash_msg =
                                Downloader::send_get_header_by_hash_msg(downloader.clone()).await;
                            match send_get_header_by_hash_msg {
                                Some(get_data_by_hash_msg) => match addr {
                                    Some(address) => {
                                        address
                                            .send(ProcessMessage::GetDataByHashMsg(
                                                Some(my_addr),
                                                get_data_by_hash_msg,
                                            ))
                                            .await;
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                DownloadMessage::BatchHeaderMsg(addr, peer_info, batch_header_msg) => {
                    Downloader::handle_batch_header_msg(
                        downloader.clone(),
                        peer_info,
                        batch_header_msg,
                    )
                    .await;
                    let send_get_body_by_hash_msg =
                        Downloader::send_get_body_by_hash_msg(downloader.clone()).await;
                    match send_get_body_by_hash_msg {
                        Some(get_body_by_hash_msg) => match addr {
                            Some(address) => {
                                address
                                    .send(ProcessMessage::GetDataByHashMsg(
                                        Some(my_addr),
                                        get_body_by_hash_msg,
                                    ))
                                    .await;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
                DownloadMessage::BatchBodyMsg(addr, batch_body_msg) => {
                    println!("{:?}", batch_body_msg);
                }
                DownloadMessage::BatchHeaderAndBodyMsg(batch_header_msg, batch_body_msg) => {
                    Downloader::do_block(
                        downloader.clone(),
                        batch_header_msg.headers,
                        batch_body_msg.bodies,
                    )
                    .await;
                }
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

/// Send download message
pub struct Downloader {
    hash_pool: TTLPool<HashWithNumber>,
    header_pool: TTLPool<HashWithBlockHeader>,
    body_pool: TTLPool<BlockBody>,
    //    _network: Addr<NetworkActor>,
    peers: HashMap<PeerInfo, LatestStateMsg>,
    chain_reader: ChainActorRef<MemChainActor>,
}

const HEAD_CT: u64 = 100;

impl Downloader {
    pub fn new(chain_reader: ChainActorRef<MemChainActor>) -> Self {
        Downloader {
            hash_pool: TTLPool::new(),
            header_pool: TTLPool::new(),
            body_pool: TTLPool::new(),
            //            _network: network,
            peers: HashMap::new(),
            chain_reader,
        }
    }

    pub async fn handle_latest_state_msg(
        downloader: Arc<RwLock<Downloader>>,
        peer: PeerInfo,
        latest_state_msg: LatestStateMsg,
    ) {
        // let hash_num = HashWithNumber {
        //     hash: latest_state_msg.hash_header.hash.clone(),
        //     number: latest_state_msg.hash_header.header.number(),
        // };
        //        self.hash_pool
        //            .insert(peer.clone(), latest_state_msg.header.number(), hash_num);
        downloader
            .write()
            .compat()
            .await
            .unwrap()
            .peers
            .insert(peer, latest_state_msg.clone());
    }

    async fn best_peer(downloader: Arc<RwLock<Downloader>>) -> PeerInfo {
        let lock = downloader.read().compat().await.unwrap();
        assert!(lock.peers.len() > 0);
        let mut peer = None;
        lock.peers.keys().for_each(|p| peer = Some(p.clone()));

        peer.take().expect("best peer is none.")
    }

    pub async fn send_get_hash_by_number_msg(
        downloader: Arc<RwLock<Downloader>>,
    ) -> Option<GetHashByNumberMsg> {
        let best_peer = Self::best_peer(downloader.clone()).await;
        let lock = downloader.read().compat().await.unwrap();
        //todo：binary search

        let latest_number = lock.chain_reader.clone().current_header().await.number();
        let number = lock
            .peers
            .get(&best_peer)
            .expect("Latest state is none.")
            .hash_header
            .header
            .number();
        if latest_number < number {
            let mut numbers = Vec::new();
            if number < HEAD_CT {
                for i in 0..(number + 1) {
                    numbers.push(i);
                }
            } else {
                for i in 0..HEAD_CT {
                    numbers.push((number - HEAD_CT + i + 1));
                }
            };

            Some(GetHashByNumberMsg { numbers })
        } else {
            None
        }
    }

    pub async fn find_ancestor(
        downloader: Arc<RwLock<Downloader>>,
        peer: PeerInfo,
        batch_hash_by_number_msg: BatchHashByNumberMsg,
    ) -> Option<HashWithNumber> {
        let mut lock = downloader.write().compat().await.unwrap();
        //TODO
        let mut exist_ancestor = false;
        let mut ancestor = None;
        let mut hashs = batch_hash_by_number_msg.hashs.clone();
        let mut not_exist_hash = Vec::new();
        hashs.reverse();
        for hash in hashs {
            if lock
                .chain_reader
                .clone()
                .get_block_by_hash(&hash.hash)
                .await
                .expect("block is none.")
                .is_some()
            {
                exist_ancestor = true;
                ancestor = Some(hash);
                break;
            } else {
                not_exist_hash.push(hash);
            }
        }

        if exist_ancestor {
            for hash in not_exist_hash {
                lock.borrow_mut()
                    .hash_pool
                    .insert(peer.clone(), hash.number.clone(), hash);
            }
        }
        ancestor
    }

    pub async fn send_get_header_by_hash_msg(
        downloader: Arc<RwLock<Downloader>>,
    ) -> Option<GetDataByHashMsg> {
        let mut lock = downloader.write().compat().await.unwrap();
        let hash_vec = lock.borrow_mut().hash_pool.take(100);
        if !hash_vec.is_empty() {
            let mut hashs = hash_vec.iter().map(|hash| hash.hash).collect();
            Some(GetDataByHashMsg {
                hashs,
                data_type: DataType::HEADER,
            })
        } else {
            None
        }
    }

    pub async fn handle_batch_header_msg(
        downloader: Arc<RwLock<Downloader>>,
        peer: PeerInfo,
        batch_header_msg: BatchHeaderMsg,
    ) {
        let mut lock = downloader.write().compat().await.unwrap();
        if !batch_header_msg.headers.is_empty() {
            for header in batch_header_msg.headers {
                lock.header_pool
                    .borrow_mut()
                    .insert(peer.clone(), header.header.number(), header);
            }
        }
    }

    pub async fn send_get_body_by_hash_msg(
        downloader: Arc<RwLock<Downloader>>,
    ) -> Option<GetDataByHashMsg> {
        let mut lock = downloader.write().compat().await.unwrap();
        let header_vec = lock.borrow_mut().header_pool.take(100);
        if !header_vec.is_empty() {
            let mut hashs = header_vec.iter().map(|header| header.hash).collect();
            Some(GetDataByHashMsg {
                hashs,
                data_type: DataType::BODY,
            })
        } else {
            None
        }
    }

    pub async fn do_block(
        downloader: Arc<RwLock<Downloader>>,
        headers: Vec<HashWithBlockHeader>,
        bodies: Vec<BlockBody>,
    ) {
        let lock = downloader.write().compat().await.unwrap();
        for (header, body) in itertools::zip_eq(headers, bodies) {
            let block = Block::new(header.header, body.transactions);
            //todo:verify block
            let _ = lock.chain_reader.clone().try_connect(block);
        }
    }
}
