use accumulator::AccumulatorNode;
use anyhow::*;
use chain::ChainActorRef;
use crypto::HashValue;
use futures::future::BoxFuture;
use logger::prelude::*;
use starcoin_state_tree::StateNode;
use std::sync::Arc;
use storage::Store;
use traits::{ChainAsyncService, Consensus};
use txpool::TxPoolService;
use types::{
    block::{BlockHeader, BlockInfo},
    peer_info::PeerId,
    transaction::TransactionInfo,
};

use crate::{
    rpc::gen_server::NetworkRpc, BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetTxns, TransactionsData,
};
use txpool_api::TxPoolSyncService;

pub struct NetworkRpcImpl<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<C>,
    txpool: TxPoolService,
    storage: Arc<dyn Store>,
}

impl<C> NetworkRpcImpl<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(
        chain_reader: ChainActorRef<C>,
        txpool: TxPoolService,
        storage: Arc<dyn Store>,
    ) -> Self {
        Self {
            chain_reader,
            txpool,
            storage,
        }
    }
}

impl<C> NetworkRpc for NetworkRpcImpl<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    fn get_txns(&self, _peer_id: PeerId, req: GetTxns) -> BoxFuture<Result<TransactionsData>> {
        let txpool = self.txpool.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let data = {
                match req.ids {
                    // get from txpool
                    None => txpool.get_pending_txns(None),
                    // get from storage
                    Some(ids) => {
                        let mut data = vec![];
                        for id in ids {
                            if let Ok(txn) = storage.get_transaction(id) {
                                if let Some(txn) = txn {
                                    if let Ok(stxn) = txn.as_signed_user_txn() {
                                        data.push(stxn.clone());
                                    }
                                }
                            }
                        }
                        data
                    }
                }
            };
            Ok(TransactionsData { txns: data })
        };
        Box::pin(fut)
    }

    fn get_txn_infos(
        &self,
        _peer_id: PeerId,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<Vec<TransactionInfo>>>> {
        let storage = self.storage.clone();
        let fut = async move {
            if let Ok(txn_infos) = storage.get_block_transaction_infos(block_id) {
                Ok(Some(txn_infos))
            } else {
                Ok(None)
            }
        };
        Box::pin(fut)
    }

    fn get_headers_by_number(
        &self,
        _peer_id: PeerId,
        request: GetBlockHeadersByNumber,
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut headers = Vec::new();
            let mut last_number = request.number;
            while headers.len() < request.max_size {
                if let Ok(header) = chain_reader
                    .clone()
                    .master_block_header_by_number(last_number)
                    .await
                {
                    headers.push(header);
                } else {
                    break;
                }

                if last_number == 0 {
                    break;
                }
                last_number = if last_number > request.step as u64 {
                    last_number - request.step as u64
                } else {
                    0
                }
            }
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_headers_with_peer(
        &self,
        _peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut headers = Vec::new();
            if let Ok(Some(header)) = chain_reader
                .clone()
                .get_header_by_hash(&request.block_id)
                .await
            {
                let mut last_number = header.number();
                while headers.len() < request.max_size {
                    let block_number = if request.reverse {
                        if last_number > request.step as u64 {
                            last_number - request.step as u64
                        } else {
                            0
                        }
                    } else {
                        last_number + request.step as u64
                    };
                    if let Ok(header) = chain_reader
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
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_info_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        let fut = async move {
            let mut infos = Vec::new();
            let chain_reader = self.chain_reader.clone();
            for hash in hashes {
                if let Ok(Some(block_info)) =
                    chain_reader.clone().get_block_info_by_hash(&hash).await
                {
                    infos.push(block_info);
                }
            }
            Ok(infos)
        };
        Box::pin(fut)
    }

    fn get_body_by_hash(
        &self,
        _peer_id: PeerId,
        hashs: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>> {
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut bodies = Vec::new();
            for hash in hashs {
                let (transactions, uncles) =
                    match chain_reader.clone().get_block_by_hash(hash).await {
                        Ok(block) => (
                            block.transactions().to_vec(),
                            if block.uncles().is_some() {
                                Some(block.uncles().expect("block.uncles() is none.").to_vec())
                            } else {
                                None
                            },
                        ),
                        _ => (Vec::new(), None),
                    };

                let body = BlockBody {
                    transactions,
                    hash,
                    uncles,
                };
                bodies.push(body);
            }
            Ok(bodies)
        };
        Box::pin(fut)
    }

    fn get_state_node_by_node_hash(
        &self,
        _peer_id: PeerId,
        state_node_key: HashValue,
    ) -> BoxFuture<Result<StateNode>> {
        let storage = self.storage.clone();
        let fut = async move {
            let mut keys = Vec::new();
            keys.push(state_node_key);
            let mut state_nodes = {
                let mut state_nodes = Vec::new();
                keys.iter()
                    .for_each(|node_key| match storage.get(node_key) {
                        Ok(node) => state_nodes.push((*node_key, node)),
                        Err(e) => error!("handle state_node {:?} err : {:?}", node_key, e),
                    });
                state_nodes
            };
            if let Some((_, state_node_res)) = state_nodes.pop() {
                if let Some(state_node) = state_node_res {
                    Ok(state_node)
                } else {
                    let err = format_err!("state_node is none");
                    debug!("{:?}", err);
                    Err(err)
                }
            } else {
                let err = format_err!("state_nodes is none");
                debug!("{:?}", err);
                Err(err)
            }
        };
        Box::pin(fut)
    }

    fn get_accumulator_node_by_node_hash(
        &self,
        _peer_id: PeerId,
        request: GetAccumulatorNodeByNodeHash,
    ) -> BoxFuture<Result<AccumulatorNode>> {
        let storage = self.storage.clone();
        let accumulator_node_key = request.node_hash;
        let accumulator_type = request.accumulator_storage_type;
        let fut = async move {
            let mut keys = Vec::new();
            keys.push(accumulator_node_key);
            let mut accumulator_nodes = {
                let mut accumulator_nodes = Vec::new();
                keys.iter().for_each(|node_key| {
                    match storage.get_node(accumulator_type.clone(), *node_key) {
                        Ok(node) => accumulator_nodes.push((*node_key, node)),
                        Err(e) => error!("handle accumulator_node {:?} err : {:?}", node_key, e),
                    }
                });
                accumulator_nodes
            };

            if let Some((_, accumulator_node_res)) = accumulator_nodes.pop() {
                if let Some(accumulator_node) = accumulator_node_res {
                    Ok(accumulator_node)
                } else {
                    let err = format_err!("accumulator_node {:?} is none.", accumulator_node_key);
                    debug!("{:?}", &err);
                    Err(err)
                }
            } else {
                let err = format_err!("accumulator_nodes is none.");
                debug!("{:?}", err);
                Err(err)
            }
        };
        Box::pin(fut)
    }
}
