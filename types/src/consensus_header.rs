use crate::block::BlockHeader;
use crate::blockhash::{BlockLevel, ORIGIN};
use crate::U256;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{HashValue as Hash, HashValue};
use std::sync::Arc;

pub trait ConsensusHeader {
    fn parents(&self) -> Vec<HashValue>;
    fn difficulty(&self) -> U256;
    fn hash(&self) -> Hash;
    fn timestamp(&self) -> u64;
}

impl ConsensusHeader for BlockHeader {
    fn parents(&self) -> Vec<HashValue> {
        self.parents_hash()
            .expect("parents in block dag should exists")
            .clone()
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
