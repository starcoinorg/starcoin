use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use std::convert::TryFrom;

/// `AccumulatorInfo` is the object we store in the storage. It consists of the
/// info that we can create MerkleAccumulator.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct AccumulatorInfo {
    /// Accumulator root hash
    pub accumulator_root: HashValue,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: u64,
    /// The total number of nodes in this accumulator.
    pub num_nodes: u64,
}

impl AccumulatorInfo {
    pub fn new(
        accumulator_root: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: u64,
        num_nodes: u64,
    ) -> Self {
        AccumulatorInfo {
            accumulator_root,
            frozen_subtree_roots,
            num_leaves,
            num_nodes,
        }
    }

    pub fn get_accumulator_root(&self) -> &HashValue {
        &self.accumulator_root
    }

    pub fn get_frozen_subtree_roots(&self) -> &Vec<HashValue> {
        &self.frozen_subtree_roots
    }

    pub fn get_num_leaves(&self) -> u64 {
        self.num_leaves
    }

    pub fn get_num_nodes(&self) -> u64 {
        self.num_nodes
    }
}

impl Default for AccumulatorInfo {
    fn default() -> Self {
        AccumulatorInfo {
            accumulator_root: HashValue::default(),
            frozen_subtree_roots: Vec::new(),
            num_leaves: 0,
            num_nodes: 0,
        }
    }
}

impl TryFrom<MerkleAccumulator> for AccumulatorInfo {
    type Error = Error;

    fn try_from(block_accumulator: MerkleAccumulator) -> Result<AccumulatorInfo> {
        Ok(AccumulatorInfo::new(
            block_accumulator.root_hash(),
            block_accumulator.get_frozen_subtree_roots()?,
            block_accumulator.num_leaves(),
            block_accumulator.num_leaves(),
        ))
    }
}
