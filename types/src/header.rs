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
//TODO: Remove it and it's store
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DagHeader {
    block_header: BlockHeader,
    parents_hash: Vec<Hash>,
}

impl DagHeader {
    pub fn new(block_header: BlockHeader) -> Self {
        let parents_hash = block_header
            .parents_hash()
            .expect("parents_hash should exists for a dag block");
        Self {
            block_header,
            parents_hash,
        }
    }
    pub fn new_genesis(genesis_header: BlockHeader) -> DagHeader {
        Self {
            block_header: genesis_header,
            parents_hash: vec![Hash::new(ORIGIN)],
        }
    }
}

impl Into<BlockHeader> for DagHeader {
    fn into(self) -> BlockHeader {
        self.block_header
    }
}

impl ConsensusHeader for DagHeader {
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
    pub header: Arc<DagHeader>,
    pub block_level: BlockLevel,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CompactHeaderData {
    pub timestamp: u64,
    pub difficulty: U256,
}
