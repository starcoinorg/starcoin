use actix::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
    transaction::SignedUserTransaction,
};
use std::cmp::Ordering;

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum SyncMessage {
    DownloadMessage(DownloadMessage),
    ProcessMessage(ProcessMessage),
}

#[derive(Clone)]
pub enum DownloadMessage {
    ClosePeerMsg(PeerInfo),
    LatestStateMsg(PeerInfo, LatestStateMsg),
    BatchHashByNumberMsg(PeerInfo, BatchHashByNumberMsg),
    BatchHeaderMsg(PeerInfo, BatchHeaderMsg),
    BatchBodyMsg(BatchBodyMsg),
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg),
    NewHeadBlock(PeerInfo, Block),
    // just fo test
    MinedBlock(Block),
}

impl Message for DownloadMessage {
    type Result = Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessMessage {
    NewPeerMsg(PeerInfo),
    GetHashByNumberMsg(GetHashByNumberMsg),
    GetDataByHashMsg(GetDataByHashMsg),
}

impl CryptoHash for ProcessMessage {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(
            scs::to_bytes(self)
                .expect("Serialization should work.")
                .as_slice(),
        )
    }
}

impl Message for ProcessMessage {
    type Result = Result<()>;
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Hash, Clone, Debug)]
pub struct LatestStateMsg {
    pub header: BlockHeader,
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
