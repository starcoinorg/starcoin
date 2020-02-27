// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use crypto::hash::{create_literal_hash, CryptoHash, HashValue};

use crate::node_index::NodeIndex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};

/// Placeholder hash of `Accumulator`.
pub static ACCUMULATOR_PLACEHOLDER_HASH: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("ACCUMULATOR_PLACEHOLDER_HASH"));

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccumulatorNode {
    Internal(InternalNode),
    Leaf(LeafNode),
    Empty,
}

//TODO how to custom
// impl CryptoHash for AccumulatorNode {
//     fn crypto_hash(&self) -> HashValue {
//         match self {
//             AccumulatorNode::Internal(n) => n.crypto_hash(),
//             AccumulatorNode::Leaf(n) => n.crypto_hash(),
//             AccumulatorNode::Empty => *ACCUMULATOR_PLACEHOLDER_HASH,
//         }
//     }
// }

impl AccumulatorNode {
    pub fn new_internal(index: NodeIndex, left: HashValue, right: HashValue) -> Self {
        AccumulatorNode::Internal(InternalNode::new(index, left, right))
    }

    pub fn new_leaf(index: NodeIndex, value: HashValue) -> Self {
        AccumulatorNode::Leaf(LeafNode::new(index, value))
    }

    pub fn new_empty() -> Self {
        AccumulatorNode::Empty
    }

    pub fn hash(&self) -> HashValue {
        match self {
            AccumulatorNode::Internal(internal) => internal.hash(),
            AccumulatorNode::Leaf(leaf) => leaf.value(),
            AccumulatorNode::Empty => *ACCUMULATOR_PLACEHOLDER_HASH,
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        if let AccumulatorNode::Empty = self {
            true
        } else {
            false
        }
    }
}

/// An internal node.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalNode {
    /// The hash of this internal node which is the root hash of the subtree.
    // #[serde(skip)]
    // hash: Cell<Option<HashValue>>,
    index: NodeIndex,
    left: HashValue,
    right: HashValue,
}

impl InternalNode {
    pub fn new(index: NodeIndex, left: HashValue, right: HashValue) -> Self {
        InternalNode {
            // hash: Cell::new(None),
            index,
            left,
            right,
        }
    }
    pub fn hash(&self) -> HashValue {
        // match self.hash.get() {
        //     Some(hash) => hash,
        //     None => {
        let mut bytes = self.left.to_vec();
        bytes.extend(self.right.to_vec());
        let hash = HashValue::from_sha3_256(bytes.as_slice());
        // self.hash.set(Some(hash));
        hash
        //     }
        // }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct LeafNode {
    index: NodeIndex,
    hash: HashValue,
}

impl LeafNode {
    pub fn new(index: NodeIndex, hash: HashValue) -> Self {
        LeafNode { index, hash }
    }

    pub fn value(&self) -> HashValue {
        self.hash
    }
}

// impl CryptoHash for LeafNode {
//     fn crypto_hash(&self) -> HashValue {
//         self.0
//     }
// }
