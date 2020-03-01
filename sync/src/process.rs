use actix::prelude::*;
use actix::{
    fut::wrap_future, fut::FutureWrap, Actor, Addr, AsyncContext, Context, Handler,
    ResponseActFuture,
};
use anyhow::Result;
use bus::{Bus, BusActor, Subscription};
use chain::{ChainActor, ChainActorRef};
use crypto::{hash::CryptoHash, HashValue};
use futures::compat::Future01CompatExt;
use futures_locks::{Mutex, RwLock};
/// Sync message which inbound
use network::sync_messages::{
    BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, DownloadMessage,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithBlockHeader, HashWithNumber, LatestStateMsg,
    ProcessMessage,
};
use network::{
    NetworkAsyncService, PeerMessage, RPCMessage, RPCRequest, RPCResponse, RpcRequestMessage,
};
use std::hash::Hash;
use std::sync::Arc;
use traits::{AsyncChain, Chain, ChainAsyncService, ChainReader, ChainService};
use txpool::TxPoolRef;
use types::{block::Block, peer_info::PeerInfo};
use futures_timer::Delay;
use std::time::Duration;

pub struct ProcessActor {
    processor: Arc<RwLock<Processor>>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService<TxPoolRef>,
    bus: Addr<BusActor>,
}

impl ProcessActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: ChainActorRef<ChainActor>,
        network: NetworkAsyncService<TxPoolRef>,
        bus: Addr<BusActor>,
    ) -> Result<Addr<ProcessActor>> {
        let process_actor = ProcessActor {
            processor: Arc::new(RwLock::new(Processor::new(chain_reader))),
            peer_info,
            network,
            bus,
        };
        Ok(process_actor.start())
    }
}

impl Actor for ProcessActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rpc_recipient = ctx.address().recipient::<RpcRequestMessage>();
        self.bus
            .send(Subscription {
                recipient: rpc_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        println!("Process actor started");
    }
}

impl Handler<ProcessMessage> for ProcessActor {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: ProcessMessage, ctx: &mut Self::Context) -> Self::Result {
        let processor = self.processor.clone();
        let my_addr = ctx.address();
        let my_peer_info = self.peer_info.as_ref().clone();
        let network = self.network.clone();
        let fut = async move {
            let id = msg.crypto_hash();
            match msg {
                ProcessMessage::NewPeerMsg(peer_info) => {
                    println!("send latest_state_msg to peer : {:?}:{:?}", peer_info.id, my_peer_info.id);
                    let latest_state_msg =
                        Processor::send_latest_state_msg(processor.clone()).await;
                    Delay::new(Duration::from_secs(1)).await;
                    network
                        .clone()
                        .send_peer_message(
                            peer_info.id,
                            PeerMessage::LatestStateMsg(latest_state_msg),
                        )
                        .await;
                }
                // ProcessMessage::GetHashByNumberMsg(peer, get_hash_by_number_msg) => {
                //     println!("get_hash_by_number_msg");
                //     let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                //         processor.clone(),
                //         get_hash_by_number_msg,
                //     )
                //         .await;
                //
                //     let resp = RPCResponse::BatchHashByNumberMsg(batch_hash_by_number_msg);
                //     network.clone().response_for(peer, id, resp).await;
                // match addr {
                //     Some(address) => {
                //         address
                //             .send(DownloadMessage::BatchHashByNumberMsg(
                //                 Some(my_addr),
                //                 peer_info,
                //                 batch_hash_by_number_msg,
                //             ))
                //             .await;
                //     }
                //     _ => {}
                // }
                // }
                // ProcessMessage::GetDataByHashMsg(addr, get_data_by_hash_msg) => {
                //     match get_data_by_hash_msg.data_type {
                //         DataType::HEADER => {
                //             let batch_header_msg = Processor::handle_get_header_by_hash_msg(
                //                 processor.clone(),
                //                 get_data_by_hash_msg.clone(),
                //             )
                //                 .await;
                //             let batch_body_msg = Processor::handle_get_body_by_hash_msg(
                //                 processor.clone(),
                //                 get_data_by_hash_msg,
                //             )
                //                 .await;
                //             println!(
                //                 "batch block size: {} : {}",
                //                 batch_header_msg.headers.len(),
                //                 batch_body_msg.bodies.len()
                //             );
                //             match addr {
                //                 Some(address) => {
                //                     address
                //                         .send(DownloadMessage::BatchHeaderAndBodyMsg(
                //                             batch_header_msg,
                //                             batch_body_msg,
                //                         ))
                //                         .await;
                //                 }
                //                 _ => {}
                //             }
                //         }
                //         _ => {}
                //     }
                //
                //     // match get_data_by_hash_msg.data_type {
                //     //     DataType::HEADER => {
                //     //         let batch_header_msg = Processor::handle_get_header_by_hash_msg(processor.clone(), get_data_by_hash_msg);
                //     //         match addr {
                //     //             Some(address) => {
                //     //                 address.send(DownloadMessage::BatchHeaderMsg(
                //     //                     Some(my_addr),
                //     //                     peer_info,
                //     //                     batch_header_msg,
                //     //                 ))
                //     //                 .await;
                //     //             }
                //     //             _ => {}
                //     //         }
                //     //     }
                //     //     DataType::BODY => {
                //     //         let batch_body_msg = Processor::handle_get_body_by_hash_msg(processor.clone(), get_data_by_hash_msg);
                //     //         match addr {
                //     //             Some(address) => {
                //     //                 address.send(DownloadMessage::BatchBodyMsg(
                //     //                     Some(my_addr),
                //     //                     batch_body_msg,
                //     //                 ))
                //     //                 .await;
                //     //             }
                //     //             _ => {}
                //     //         }
                //     //     }
                //     // };
                // }
                _ => {}
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

impl Handler<RpcRequestMessage> for ProcessActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: RpcRequestMessage, ctx: &mut Self::Context) -> Self::Result {
        let id = (&msg.request).get_id();
        let peer_id = (&msg).peer_id;
        let processor = self.processor.clone();
        let network = self.network.clone();
        match msg.request {
            RPCRequest::TestRequest(_r) => {}
            RPCRequest::GetHashByNumberMsg(process_msg) => match process_msg {
                ProcessMessage::GetHashByNumberMsg(peer, get_hash_by_number_msg) => {
                    println!("get_hash_by_number_msg");
                    Arbiter::spawn(async move {
                        let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                            id.clone(),
                            processor.clone(),
                            get_hash_by_number_msg,
                        )
                            .await;

                        let resp = RPCResponse::BatchHashByNumberMsg(batch_hash_by_number_msg);
                        network.clone().response_for(peer, id, resp).await;
                    });
                }
                _ => {}
            },
        }

        Ok(())
    }
}

/// Process request for syncing block
pub struct Processor {
    //    chain: Addr<ChainActor>,
    //    _network: Addr<NetworkActor>,
    chain_reader: ChainActorRef<ChainActor>,
}

impl Processor {
    pub fn new(chain_reader: ChainActorRef<ChainActor>) -> Self {
        Processor {
            chain_reader,
            //            _network: network,
        }
    }

    pub async fn head_block(processor: Arc<RwLock<Processor>>) -> Block {
        let lock = processor.read().compat().await.unwrap();
        lock.chain_reader.clone().head_block().await.unwrap()
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
        req_id: HashValue,
        processor: Arc<RwLock<Processor>>,
        get_hash_by_number_msg: GetHashByNumberMsg,
    ) -> BatchHashByNumberMsg {
        let lock = processor.read().compat().await.unwrap();
        let mut hashs = Vec::new();
        for number in get_hash_by_number_msg.numbers {
            let block = lock
                .chain_reader
                .clone()
                .get_block_by_number(number)
                .await
                .unwrap();
            println!(
                "block number:{:?}, hash {:?}",
                block.header().number(),
                block.crypto_hash()
            );
            let hash_with_number = HashWithNumber {
                number: block.header().number(),
                hash: block.crypto_hash(),
            };

            hashs.push(hash_with_number);
        }

        BatchHashByNumberMsg { id: req_id, hashs }
    }

    pub async fn handle_get_header_by_hash_msg(
        processor: Arc<RwLock<Processor>>,
        get_header_by_hash_msg: GetDataByHashMsg,
    ) -> BatchHeaderMsg {
        let lock = processor.read().compat().await.unwrap();

        let mut headers = Vec::new();
        for hash in get_header_by_hash_msg.hashs {
            let header = lock
                .chain_reader
                .clone()
                .get_header_by_hash(&hash)
                .await
                .unwrap();
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
            let transactions = match lock.chain_reader.clone().get_block_by_hash(&hash).await {
                Some(block) => block.transactions().clone().to_vec(),
                _ => Vec::new(),
            };

            let body = BlockBody { transactions, hash };

            bodies.push(body);
        }
        BatchBodyMsg { bodies }
    }
}
