use crate::rpc_impl::NetworkRpcImpl;
use accumulator::node::AccumulatorStoreType;
use actix::{Addr, Message};

use anyhow::Result;
use chain::ChainActorRef;
use crypto::HashValue;
use futures::channel::mpsc;
use network_api::messages::RawRpcRequestMessage;
use network_rpc_core::server::NetworkRpcServer;
use rpc::gen_server::NetworkRpc;
use serde::{Deserialize, Serialize};
use state_api::StateWithProof;
use state_service::ChainStateServiceImpl;
use std::sync::Arc;
use storage::Store;
use traits::Consensus;
use txpool::TxPoolService;
use types::access_path::AccessPath;
use types::block::{BlockHeader, BlockNumber};
use types::transaction::SignedUserTransaction;

mod rpc;
mod rpc_impl;
#[cfg(test)]
mod tests;

pub use rpc::gen_client;

pub fn start_network_rpc_server<C>(
    rpc_rx: mpsc::UnboundedReceiver<RawRpcRequestMessage>,
    chain: ChainActorRef<C>,
    storage: Arc<dyn Store>,
    txpool: TxPoolService,
) -> Result<Addr<NetworkRpcServer>>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    let state_node_store = storage.clone().into_super_arc();
    let state_service = ChainStateServiceImpl::new(state_node_store, None);
    let rpc_impl = NetworkRpcImpl::new(chain, txpool, state_service, storage);
    NetworkRpcServer::start(rpc_rx, rpc_impl.to_delegate())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionsData {
    pub txns: Vec<SignedUserTransaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeaders {
    pub block_id: HashValue,
    pub max_size: usize,
    pub step: usize,
    pub reverse: bool,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlockBody {
    pub hash: HashValue,
    pub transactions: Vec<SignedUserTransaction>,
    pub uncles: Option<Vec<BlockHeader>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: usize,
    pub step: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GetAccumulatorNodeByNodeHash {
    pub node_hash: HashValue,
    pub accumulator_storage_type: AccumulatorStoreType,
}

impl GetBlockHeadersByNumber {
    pub fn new(number: BlockNumber, step: usize, max_size: usize) -> Self {
        GetBlockHeadersByNumber {
            number,
            max_size,
            step,
        }
    }
}

impl GetBlockHeaders {
    pub fn new(block_id: HashValue, step: usize, reverse: bool, max_size: usize) -> Self {
        GetBlockHeaders {
            block_id,
            max_size,
            step,
            reverse,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTxns {
    pub ids: Option<Vec<HashValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetStateWithProof {
    pub state_root: HashValue,
    pub access_path: AccessPath,
}

impl Message for GetStateWithProof {
    type Result = Result<StateWithProof>;
}

pub(crate) const DELAY_TIME: u64 = 15;
