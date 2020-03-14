use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use crypto::hash::CryptoHash;
use futures::sink::SinkExt;
use futures_timer::Delay;
/// Sync message which inbound
use network::sync_messages::{
    BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType, GetDataByHashMsg,
    GetHashByNumberMsg, HashWithNumber, LatestStateMsg, ProcessMessage,
};
use network::{
    NetworkAsyncService, PeerMessage, RPCMessage, RPCRequest, RPCResponse, RpcRequestMessage,
};
use std::sync::Arc;
use std::time::Duration;
use traits::AsyncChain;
use txpool::TxPoolRef;
use types::{block::Block, peer_info::PeerInfo};

pub struct ProcessActor {
    processor: Arc<Processor>,
    peer_info: Arc<PeerInfo>,
    network: NetworkAsyncService<TxPoolRef>,
    bus: Addr<BusActor>,
}

impl ProcessActor {
    pub fn launch(
        peer_info: Arc<PeerInfo>,
        chain_reader: ChainActorRef,
        network: NetworkAsyncService<TxPoolRef>,
        bus: Addr<BusActor>,
    ) -> Result<Addr<ProcessActor>> {
        let process_actor = ProcessActor {
            processor: Arc::new(Processor::new(chain_reader)),
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
        info!("Process actor started");
    }
}

impl Handler<ProcessMessage> for ProcessActor {
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: ProcessMessage, _ctx: &mut Self::Context) -> Self::Result {
        let processor = self.processor.clone();
        let my_peer_info = self.peer_info.as_ref().clone();
        let network = self.network.clone();
        let fut = async move {
            let id = msg.crypto_hash();
            match msg {
                ProcessMessage::NewPeerMsg(peer_info) => {
                    info!(
                        "send latest_state_msg to peer : {:?}:{:?}, message id is {:?}",
                        peer_info.id, my_peer_info.id, id
                    );
                    let latest_state_msg =
                        Processor::send_latest_state_msg(processor.clone()).await;
                    Delay::new(Duration::from_secs(1)).await;
                    if let Err(e) = network
                        .clone()
                        .send_peer_message(
                            peer_info.id.into(),
                            PeerMessage::LatestStateMsg(latest_state_msg),
                        )
                        .await
                    {
                        warn!("err :{:?}", e);
                    }
                }
                _ => {}
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

impl Handler<RpcRequestMessage> for ProcessActor {
    type Result = Result<()>;

    fn handle(&mut self, msg: RpcRequestMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut responder = msg.responder.clone();
        let processor = self.processor.clone();
        let id = msg.request.get_id();
        match msg.request {
            RPCRequest::TestRequest(_r) => {}
            RPCRequest::GetHashByNumberMsg(process_msg)
            | RPCRequest::GetDataByHashMsg(process_msg) => match process_msg {
                ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg) => {
                    info!(
                        "get_hash_by_number_msg:{:?}, do request id : {:?} begin",
                        get_hash_by_number_msg, id
                    );
                    Arbiter::spawn(async move {
                        let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                            processor.clone(),
                            get_hash_by_number_msg,
                        )
                        .await;

                        let resp = RPCResponse::BatchHashByNumberMsg(batch_hash_by_number_msg);

                        responder.send(resp).await.unwrap();

                        info!("do request id : {:?} end", id);
                    });
                }
                ProcessMessage::GetDataByHashMsg(get_data_by_hash_msg) => {
                    Arbiter::spawn(async move {
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
                                debug!(
                                    "batch block size: {} : {}",
                                    batch_header_msg.headers.len(),
                                    batch_body_msg.bodies.len()
                                );

                                let resp = RPCResponse::BatchHeaderAndBodyMsg(
                                    batch_header_msg,
                                    batch_body_msg,
                                );
                                responder.send(resp).await.unwrap();
                            }
                            _ => {}
                        }
                    });
                }
                ProcessMessage::NewPeerMsg(_) => unreachable!(),
            },
        }

        Ok(())
    }
}

/// Process request for syncing block
pub struct Processor {
    chain_reader: ChainActorRef,
}

impl Processor {
    pub fn new(chain_reader: ChainActorRef) -> Self {
        Processor { chain_reader }
    }

    pub async fn head_block(processor: Arc<Processor>) -> Block {
        processor.chain_reader.clone().head_block().await.unwrap()
    }

    pub async fn send_latest_state_msg(processor: Arc<Processor>) -> LatestStateMsg {
        let head_block = Self::head_block(processor.clone()).await;
        //todo:send to network
        LatestStateMsg {
            header: head_block.header().clone(),
        }
    }

    pub async fn handle_get_hash_by_number_msg(
        processor: Arc<Processor>,
        get_hash_by_number_msg: GetHashByNumberMsg,
    ) -> BatchHashByNumberMsg {
        let mut hashs = Vec::new();
        for number in get_hash_by_number_msg.numbers {
            info!("get block from get_block_by_number with {}", number);
            let block = processor
                .chain_reader
                .clone()
                .get_block_by_number(number)
                .await
                .expect("block is none after get_block_by_number");
            debug!(
                "block number:{:?}, hash {:?}",
                block.header().number(),
                block.header().id()
            );
            let hash_with_number = HashWithNumber {
                number: block.header().number(),
                hash: block.header().id(),
            };

            hashs.push(hash_with_number);
        }

        BatchHashByNumberMsg { hashs }
    }

    pub async fn handle_get_header_by_hash_msg(
        processor: Arc<Processor>,
        get_header_by_hash_msg: GetDataByHashMsg,
    ) -> BatchHeaderMsg {
        let mut headers = Vec::new();
        for hash in get_header_by_hash_msg.hashs {
            let header = processor
                .chain_reader
                .clone()
                .get_header_by_hash(&hash)
                .await
                .unwrap();
            headers.push(header);
        }
        BatchHeaderMsg { headers }
    }

    pub async fn handle_get_body_by_hash_msg(
        processor: Arc<Processor>,
        get_body_by_hash_msg: GetDataByHashMsg,
    ) -> BatchBodyMsg {
        let mut bodies = Vec::new();
        for hash in get_body_by_hash_msg.hashs {
            let transactions = match processor
                .chain_reader
                .clone()
                .get_block_by_hash(&hash)
                .await
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
