use actix::prelude::*;
use actix::{fut::wrap_future, Actor, Addr, AsyncContext, Context, Handler, ResponseActFuture};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use futures::sink::SinkExt;
use logger::prelude::*;
use network::{NetworkAsyncService, RawRpcRequestMessage};
/// Sync message which inbound
use network_p2p_api::sync_messages::{
    BatchBlockInfo, BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithNumber, ProcessMessage, SyncRpcRequest,
    SyncRpcResponse,
};
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::{StateNode, StateNodeStore};
use std::sync::Arc;
use traits::ChainAsyncService;
use traits::Consensus;
use types::peer_info::PeerId;

pub struct ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    processor: Arc<Processor<E, C>>,
    self_peer_id: Arc<PeerId>,
    network: NetworkAsyncService,
    bus: Addr<BusActor>,
}

impl<E, C> ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        peer_id: Arc<PeerId>,
        chain_reader: ChainActorRef<E, C>,
        network: NetworkAsyncService,
        bus: Addr<BusActor>,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) -> Result<Addr<ProcessActor<E, C>>> {
        let process_actor = ProcessActor {
            processor: Arc::new(Processor::new(chain_reader, state_node_storage)),
            self_peer_id: peer_id,
            network,
            bus,
        };
        Ok(process_actor.start())
    }
}

impl<E, C> Actor for ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let rpc_recipient = ctx.address().recipient::<RawRpcRequestMessage>();
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

impl<E, C> Handler<ProcessMessage> for ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = ResponseActFuture<Self, Result<()>>;

    fn handle(&mut self, msg: ProcessMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _processor = self.processor.clone();
        let _self_peer_id = self.self_peer_id.as_ref().clone();
        let _network = self.network.clone();
        let fut = async move {
            match msg {
                _ => {}
            }

            Ok(())
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

impl<E, C> Handler<RawRpcRequestMessage> for ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: RawRpcRequestMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut responder = msg.responder.clone();
        let processor = self.processor.clone();
        let req = SyncRpcRequest::decode(msg.request.as_slice())?;
        match req {
            SyncRpcRequest::GetHashByNumberMsg(process_msg)
            | SyncRpcRequest::GetDataByHashMsg(process_msg) => match process_msg {
                ProcessMessage::GetHashByNumberMsg(get_hash_by_number_msg) => {
                    info!(
                        "get_hash_by_number_msg:{:?}, do request begin",
                        get_hash_by_number_msg
                    );
                    Arbiter::spawn(async move {
                        let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                            processor.clone(),
                            get_hash_by_number_msg,
                        )
                        .await;

                        let resp = SyncRpcResponse::encode(&SyncRpcResponse::BatchHashByNumberMsg(
                            batch_hash_by_number_msg,
                        ))
                        .unwrap();

                        responder.send(resp).await.unwrap();

                        info!("do request end");
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
                                    get_data_by_hash_msg.clone(),
                                )
                                .await;
                                let batch_block_info_msg =
                                    Processor::handle_get_block_info_by_hash_msg(
                                        processor.clone(),
                                        get_data_by_hash_msg,
                                    )
                                    .await;
                                debug!(
                                    "batch block size: {} : {} : {}",
                                    batch_header_msg.headers.len(),
                                    batch_body_msg.bodies.len(),
                                    batch_block_info_msg.infos.len()
                                );

                                let resp = SyncRpcResponse::encode(
                                    &SyncRpcResponse::BatchHeaderAndBodyMsg(
                                        batch_header_msg,
                                        batch_body_msg,
                                        batch_block_info_msg,
                                    ),
                                )
                                .unwrap();
                                responder.send(resp).await.unwrap();
                            }
                            _ => {}
                        }
                    });
                }
            },
            SyncRpcRequest::GetStateNodeByNodeHash(state_node_key) => {
                Arbiter::spawn(async move {
                    let mut keys = Vec::new();
                    keys.push(state_node_key);
                    let mut state_nodes =
                        Processor::handle_state_node_msg(processor.clone(), keys).await;
                    let resp = SyncRpcResponse::encode(&SyncRpcResponse::GetStateNodeByNodeHash(
                        state_nodes
                            .pop()
                            .expect("state_nodes is none.")
                            .1
                            .expect("state_node is none."),
                    ))
                    .unwrap();
                    responder.send(resp).await.unwrap();
                });
            }
        }

        Ok(())
    }
}

/// Process request for syncing block
pub struct Processor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<E, C>,
    state_node_storage: Arc<dyn StateNodeStore>,
}

impl<E, C> Processor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(
        chain_reader: ChainActorRef<E, C>,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) -> Self {
        Processor {
            chain_reader,
            state_node_storage,
        }
    }

    pub async fn handle_get_hash_by_number_msg(
        processor: Arc<Processor<E, C>>,
        get_hash_by_number_msg: GetHashByNumberMsg,
    ) -> BatchHashByNumberMsg {
        let mut hashs = Vec::new();
        for number in get_hash_by_number_msg.numbers {
            info!("get block from get_block_by_number with {}", number);
            let block = processor
                .chain_reader
                .clone()
                .master_block_by_number(number)
                .await;
            match block {
                Some(b) => {
                    debug!(
                        "block number:{:?}, hash {:?}",
                        b.header().number(),
                        b.header().id()
                    );
                    let hash_with_number = HashWithNumber {
                        number: b.header().number(),
                        hash: b.header().id(),
                    };

                    hashs.push(hash_with_number);
                }
                None => {
                    warn!("block is none.");
                }
            }
        }

        BatchHashByNumberMsg { hashs }
    }

    pub async fn handle_get_header_by_hash_msg(
        processor: Arc<Processor<E, C>>,
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
        processor: Arc<Processor<E, C>>,
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

    pub async fn handle_get_block_info_by_hash_msg(
        processor: Arc<Processor<E, C>>,
        get_body_by_hash_msg: GetDataByHashMsg,
    ) -> BatchBlockInfo {
        let mut infos = Vec::new();
        for hash in get_body_by_hash_msg.hashs {
            if let Some(block_info) = processor
                .chain_reader
                .clone()
                .get_block_info_by_hash(&hash)
                .await
            {
                infos.push(block_info);
            }
        }
        BatchBlockInfo { infos }
    }

    pub async fn handle_state_node_msg(
        processor: Arc<Processor<E, C>>,
        nodes_hash: Vec<HashValue>,
    ) -> Vec<(HashValue, Option<StateNode>)> {
        let mut state_nodes = Vec::new();
        nodes_hash.iter().for_each(|node_key| {
            let node = processor
                .state_node_storage
                .get(node_key)
                .expect("Get state node err.");
            state_nodes.push((node_key.clone(), node));
        });

        state_nodes
    }
}
