/// Sync message which outbound
use crate::pool::TTLPool;
use crate::proto::{
    BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg,
};
use actix::Addr;
use atomic_refcell::AtomicRefCell;
use chain::{mem_chain::MemChain, BlockChain};
use itertools;
use network::NetworkActor;
use std::collections::HashMap;
use std::sync::Arc;
use types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
};

/// Send download message
pub struct Downloader {
    hash_pool: TTLPool<HashWithNumber>,
    header_pool: TTLPool<HashWithBlockHeader>,
    body_pool: TTLPool<BlockBody>,
    //    _network: Addr<NetworkActor>,
    peers: HashMap<PeerInfo, LatestStateMsg>,
    chain_reader: Arc<AtomicRefCell<MemChain>>,
}

const HEAD_CT: u64 = 10;

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

    pub fn send_get_hash_by_number_msg(&self) -> GetHashByNumberMsg {
        //todo：binary search

        let best_peer = self.best_peer();
        let number = self
            .peers
            .get(&best_peer)
            .expect("Latest state is none.")
            .hash_header
            .header
            .number();
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

        GetHashByNumberMsg { numbers }
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
                data_type: DataType::HEADER,
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
