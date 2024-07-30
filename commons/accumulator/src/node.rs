// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::node_index::{NodeIndex, G_NODE_ERROR_INDEX};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher, ACCUMULATOR_PLACEHOLDER_HASH},
    HashValue,
};

//TODO move to a more suitable crate.
#[derive(
    Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash,
)]
pub enum AccumulatorStoreType {
    Transaction,
    Block,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub enum AccumulatorNode {
    Internal(InternalNode),
    Leaf(LeafNode),
    Empty,
}

impl AccumulatorNode {
    pub fn new_internal(index: NodeIndex, left: HashValue, right: HashValue) -> Self {
        Self::Internal(InternalNode::new(index, left, right))
    }

    pub fn new_leaf(index: NodeIndex, value: HashValue) -> Self {
        Self::Leaf(LeafNode::new(index, value))
    }

    pub fn hash(&self) -> HashValue {
        match self {
            Self::Internal(internal) => internal.hash(),
            Self::Leaf(leaf) => leaf.value(),
            Self::Empty => *ACCUMULATOR_PLACEHOLDER_HASH,
        }
    }

    pub fn index(&self) -> NodeIndex {
        match self {
            Self::Internal(internal) => internal.index(),
            Self::Leaf(leaf) => leaf.index(),
            Self::Empty => {
                // bail!("error for get index");
                *G_NODE_ERROR_INDEX
            }
        }
    }

    pub fn frozen(&mut self) -> Result<()> {
        let _node = match self {
            Self::Internal(internal) => internal.set_frozen(),
            _ => Ok(()),
        };
        Ok(())
    }

    pub fn is_frozen(&self) -> bool {
        match self {
            Self::Internal(internal) => internal.is_frozen,
            Self::Leaf(_) => true,
            Self::Empty => false,
        }
    }
}

/// An internal node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalNode {
    index: NodeIndex,
    left: HashValue,
    right: HashValue,
    is_frozen: bool,
}

impl InternalNode {
    pub fn new(index: NodeIndex, left: HashValue, right: HashValue) -> Self {
        Self {
            index,
            left,
            right,
            is_frozen: right != *ACCUMULATOR_PLACEHOLDER_HASH,
        }
    }

    pub fn hash(&self) -> HashValue {
        let mut bytes = self.left.to_vec();
        bytes.extend(self.right.to_vec());
        HashValue::sha3_256_of(bytes.as_slice())
    }

    pub fn index(&self) -> NodeIndex {
        self.index
    }
    pub fn left(&self) -> HashValue {
        self.left
    }
    pub fn right(&self) -> HashValue {
        self.right
    }

    pub fn set_frozen(&mut self) -> Result<()> {
        self.is_frozen = true;
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct LeafNode {
    index: NodeIndex,
    hash: HashValue,
}

impl LeafNode {
    pub fn new(index: NodeIndex, hash: HashValue) -> Self {
        Self { index, hash }
    }

    pub fn value(&self) -> HashValue {
        self.hash
    }

    pub fn index(&self) -> NodeIndex {
        self.index
    }
}
