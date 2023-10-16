use crate::block::BlockHeader;
use crate::blockhash::{BlockLevel, ORIGIN};
use crate::U256;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use std::sync::Arc;

pub trait ConsensusHeader {
    fn parents_hash(&self) -> &[Hash];
    fn difficulty(&self) -> U256;
    fn hash(&self) -> Hash;
    fn timestamp(&self) -> u64;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Header {
    block_header: BlockHeader,
    parents_hash: Vec<Hash>,
}

impl Header {
    pub fn new(block_header: BlockHeader, parents_hash: Vec<Hash>) -> Self {
        Self {
            block_header,
            parents_hash,
        }
    }

    pub fn genesis_hash(&self) -> Hash {
        Hash::new(ORIGIN)
    }
}

impl ConsensusHeader for Header {
    fn parents_hash(&self) -> &[Hash] {
        &self.parents_hash
    }
    fn difficulty(&self) -> U256 {
        self.block_header.difficulty()
    }
    fn hash(&self) -> Hash {
        self.block_header.id()
    }

    fn timestamp(&self) -> u64 {
        self.block_header.timestamp()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HeaderWithBlockLevel {
    pub header: Arc<Header>,
    pub block_level: BlockLevel,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CompactHeaderData {
    pub timestamp: u64,
    pub difficulty: U256,
}
