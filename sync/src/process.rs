use crate::helper::{do_get_block_by_hash, do_get_hash_by_number, do_state_node};
use actix::prelude::*;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::ChainActorRef;
use crypto::hash::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::RawRpcRequestMessage;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::{StateNode, StateNodeStore};
/// Sync message which inbound
use starcoin_sync_api::sync_messages::{
    BatchBlockInfo, BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, BlockBody, DataType,
    GetDataByHashMsg, GetHashByNumberMsg, HashWithNumber, SyncRpcRequest,
};
use std::sync::Arc;
use traits::ChainAsyncService;
use traits::Consensus;

pub struct ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    processor: Arc<Processor<E, C>>,
    bus: Addr<BusActor>,
}

impl<E, C> ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn launch(
        chain_reader: ChainActorRef<E, C>,
        bus: Addr<BusActor>,
        state_node_storage: Arc<dyn StateNodeStore>,
    ) -> Result<Addr<ProcessActor<E, C>>> {
        let process_actor = ProcessActor {
            processor: Arc::new(Processor::new(chain_reader, state_node_storage)),
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

impl<E, C> Handler<RawRpcRequestMessage> for ProcessActor<E, C>
where
    E: TransactionExecutor + Sync + Send + 'static + Clone,
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
            }
        });

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
        nodes_hash.iter().for_each(
            |node_key| match processor.state_node_storage.get(node_key) {
                Ok(node) => state_nodes.push((node_key.clone(), node)),
                Err(e) => error!("error: {:?}", e),
            },
        );

        state_nodes
    }
}
