use accumulator::node::AccumulatorStoreType;
use crypto::HashValue;
use serde::{Deserialize, Serialize};
use types::block::{BlockHeader, BlockNumber};
use types::transaction::SignedUserTransaction;

mod rpc;
mod rpc_impl;
mod tests;

pub use rpc::{gen_client, gen_server};
pub use rpc_impl::NetworkRpcImpl;

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

pub(crate) const DELAY_TIME: u64 = 15;
