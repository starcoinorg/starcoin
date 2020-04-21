use crate::get_txns_handler::GetTxnsHandler;
use crate::helper::{
    do_accumulator_node, do_get_block_by_hash, do_get_hash_by_number, do_state_node,
};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use crypto::hash::HashValue;
use logger::prelude::*;
use network::RawRpcRequestMessage;
use starcoin_accumulator::AccumulatorNode;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
/// Sync message which inbound
use starcoin_sync_api::sync_messages::{
    BatchBlockInfo, BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithNumber, SyncRpcRequest,
};
use std::sync::Arc;
use traits::ChainAsyncService;
use traits::Consensus;
use txpool::TxPoolRef;

pub struct ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    processor: Arc<Processor<C>>,
    bus: Addr<BusActor>,
}

impl<C> ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        chain_reader: ChainActorRef<C>,
        txpool: TxPoolRef,
        bus: Addr<BusActor>,
        storage: Arc<dyn Store>,
    ) -> Result<Addr<ProcessActor<C>>> {
        let process_actor = ProcessActor {
            processor: Arc::new(Processor::new(chain_reader, txpool, storage)),
            bus,
        };
        Ok(process_actor.start())
    }
}

impl<C> Actor for ProcessActor<C>
where
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

impl<C> Handler<RawRpcRequestMessage> for ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: RawRpcRequestMessage, _ctx: &mut Self::Context) -> Self::Result {
        let responder = msg.responder.clone();
        let processor = self.processor.clone();
        let req = SyncRpcRequest::decode(msg.request.as_slice())?;
        Arbiter::spawn(async move {
            info!("process req :{:?}", req);
            match req {
                SyncRpcRequest::GetHashByNumberMsg(get_hash_by_number_msg) => {
                    let batch_hash_by_number_msg = Processor::handle_get_hash_by_number_msg(
                        processor.clone(),
                        get_hash_by_number_msg,
                    )
                    .await;
                    if let Err(e) = do_get_hash_by_number(responder, batch_hash_by_number_msg).await
                    {
                        error!("error: {:?}", e);
                    }
                }
                SyncRpcRequest::GetDataByHashMsg(get_data_by_hash_msg) => {
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

                            if let Err(e) = do_get_block_by_hash(
                                responder,
                                batch_header_msg,
                                batch_body_msg,
                                batch_block_info_msg,
                            )
                            .await
                            {
                                error!("error: {:?}", e);
                            }
                        }
                        _ => {}
                    }
                }
                SyncRpcRequest::GetStateNodeByNodeHash(state_node_key) => {
                    let mut keys = Vec::new();
                    keys.push(state_node_key);
                    let mut state_nodes =
                        Processor::handle_state_node_msg(processor.clone(), keys).await;
                    if let Some((_, state_node_res)) = state_nodes.pop() {
                        if let Some(state_node) = state_node_res {
                            if let Err(e) = do_state_node(responder, state_node).await {
                                error!("error: {:?}", e);
                            }
                        } else {
                            warn!("{:?}", "state_node is none.");
                        }
                    } else {
                        warn!("{:?}", "state_nodes is none.");
                    }
                }
                SyncRpcRequest::GetAccumulatorNodeByNodeHash(accumulator_node_key) => {
                    let mut keys = Vec::new();
                    keys.push(accumulator_node_key);
                    let mut accumulator_nodes =
                        Processor::handle_accumulator_node_msg(processor.clone(), keys).await;
                    if let Some((_, accumulator_node_res)) = accumulator_nodes.pop() {
                        if let Some(accumulator_node) = accumulator_node_res {
                            if let Err(e) = do_accumulator_node(responder, accumulator_node).await {
                                error!("error: {:?}", e);
                            }
                        } else {
                            warn!("accumulator_node {:?} is none.", accumulator_node_key);
                        }
                    } else {
                        warn!("{:?}", "accumulator_nodes is none.");
                    }
                }
                SyncRpcRequest::GetTxns(msg) => {
                    let handler = GetTxnsHandler::new(processor.txpool.clone());
                    let result = handler.handle(responder, msg).await;
                    if let Err(e) = result {
                        warn!("handle get txn fail, error: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

/// Process request for syncing block
pub struct Processor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<C>,
    txpool: TxPoolRef,
    storage: Arc<dyn Store>,
}

impl<C> Processor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>, txpool: TxPoolRef, storage: Arc<dyn Store>) -> Self {
        Processor {
            chain_reader,
            txpool,
            storage,
        }
    }

    pub async fn handle_get_hash_by_number_msg(
        processor: Arc<Processor<C>>,
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
                Ok(b) => {
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
                Err(_) => {
                    warn!("block is none.");
                }
            }
        }

        BatchHashByNumberMsg { hashs }
    }

    pub async fn handle_get_header_by_hash_msg(
        processor: Arc<Processor<C>>,
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
        processor: Arc<Processor<C>>,
        get_body_by_hash_msg: GetDataByHashMsg,
    ) -> BatchBodyMsg {
        let mut bodies = Vec::new();
        for hash in get_body_by_hash_msg.hashs {
            let transactions = match processor.chain_reader.clone().get_block_by_hash(hash).await {
                Ok(block) => block.transactions().clone().to_vec(),
                _ => Vec::new(),
            };

            let body = BlockBody { transactions, hash };

            bodies.push(body);
        }
        BatchBodyMsg { bodies }
    }

    pub async fn handle_get_block_info_by_hash_msg(
        processor: Arc<Processor<C>>,
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
        processor: Arc<Processor<C>>,
        nodes_hash: Vec<HashValue>,
    ) -> Vec<(HashValue, Option<StateNode>)> {
        let mut state_nodes = Vec::new();
        nodes_hash
            .iter()
            .for_each(|node_key| match processor.storage.get(node_key) {
                Ok(node) => state_nodes.push((node_key.clone(), node)),
                Err(e) => error!("error: {:?}", e),
            });

        state_nodes
    }

    pub async fn handle_accumulator_node_msg(
        processor: Arc<Processor<C>>,
        nodes_hash: Vec<HashValue>,
    ) -> Vec<(HashValue, Option<AccumulatorNode>)> {
        let mut accumulator_nodes = Vec::new();
        nodes_hash.iter().for_each(
            |node_key| match processor.storage.get_node(node_key.clone()) {
                Ok(node) => accumulator_nodes.push((node_key.clone(), node)),
                Err(e) => error!("error: {:?}", e),
            },
        );

        accumulator_nodes
    }
}
