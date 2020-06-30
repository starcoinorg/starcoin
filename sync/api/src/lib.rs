use actix::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::HashValue;
use starcoin_state_tree::StateNode;
use starcoin_types::block::BlockNumber;
use starcoin_types::peer_info::PeerId;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo},
    transaction::{SignedUserTransaction, TransactionInfo},
};
use std::cmp::Ordering;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct StartSyncTxnEvent;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct PeerNewBlock {
    peer_id: PeerId,
    new_block: Block,
}

impl PeerNewBlock {
    pub fn new(peer_id: PeerId, new_block: Block) -> Self {
        PeerNewBlock { peer_id, new_block }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block(&self) -> Block {
        self.new_block.clone()
    }
}

#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "Result<()>")]
pub enum SyncRpcRequest {
    GetBlockHeadersByNumber(GetBlockHeadersByNumber),
    GetBlockHeaders(GetBlockHeaders),
    GetBlockInfos(Vec<HashValue>),
    GetBlockBodies(Vec<HashValue>),
    GetStateNodeByNodeHash(HashValue),
    GetAccumulatorNodeByNodeHash(HashValue, AccumulatorStoreType),
    GetTxns(GetTxns),
    GetTxnInfos(HashValue),
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "Result<()>")]
pub enum SyncRpcResponse {
    BlockHeaders(Vec<BlockHeader>),
    BlockBodies(Vec<BlockBody>),
    BlockInfos(Vec<BlockInfo>),
    StateNode(StateNode),
    AccumulatorNode(AccumulatorNode),
    GetTxns(TransactionsData),
    GetTxnInfos(Option<Vec<TransactionInfo>>),
}

#[derive(Debug, Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub enum SyncNotify {
    ClosePeerMsg(PeerId),
    NewHeadBlock(PeerId, Box<Block>),
    NewPeerMsg(PeerId),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTxns {
    pub ids: Option<Vec<HashValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TransactionsData {
    pub txns: Vec<SignedUserTransaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeadersByNumber {
    pub number: BlockNumber,
    pub max_size: usize,
    pub step: usize,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetBlockHeaders {
    pub block_id: HashValue,
    pub max_size: usize,
    pub step: usize,
    pub reverse: bool,
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

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlockBody {
    pub hash: HashValue,
    pub transactions: Vec<SignedUserTransaction>,
}

impl Into<(HashValue, Vec<SignedUserTransaction>)> for BlockBody {
    fn into(self) -> (HashValue, Vec<SignedUserTransaction>) {
        (self.hash, self.transactions)
    }
}

impl PartialOrd for BlockBody {
    fn partial_cmp(&self, other: &BlockBody) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockBody {
    fn cmp(&self, other: &BlockBody) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}
