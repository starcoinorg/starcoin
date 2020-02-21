/// Sync message which inbound
use crate::message::{
    BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg,
    ProcessMessage,
};
use actix::prelude::*;
use actix::{
    fut::wrap_future, fut::FutureWrap, Actor, Addr, AsyncContext, Context, Handler,
    ResponseActFuture,
};
use anyhow::Result;
use chain::{mem_chain::MemChainActor, ChainActorRef};
use crypto::hash::CryptoHash;
use futures::compat::Future01CompatExt;
use futures_locks::{Mutex, RwLock};
use network::NetworkActor;
use std::hash::Hash;
use std::sync::Arc;
use traits::Chain;
use types::{block::Block, peer_info::PeerInfo};

pub struct ProcessActor {
    processor: Arc<RwLock<Processor>>,
    peer_info: Arc<PeerInfo>,
}

impl ProcessActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: ChainActorRef<MemChainActor>,
    ) -> Result<Addr<ProcessActor>> {
        let process_actor = ProcessActor {
            processor: Arc::new(RwLock::new(Processor::new(chain_reader))),
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
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: ProcessMessage, ctx: &mut Self::Context) -> Self::Result {
        let processor = self.processor.clone();
        let my_addr = ctx.address();
        let peer_info = self.peer_info.as_ref().clone();
        let fut = async move {
            match msg {
                ProcessMessage::NewPeerMsg(addr, peer_info) => {
                    let latest_state_msg =
                        Processor::send_latest_state_msg(processor.clone()).await;
                    match addr {
                        Some(address) => {
                            address
                                .send(DownloadMessage::LatestStateMsg(
                                    Some(my_addr),
                                    peer_info,
                                    latest_state_msg,
                                ))
                                .await;
                        }
                        _ => {}
                    }
                }
                ProcessMessage::GetHashByNumberMsg(addr, get_hash_by_number_msg) => {
                    let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                        processor.clone(),
                        get_hash_by_number_msg,
                    )
                    .await;
                    match addr {
                        Some(address) => {
                            address
                                .send(DownloadMessage::BatchHashByNumberMsg(
                                    Some(my_addr),
                                    peer_info,
                                    batch_hash_by_number_msg,
                                ))
                                .await;
                        }
                        _ => {}
                    }
                }
                ProcessMessage::GetDataByHashMsg(addr, get_data_by_hash_msg) => {
                    match get_data_by_hash_msg.data_type {
                        DataType::HEADER => {
                            let batch_header_msg = Processor::handle_get_header_by_hash_msg(
                                processor.clone(),
                                get_data_by_hash_msg.clone(),
                            )
                            .await;
                            let batch_body_msg = Processor::handle_get_body_by_hash_msg(
                                processor.clone(),
                                get_data_by_hash_msg,
                            )
                            .await;
                            match addr {
                                Some(address) => {
                                    address
                                        .send(DownloadMessage::BatchHeaderAndBodyMsg(
                                            batch_header_msg,
                                            batch_body_msg,
                                        ))
                                        .await;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }

                    // match get_data_by_hash_msg.data_type {
                    //     DataType::HEADER => {
                    //         let batch_header_msg = Processor::handle_get_header_by_hash_msg(processor.clone(), get_data_by_hash_msg);
                    //         match addr {
                    //             Some(address) => {
                    //                 address.send(DownloadMessage::BatchHeaderMsg(
                    //                     Some(my_addr),
                    //                     peer_info,
                    //                     batch_header_msg,
                    //                 ))
                    //                 .await;
                    //             }
                    //             _ => {}
                    //         }
                    //     }
                    //     DataType::BODY => {
                    //         let batch_body_msg = Processor::handle_get_body_by_hash_msg(processor.clone(), get_data_by_hash_msg);
                    //         match addr {
                    //             Some(address) => {
                    //                 address.send(DownloadMessage::BatchBodyMsg(
                    //                     Some(my_addr),
                    //                     batch_body_msg,
                    //                 ))
                    //                 .await;
                    //             }
                    //             _ => {}
                    //         }
                    //     }
                    // };
                }
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

/// Process request for syncing block
pub struct Processor {
    //    chain: Addr<ChainActor>,
    //    _network: Addr<NetworkActor>,
    chain_reader: ChainActorRef<MemChainActor>,
}

impl Processor {
    pub fn new(chain_reader: ChainActorRef<MemChainActor>) -> Self {
        Processor {
            chain_reader,
            //            _network: network,
        }
    }

    pub async fn head_block(processor: Arc<RwLock<Processor>>) -> Block {
        let lock = processor.read().compat().await.unwrap();
        lock.chain_reader.clone().head_block().await
    }

    pub async fn send_latest_state_msg(processor: Arc<RwLock<Processor>>) -> LatestStateMsg {
        let head_block = Self::head_block(processor.clone()).await;
        let lock = processor.read().compat().await.unwrap();
        //todo:send to network
        let hash_header = HashWithBlockHeader {
            hash: head_block.crypto_hash(),
            header: head_block.header().clone(),
        };
        LatestStateMsg { hash_header }
    }

    pub async fn handle_get_hash_by_number_msg(
        processor: Arc<RwLock<Processor>>,
        get_hash_by_number_msg: GetHashByNumberMsg,
    ) -> BatchHashByNumberMsg {
        let lock = processor.read().compat().await.unwrap();
        let mut hashs = Vec::new();
        for number in get_hash_by_number_msg.numbers {
            let block = lock.chain_reader.clone().get_block_by_number(number).await;
            let hash_with_number = HashWithNumber {
                number: block.header().number(),
                hash: block.crypto_hash(),
            };

            hashs.push(hash_with_number);
        }

        BatchHashByNumberMsg { hashs }
    }

    pub async fn handle_get_header_by_hash_msg(
        processor: Arc<RwLock<Processor>>,
        get_header_by_hash_msg: GetDataByHashMsg,
    ) -> BatchHeaderMsg {
        let lock = processor.read().compat().await.unwrap();

        let mut headers = Vec::new();
        for hash in get_header_by_hash_msg.hashs {
            let header = lock.chain_reader.clone().get_header(&hash).await;
            let header = HashWithBlockHeader { header, hash };

            headers.push(header);
        }
        BatchHeaderMsg { headers }
    }

    pub async fn handle_get_body_by_hash_msg(
        processor: Arc<RwLock<Processor>>,
        get_body_by_hash_msg: GetDataByHashMsg,
    ) -> BatchBodyMsg {
        let lock = processor.read().compat().await.unwrap();

        let mut bodies = Vec::new();
        for hash in get_body_by_hash_msg.hashs {
            let transactions = match lock
                .chain_reader
                .clone()
                .get_block_by_hash(&hash)
                .await
                .expect("block is none.")
            {
                Some(block) => block.transactions().clone().to_vec(),
                _ => Vec::new(),
            };

            let body = BlockBody { transactions, hash };

            bodies.push(body);
        }
        BatchBodyMsg { bodies }
    }
}
