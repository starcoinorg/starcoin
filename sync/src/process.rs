/// Sync message which inbound
use crate::message::{
    BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg,
    ProcessMessage,
};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::Result;
use atomic_refcell::AtomicRefCell;
use chain::{mem_chain::MemChain, ChainWriter};
use crypto::hash::CryptoHash;
use network::NetworkActor;
use std::hash::Hash;
use std::sync::Arc;
use types::{block::Block, peer_info::PeerInfo};

pub struct ProcessActor {
    processor: Processor,
    peer_info: Arc<PeerInfo>,
}

impl ProcessActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: Arc<AtomicRefCell<MemChain>>,
    ) -> Result<Addr<ProcessActor>> {
        let process_actor = ProcessActor {
            processor: Processor::new(chain_reader),
            peer_info,
        };
        Ok(process_actor.start())
    }
}

impl Actor for ProcessActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Process actor started");
    }
}

impl Handler<ProcessMessage> for ProcessActor {
    type Result = ();

    fn handle(&mut self, msg: ProcessMessage, ctx: &mut Self::Context) {
        match msg {
            ProcessMessage::NewPeerMsg(addr, peer_info) => {
                let latest_state_msg = self.processor.send_latest_state_msg();
                match addr {
                    Some(address) => {
                        address
                            .send(DownloadMessage::LatestStateMsg(
                                Some(ctx.address()),
                                self.peer_info.as_ref().clone(),
                                latest_state_msg,
                            ))
                            .into_actor(self)
                            .then(|_result, act, _ctx| async {}.into_actor(act))
                            .wait(ctx);
                    }
                    _ => {}
                }
            }
            ProcessMessage::GetHashByNumberMsg(addr, get_hash_by_number_msg) => {
                let batch_hash_by_number_msg = self
                    .processor
                    .handle_get_hash_by_number_msg(get_hash_by_number_msg);
                match addr {
                    Some(address) => {
                        address
                            .send(DownloadMessage::BatchHashByNumberMsg(
                                Some(ctx.address()),
                                self.peer_info.as_ref().clone(),
                                batch_hash_by_number_msg,
                            ))
                            .into_actor(self)
                            .then(|_result, act, _ctx| async {}.into_actor(act))
                            .wait(ctx);
                    }
                    _ => {}
                }
            }
            ProcessMessage::GetDataByHashMsg(addr, get_data_by_hash_msg) => {
                match get_data_by_hash_msg.data_type {
                    DataType::HEADER => {
                        let batch_header_msg = self
                            .processor
                            .handle_get_header_by_hash_msg(get_data_by_hash_msg.clone());
                        let batch_body_msg = self
                            .processor
                            .handle_get_body_by_hash_msg(get_data_by_hash_msg);
                        match addr {
                            Some(address) => {
                                address
                                    .send(DownloadMessage::BatchHeaderAndBodyMsg(
                                        batch_header_msg,
                                        batch_body_msg,
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

                // match get_data_by_hash_msg.data_type {
                //     DataType::HEADER => {
                //         let batch_header_msg = self
                //             .processor
                //             .handle_get_header_by_hash_msg(get_data_by_hash_msg);
                //         match addr {
                //             Some(address) => {
                //                 address.send(DownloadMessage::BatchHeaderMsg(
                //                     Some(ctx.address()),
                //                     self.peer_info.as_ref().clone(),
                //                     batch_header_msg,
                //                 ))
                //                 .into_actor(self)
                //                 .then(|_result, act, _ctx| async {}.into_actor(act))
                //                 .wait(ctx);
                //             }
                //             _ => {}
                //         }
                //     }
                //     DataType::BODY => {
                //         let batch_body_msg = self
                //             .processor
                //             .handle_get_body_by_hash_msg(get_data_by_hash_msg);
                //         match addr {
                //             Some(address) => {
                //                 address.send(DownloadMessage::BatchBodyMsg(
                //                     Some(ctx.address()),
                //                     batch_body_msg,
                //                 ))
                //                 .into_actor(self)
                //                 .then(|_result, act, _ctx| async {}.into_actor(act))
                //                 .wait(ctx);
                //             }
                //             _ => {}
                //         }
                //     }
                // };
            }
        }
    }
}

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
