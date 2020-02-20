use crate::message::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg, ProcessMessage,
};
/// Sync message which outbound
use crate::pool::TTLPool;
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::Result;
use atomic_refcell::AtomicRefCell;
use chain::{mem_chain::MemChain, ChainWriter};
use itertools;
use network::NetworkActor;
use std::collections::HashMap;
use std::sync::Arc;
use types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
};

pub struct DownloadActor {
    downloader: Downloader,
    peer_info: Arc<PeerInfo>,
}

impl DownloadActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: Arc<AtomicRefCell<MemChain>>,
    ) -> Result<Addr<DownloadActor>> {
        let download_actor = DownloadActor {
            downloader: Downloader::new(chain_reader),
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
    type Result = ();

    fn handle(&mut self, msg: DownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            DownloadMessage::LatestStateMsg(addr, peer_info, latest_state_msg) => {
                self.downloader
                    .handle_latest_state_msg(peer_info, latest_state_msg);
                match addr {
                    Some(address) => {
                        let send_get_hash_by_number_msg =
                            self.downloader.send_get_hash_by_number_msg();
                        match send_get_hash_by_number_msg {
                            Some(get_hash_by_number_msg) => {
                                address
                                    .send(ProcessMessage::GetHashByNumberMsg(
                                        Some(ctx.address()),
                                        get_hash_by_number_msg,
                                    ))
                                    .into_actor(self)
                                    .then(|_result, act, _ctx| async {}.into_actor(act))
                                    .wait(ctx);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            DownloadMessage::BatchHashByNumberMsg(addr, peer_info, batch_hash_by_number_msg) => {
                let hash_with_number = self
                    .downloader
                    .find_ancestor(peer_info, batch_hash_by_number_msg);
                match hash_with_number {
                    Some(_) => {
                        let send_get_header_by_hash_msg =
                            self.downloader.send_get_header_by_hash_msg();
                        match send_get_header_by_hash_msg {
                            Some(get_data_by_hash_msg) => match addr {
                                Some(address) => {
                                    address
                                        .send(ProcessMessage::GetDataByHashMsg(
                                            Some(ctx.address()),
                                            get_data_by_hash_msg,
                                        ))
                                        .into_actor(self)
                                        .then(|_result, act, _ctx| async {}.into_actor(act))
                                        .wait(ctx);
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
                self.downloader
                    .handle_batch_header_msg(peer_info, batch_header_msg);
                let send_get_body_by_hash_msg = self.downloader.send_get_body_by_hash_msg();
                match send_get_body_by_hash_msg {
                    Some(get_body_by_hash_msg) => match addr {
                        Some(address) => {
                            address
                                .send(ProcessMessage::GetDataByHashMsg(
                                    Some(ctx.address()),
                                    get_body_by_hash_msg,
                                ))
                                .into_actor(self)
                                .then(|_result, act, _ctx| async {}.into_actor(act))
                                .wait(ctx);
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
                self.downloader
                    .do_block(batch_header_msg.headers, batch_body_msg.bodies);
            }
        }
    }
}

/// Send download message
pub struct Downloader {
    hash_pool: TTLPool<HashWithNumber>,
    header_pool: TTLPool<HashWithBlockHeader>,
    body_pool: TTLPool<BlockBody>,
    //    _network: Addr<NetworkActor>,
    peers: HashMap<PeerInfo, LatestStateMsg>,
    chain_reader: Arc<AtomicRefCell<MemChain>>,
}

const HEAD_CT: u64 = 100;

impl Downloader {
    pub fn new(chain_reader: Arc<AtomicRefCell<MemChain>>) -> Self {
        Downloader {
            hash_pool: TTLPool::new(),
            header_pool: TTLPool::new(),
            body_pool: TTLPool::new(),
            //            _network: network,
            peers: HashMap::new(),
            chain_reader,
        }
    }

    pub fn handle_latest_state_msg(&mut self, peer: PeerInfo, latest_state_msg: LatestStateMsg) {
        // let hash_num = HashWithNumber {
        //     hash: latest_state_msg.hash_header.hash.clone(),
        //     number: latest_state_msg.hash_header.header.number(),
        // };
        //        self.hash_pool
        //            .insert(peer.clone(), latest_state_msg.header.number(), hash_num);
        self.peers.insert(peer, latest_state_msg.clone());
    }

    fn best_peer(&self) -> PeerInfo {
        assert!(self.peers.len() > 0);
        let mut peer = None;
        self.peers.keys().for_each(|p| peer = Some(p.clone()));

        peer.take().expect("best peer is none.")
    }

    pub fn send_get_hash_by_number_msg(&self) -> Option<GetHashByNumberMsg> {
        //todo：binary search

        let best_peer = self.best_peer();
        let latest_number = self.chain_reader.borrow().current_header().number();
        let number = self
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

    pub fn find_ancestor(
        &mut self,
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
            if self
                .chain_reader
                .borrow()
                .get_block_by_hash(&hash.hash)
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
                self.hash_pool
                    .insert(peer.clone(), hash.number.clone(), hash);
            }
        }
        ancestor
    }

    pub fn send_get_header_by_hash_msg(&mut self) -> Option<GetDataByHashMsg> {
        let hash_vec = self.hash_pool.take(100);
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

    pub fn handle_batch_header_msg(&mut self, peer: PeerInfo, batch_header_msg: BatchHeaderMsg) {
        if !batch_header_msg.headers.is_empty() {
            for header in batch_header_msg.headers {
                self.header_pool
                    .insert(peer.clone(), header.header.number(), header);
            }
        }
    }

    pub fn send_get_body_by_hash_msg(&mut self) -> Option<GetDataByHashMsg> {
        let header_vec = self.header_pool.take(100);
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

    pub fn do_block(&mut self, headers: Vec<HashWithBlockHeader>, bodies: Vec<BlockBody>) {
        for (header, body) in itertools::zip_eq(headers, bodies) {
            let block = Block::new(header.header, body.transactions);
            //todo:verify block
            let _ = self.chain_reader.borrow_mut().try_connect(block);
        }
    }
}
