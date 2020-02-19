use crate::proto::{
    BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg,
};
use actix::Addr;
use atomic_refcell::AtomicRefCell;
/// Sync message which inbound
use chain::{mem_chain::MemChain, BlockChain};
use crypto::hash::CryptoHash;
use network::NetworkActor;
use std::hash::Hash;
use std::sync::Arc;
use types::block::Block;

/// Process request for syncing block
pub struct Processor {
    //    chain: Addr<ChainActor>,
    chain_reader: Arc<AtomicRefCell<MemChain>>,
    //    _network: Addr<NetworkActor>,
}

impl Processor {
    pub fn new(chain_reader: Arc<AtomicRefCell<MemChain>>) -> Self {
        Processor {
            chain_reader,
            //            _network: network,
        }
    }

    pub fn head_block(&self) -> Block {
        self.chain_reader.borrow().head_block().clone()
    }

    pub fn send_latest_state_msg(&self) -> LatestStateMsg {
        //todo:send to network
        let head_block = self.head_block();
        let hash_header = HashWithBlockHeader {
            hash: head_block.crypto_hash(),
            header: head_block.header().clone(),
        };
        LatestStateMsg { hash_header }
    }

    pub fn handle_get_hash_by_number_msg(
        &self,
        get_hash_by_number_msg: GetHashByNumberMsg,
    ) -> BatchHashByNumberMsg {
        let mut hashs = Vec::new();
        for number in get_hash_by_number_msg.numbers {
            let block = self
                .chain_reader
                .borrow()
                .get_block_by_number_from_master(&number)
                .expect("block is none.");
            let hash_with_number = HashWithNumber {
                number: block.header().number(),
                hash: block.crypto_hash(),
            };

            hashs.push(hash_with_number);
        }

        BatchHashByNumberMsg { hashs }
    }

    pub fn handle_get_header_by_hash_msg(
        &self,
        get_header_by_hash_msg: GetDataByHashMsg,
    ) -> BatchHeaderMsg {
        let headers = get_header_by_hash_msg
            .hashs
            .iter()
            .map(|hash| HashWithBlockHeader {
                header: self
                    .chain_reader
                    .borrow()
                    .get_block_by_hash(hash)
                    .expect("block is none.")
                    .header()
                    .clone(),
                hash: hash.clone(),
            })
            .collect();
        BatchHeaderMsg { headers }
    }

    pub fn handle_get_body_by_hash_msg(
        &self,
        get_body_by_hash_msg: GetDataByHashMsg,
    ) -> BatchBodyMsg {
        let bodies = get_body_by_hash_msg
            .hashs
            .iter()
            .map(|hash| BlockBody {
                transactions: self
                    .chain_reader
                    .borrow()
                    .get_block_by_hash(hash)
                    .expect("block is none.")
                    .transactions()
                    .clone()
                    .to_vec(),
                hash: hash.clone(),
            })
            .collect();
        BatchBodyMsg { bodies }
    }
}
