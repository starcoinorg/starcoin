use crate::get_txns_handler::GetTxnsHandler;
use crate::helper::{
    do_accumulator_node, do_get_body_by_hash, do_get_headers, do_get_info_by_hash, do_state_node,
};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, StreamHandler};
use anyhow::Result;
use chain::ChainActorRef;
use crypto::hash::HashValue;
use logger::prelude::*;
use network::RawRpcRequestMessage;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
/// Sync message which inbound
use starcoin_sync_api::sync_messages::{BlockBody, GetBlockHeaders, SyncRpcRequest};
use std::sync::Arc;
use traits::ChainAsyncService;
use traits::Consensus;
use txpool::TxPoolService;
use types::block::{BlockHeader, BlockInfo};

pub struct ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    processor: Arc<Processor<C>>,
}

impl<C> ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        chain_reader: ChainActorRef<C>,
        txpool: TxPoolService,
        storage: Arc<dyn Store>,
        rpc_rx: futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>,
    ) -> Result<Addr<ProcessActor<C>>> {
        Ok(ProcessActor::create(
            move |ctx: &mut Context<ProcessActor<C>>| {
                ctx.add_stream(rpc_rx);
                ProcessActor {
                    processor: Arc::new(Processor::new(chain_reader, txpool, storage)),
                }
            },
        ))
    }
}

impl<C> Actor for ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    type Context = Context<Self>;
}

impl<C> StreamHandler<RawRpcRequestMessage> for ProcessActor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn handle(&mut self, msg: RawRpcRequestMessage, _ctx: &mut Self::Context) {
        let responder = msg.responder.clone();
        let processor = self.processor.clone();
        if let Ok(req) = SyncRpcRequest::decode(msg.request.as_slice()) {
            Arbiter::spawn(async move {
                match req {
                    SyncRpcRequest::GetBlockHeaders(get_block_headers) => {
                        let headers = Processor::handle_get_block_headers_msg(
                            processor.clone(),
                            get_block_headers,
                        )
                        .await;
                        if let Err(e) = do_get_headers(responder, headers).await {
                            error!("do_get_headers request failed : {:?}", e);
                        }
                    }
                    SyncRpcRequest::GetBlockInfos(hashs) => {
                        let infos =
                            Processor::handle_get_block_info_by_hash_msg(processor.clone(), hashs)
                                .await;
                        if let Err(e) = do_get_info_by_hash(responder, infos).await {
                            error!("do_get_info_by_hash request failed : {:?}", e);
                        }
                    }
                    SyncRpcRequest::GetBlockBodies(hashs) => {
                        let bodies =
                            Processor::handle_get_body_by_hash_msg(processor.clone(), hashs).await;
                        if let Err(e) = do_get_body_by_hash(responder, bodies).await {
                            error!("do_get_body_by_hash request failed : {:?}", e);
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
                                    error!("do state_node request failed : {:?}", e);
                                }
                            } else {
                                debug!("{:?}", "state_node is none.");
                            }
                        } else {
                            debug!("{:?}", "state_nodes is none.");
                        }
                    }
                    SyncRpcRequest::GetAccumulatorNodeByNodeHash(
                        accumulator_node_key,
                        accumulator_type,
                    ) => {
                        let mut keys = Vec::new();
                        keys.push(accumulator_node_key);
                        let mut accumulator_nodes = Processor::handle_accumulator_node_msg(
                            processor.clone(),
                            keys,
                            accumulator_type,
                        )
                        .await;
                        if let Some((_, accumulator_node_res)) = accumulator_nodes.pop() {
                            if let Some(accumulator_node) = accumulator_node_res {
                                if let Err(e) =
                                    do_accumulator_node(responder, accumulator_node).await
                                {
                                    error!("do accumulator_node request failed : {:?}", e);
                                }
                            } else {
                                debug!("accumulator_node {:?} is none.", accumulator_node_key);
                            }
                        } else {
                            debug!("{:?}", "accumulator_nodes is none.");
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
        }
    }
}

/// Process request for syncing block
pub struct Processor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<C>,
    txpool: TxPoolService,
    storage: Arc<dyn Store>,
}

impl<C> Processor<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(
        chain_reader: ChainActorRef<C>,
        txpool: TxPoolService,
        storage: Arc<dyn Store>,
    ) -> Self {
        Processor {
            chain_reader,
            txpool,
            storage,
        }
    }

    pub async fn handle_get_block_headers_msg(
        processor: Arc<Processor<C>>,
        get_block_headers: GetBlockHeaders,
    ) -> Vec<BlockHeader> {
        let mut headers = Vec::new();
        if let Ok(Some(header)) = processor
            .chain_reader
            .clone()
            .get_header_by_hash(&get_block_headers.block_id)
            .await
        {
            let mut last_number = header.number();
            while headers.len() < get_block_headers.max_size {
                let block_number = if get_block_headers.reverse {
                    if last_number > get_block_headers.step as u64 {
                        last_number - get_block_headers.step as u64
                    } else {
                        0
                    }
                } else {
                    last_number + get_block_headers.step as u64
                };
                if let Ok(header) = processor
                    .chain_reader
                    .clone()
                    .master_block_header_by_number(block_number)
                    .await
                {
                    headers.push(header);
                } else {
                    break;
                }

                if block_number == 0 {
                    break;
                }
                last_number = block_number;
            }
        }
        headers
    }

    pub async fn handle_get_body_by_hash_msg(
        processor: Arc<Processor<C>>,
        hashs: Vec<HashValue>,
    ) -> Vec<BlockBody> {
        let mut bodies = Vec::new();
        for hash in hashs {
            let transactions = match processor.chain_reader.clone().get_block_by_hash(hash).await {
                Ok(block) => block.transactions().to_vec(),
                _ => Vec::new(),
            };

            let body = BlockBody { transactions, hash };

            bodies.push(body);
        }
        bodies
    }

    pub async fn handle_get_block_info_by_hash_msg(
        processor: Arc<Processor<C>>,
        hashs: Vec<HashValue>,
    ) -> Vec<BlockInfo> {
        let mut infos = Vec::new();
        for hash in hashs {
            if let Ok(Some(block_info)) = processor
                .chain_reader
                .clone()
                .get_block_info_by_hash(&hash)
                .await
            {
                infos.push(block_info);
            }
        }
        infos
    }

    pub async fn handle_state_node_msg(
        processor: Arc<Processor<C>>,
        nodes_hash: Vec<HashValue>,
    ) -> Vec<(HashValue, Option<StateNode>)> {
        let mut state_nodes = Vec::new();
        nodes_hash
            .iter()
            .for_each(|node_key| match processor.storage.get(node_key) {
                Ok(node) => state_nodes.push((*node_key, node)),
                Err(e) => error!("handle state_node {:?} err : {:?}", node_key, e),
            });

        state_nodes
    }

    pub async fn handle_accumulator_node_msg(
        processor: Arc<Processor<C>>,
        nodes_hash: Vec<HashValue>,
        accumulator_type: AccumulatorStoreType,
    ) -> Vec<(HashValue, Option<AccumulatorNode>)> {
        let mut accumulator_nodes = Vec::new();
        nodes_hash.iter().for_each(|node_key| {
            match processor
                .storage
                .get_node(accumulator_type.clone(), *node_key)
            {
                Ok(node) => accumulator_nodes.push((*node_key, node)),
                Err(e) => error!("handle accumulator_node {:?} err : {:?}", node_key, e),
            }
        });

        accumulator_nodes
    }
}
