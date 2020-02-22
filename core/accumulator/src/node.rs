// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use crypto::hash::{create_literal_hash, CryptoHash, HashValue};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::sync::Arc;

/// Placeholder hash of `Accumulator`.
pub static ACCUMULATOR_PLACEHOLDER_HASH: Lazy<HashValue> =
    Lazy::new(|| create_literal_hash("ACCUMULATOR_PLACEHOLDER_HASH"));

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
    pub fn new_internal(left: HashValue, right: HashValue) -> Self {
        AccumulatorNode::Internal(InternalNode::new(left, right))
    }

    pub fn new_leaf(value: HashValue) -> Self {
        AccumulatorNode::Leaf(LeafNode::new(value))
    }

    pub fn new_empty() -> Self {
        AccumulatorNode::Empty
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InternalNode {
    /// The hash of this internal node which is the root hash of the subtree.
    #[serde(skip)]
    hash: Cell<Option<HashValue>>,
    left: HashValue,
    right: HashValue,
}

impl InternalNode {
    fn new(left: HashValue, right: HashValue) -> Self {
        InternalNode {
            hash: Cell::new(None),
            left,
            right,
        }
    }
}

// impl CryptoHash for InternalNode {
//     fn crypto_hash(&self) -> HashValue {
//         match self.hash.get() {
//             Some(hash) => hash,
//             None => {
//                 let mut bytes = self.left.to_vec();
//                 bytes.extend(self.right.to_vec());
//                 let hash = HashValue::from_sha3_256(bytes.as_slice());
//                 self.hash.set(Some(hash))
//             }
//         }
//     }
// }

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LeafNode(HashValue);

impl LeafNode {
    pub fn new(value: HashValue) -> Self {
        LeafNode(value)
    }

    pub fn value(&self) -> HashValue {
        self.0
    }
}

// impl CryptoHash for LeafNode {
//     fn crypto_hash(&self) -> HashValue {
//         self.0
//     }
// }
