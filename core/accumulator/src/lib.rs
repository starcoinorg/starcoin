// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use crypto::{hash::CryptoHash, HashValue};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;

mod node;

pub use node::AccumulatorNode;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AccumulatorProof {}

pub trait Accumulator {
    /// Append leaves and return new root
    fn append(&self, leaves: &[HashValue]) -> Result<HashValue>;
    /// Get leaf hash by leaf index.
    fn get_leaf(&self, leaf_index: u64) -> Result<Option<HashValue>>;

    fn get_proof(&self, leaf_index: u64) -> Result<Option<AccumulatorProof>>;

    fn root_hash(&self) -> HashValue;
}

pub trait AccumulatorNodeReader {
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>>;
}

pub trait AccumulatorNodeWriter {
    fn save_node(&self, node: AccumulatorNode) -> Result<()>;
}

pub trait AccumulatorNodeStore: AccumulatorNodeReader + AccumulatorNodeWriter {}

/// MerkleAccumulator is a accumulator algorithm implement and it is stateless.
pub struct MerkleAccumulator<R>
where
    R: AccumulatorNodeReader,
{
    reader: PhantomData<R>,
}

impl<R> MerkleAccumulator<R>
where
    R: AccumulatorNodeReader,
{
    pub fn append(
        &self,
        _reader: &R,
        root_hash: HashValue,
        _leaves: &[HashValue],
    ) -> Result<(HashValue, Vec<AccumulatorNode>)> {
        //TODO
        Ok((root_hash, vec![]))
    }

    pub fn get_proof(
        &self,
        _reader: &R,
        _root_hash: HashValue,
        _leaf_index: u64,
    ) -> Result<Option<AccumulatorProof>> {
        //TODO
        Ok(Some(AccumulatorProof {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator() {}
}
