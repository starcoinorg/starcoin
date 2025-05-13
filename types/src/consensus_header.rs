use crate::block::{BlockHeader, ParentsHash};
use crate::blockhash::BlockLevel;
use crate::U256;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use std::sync::Arc;

pub trait ConsensusHeader {
    fn parents(&self) -> ParentsHash;
    fn difficulty(&self) -> U256;
    fn hash(&self) -> Hash;
    fn timestamp(&self) -> u64;
}

impl ConsensusHeader for BlockHeader {
    fn parents(&self) -> ParentsHash {
        self.parents_hash()
    }
    fn difficulty(&self) -> U256 {
        self.difficulty()
    }
    fn hash(&self) -> Hash {
        self.id()
    }

    fn timestamp(&self) -> u64 {
        self.timestamp()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeaderWithBlockLevel {
    pub header: Arc<BlockHeader>,
    pub block_level: BlockLevel,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CompactHeaderData {
    pub timestamp: u64,
    pub difficulty: U256,
}
