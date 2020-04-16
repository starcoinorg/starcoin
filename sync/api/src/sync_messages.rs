use actix::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_state_tree::StateNode;
use starcoin_types::peer_info::PeerId;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo},
    transaction::SignedUserTransaction,
};
use std::cmp::Ordering;

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
    GetHashByNumberMsg(GetHashByNumberMsg),
    GetDataByHashMsg(GetDataByHashMsg),
    GetStateNodeByNodeHash(HashValue),
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "Result<()>")]
pub enum SyncRpcResponse {
    BatchHashByNumberMsg(BatchHashByNumberMsg),
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg, BatchBlockInfo),
    GetStateNodeByNodeHash(StateNode),
}

#[derive(Debug, Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub enum SyncNotify {
    ClosePeerMsg(PeerId),
    NewHeadBlock(PeerId, Block),
    NewPeerMsg(PeerId),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetHashByNumberMsg {
    pub numbers: Vec<u64>,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct HashWithNumber {
    pub hash: HashValue,
    pub number: u64,
}

impl Ord for HashWithNumber {
    fn cmp(&self, other: &HashWithNumber) -> Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Equal => {
                return self.hash.cmp(&other.hash);
            }
            ordering => return ordering,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct BatchHashByNumberMsg {
    pub hashs: Vec<HashWithNumber>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DataType {
    HEADER,
    BODY,
    INFO,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetDataByHashMsg {
    pub hashs: Vec<HashValue>,
    pub data_type: DataType,
}

#[derive(Clone, Eq, Serialize, Deserialize, PartialEq, Debug)]
pub struct BatchHeaderMsg {
    pub headers: Vec<BlockHeader>,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct BlockBody {
    pub hash: HashValue,
    pub transactions: Vec<SignedUserTransaction>,
}

impl PartialOrd for BlockBody {
    fn partial_cmp(&self, other: &BlockBody) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockBody {
    fn cmp(&self, other: &BlockBody) -> Ordering {
        return self.hash.cmp(&other.hash);
    }
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BatchBodyMsg {
    pub bodies: Vec<BlockBody>,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BatchBlockInfo {
    pub infos: Vec<BlockInfo>,
}
