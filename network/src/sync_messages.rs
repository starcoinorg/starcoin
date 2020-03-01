use actix::prelude::*;
use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
    transaction::SignedUserTransaction,
};

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum SyncMessage {
    DownloadMessage(DownloadMessage),
    ProcessMessage(ProcessMessage),
}

#[derive(Clone)]
pub enum DownloadMessage {
    LatestStateMsg(PeerInfo, LatestStateMsg),
    BatchHashByNumberMsg(PeerInfo, BatchHashByNumberMsg),
    BatchHeaderMsg(PeerInfo, BatchHeaderMsg),
    BatchBodyMsg(BatchBodyMsg),
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg),
    // just fo test
    NewBlock(Block),
}

impl Message for DownloadMessage {
    type Result = Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessMessage {
    NewPeerMsg(PeerInfo),
    GetHashByNumberMsg(AccountAddress, GetHashByNumberMsg),
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
    pub hash_header: HashWithBlockHeader,
}

#[derive(Eq, Serialize, Deserialize, PartialEq, Hash, Clone, Debug)]
pub struct HashWithBlockHeader {
    pub hash: HashValue,
    pub header: BlockHeader,
}

impl PartialOrd for HashWithBlockHeader {
    fn partial_cmp(&self, other: &HashWithBlockHeader) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HashWithBlockHeader {
    fn cmp(&self, other: &Self) -> Ordering {
        self.header.cmp(&other.header)
    }
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
    pub id: HashValue,
    pub hashs: Vec<HashWithNumber>,
}

struct StateNodeHashMsg {
    hash: HashValue,
}

struct BatchStateNodeDataMsg {
    //nodes:
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BatchHeaderMsg {
    pub headers: Vec<HashWithBlockHeader>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchBodyMsg {
    pub bodies: Vec<BlockBody>,
}
