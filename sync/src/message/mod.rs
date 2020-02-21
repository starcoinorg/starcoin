use crate::download::DownloadActor;
use crate::process::ProcessActor;
use actix::prelude::*;
use crypto::HashValue;
use std::cmp::Ordering;
use types::{block::BlockHeader, peer_info::PeerInfo, transaction::SignedUserTransaction};

#[derive(Message)]
#[rtype(result = "()")]
pub enum SyncMessage {
    DownloadMessage(DownloadMessage),
    ProcessMessage(ProcessMessage),
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum DownloadMessage {
    LatestStateMsg(Option<Addr<ProcessActor>>, PeerInfo, LatestStateMsg),
    BatchHashByNumberMsg(Option<Addr<ProcessActor>>, PeerInfo, BatchHashByNumberMsg),
    BatchHeaderMsg(Option<Addr<ProcessActor>>, PeerInfo, BatchHeaderMsg),
    BatchBodyMsg(Option<Addr<ProcessActor>>, BatchBodyMsg),
    BatchHeaderAndBodyMsg(BatchHeaderMsg, BatchBodyMsg), // just fo test
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum ProcessMessage {
    NewPeerMsg(Option<Addr<DownloadActor>>, PeerInfo),
    GetHashByNumberMsg(Option<Addr<DownloadActor>>, GetHashByNumberMsg),
    GetDataByHashMsg(Option<Addr<DownloadActor>>, GetDataByHashMsg),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct LatestStateMsg {
    pub hash_header: HashWithBlockHeader,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
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

#[derive(Debug)]
pub struct GetHashByNumberMsg {
    pub numbers: Vec<u64>,
}

#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
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

#[derive(Debug)]
pub struct BatchHashByNumberMsg {
    pub hashs: Vec<HashWithNumber>,
}

struct StateNodeHashMsg {
    hash: HashValue,
}

struct BatchStateNodeDataMsg {
    //nodes:
}

#[derive(Clone, Debug)]
pub enum DataType {
    HEADER,
    BODY,
}

#[derive(Clone, Debug)]
pub struct GetDataByHashMsg {
    pub hashs: Vec<HashValue>,
    pub data_type: DataType,
}

#[derive(Clone, Debug)]
pub struct BatchHeaderMsg {
    pub headers: Vec<HashWithBlockHeader>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
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

#[derive(Debug)]
pub struct BatchBodyMsg {
    pub bodies: Vec<BlockBody>,
}
