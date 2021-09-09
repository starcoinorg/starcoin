// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
/// `AccumulatorInfo` is the object we store in the storage. It consists of the
/// info that we can create MerkleAccumulator.
#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
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
            accumulator_root: *ACCUMULATOR_PLACEHOLDER_HASH,
            frozen_subtree_roots: Vec::new(),
            num_leaves: 0,
            num_nodes: 0,
        }
    }
}

impl Sample for AccumulatorInfo {
    fn sample() -> Self {
        Self::default()
    }
}
