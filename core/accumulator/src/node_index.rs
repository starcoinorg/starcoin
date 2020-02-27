// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::LeafCount;
use mirai_annotations::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeIndex(u64);

impl NodeIndex {
    pub fn new(index: u64) -> Self {
        NodeIndex(index)
    }

    pub fn to_inorder_index(self) -> u64 {
        self.0
    }
    /// What level is this node in the tree, 0 if the node is a leaf,
    /// 1 if the level is one above a leaf, etc.
    pub fn level(self) -> u32 {
        (!self.0).trailing_zeros()
    }
    /// pos count start from 0 on each level
    pub fn from_level_and_pos(level: u32, pos: u64) -> Self {
        precondition!(level < 63);
        assume!(1u64 << level > 0); // bitwise and integer operations don't mix.
        let level_one_bits = (1u64 << level) - 1;
        let shifted_pos = pos << (level + 1);
        NodeIndex(shifted_pos | level_one_bits)
    }
    pub fn from_leaf_index(leaf_index: u64) -> Self {
        Self::from_level_and_pos(0, leaf_index)
    }

    pub fn root_from_leaf_index(leaf_index: u64) -> Self {
        let leaf = Self::from_leaf_index(leaf_index);
        Self(smear_ones_for_u64(leaf.0) >> 1)
    }

    pub fn root_from_leaf_count(leaf_count: LeafCount) -> Self {
        assert!(leaf_count > 0);
        Self::root_from_leaf_index((leaf_count - 1) as u64)
    }

    /// Creates an `AncestorSiblingIterator` using this node_index.
    pub fn iter_ancestor_sibling(self) -> AncestorSiblingIterator {
        AncestorSiblingIterator { nodeIndex: self }
    }

    /// Given a node, find its left most child in its subtree
    /// Left most child is a node, could be itself, at level 0
    pub fn left_most_child(self) -> Self {
        // Turn off its right most x bits. while x=level of node
        let level = self.level();
        Self(turn_off_right_most_n_bits(self.0, level))
    }

    pub fn is_placeholder(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        if self.0 <= leaf.0 {
            return false;
        }
        if self.left_most_child().0 <= leaf.0 {
            return false;
        }
        true
    }

    /// What is the parent of this node?
    pub fn parent(self) -> Self {
        assume!(self.0 < u64::max_value() - 1); // invariant
        Self(
            (self.0 | isolate_rightmost_zero_bit(self.0))
                & !(isolate_rightmost_zero_bit(self.0) << 1),
        )
    }

    /// This method takes in a node position and return its sibling position
    ///
    /// The observation is that, after stripping out the right-most common bits,
    /// two sibling nodes flip the the next right-most bits with each other.
    /// To find out the right-most common bits, first remove all the right-most ones
    /// because they are corresponding to level's indicator. Then remove next zero right after.
    pub fn sibling(self) -> Self {
        assume!(self.0 < u64::max_value() - 1); // invariant
        Self(self.0 ^ (isolate_rightmost_zero_bit(self.0) << 1))
    }
}

/// Smearing all the bits starting from MSB with ones
fn smear_ones_for_u64(v: u64) -> u64 {
    let mut n = v;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n |= n >> 32;
    n
}

/// Finds the rightmost 0-bit, turns off all bits, and sets this bit to 1 in
fn isolate_rightmost_zero_bit(v: u64) -> u64 {
    !v & (v + 1)
}

/// Turn off n right most bits
fn turn_off_right_most_n_bits(v: u64, n: u32) -> u64 {
    precondition!(n < 64);
    (v >> n) << n
}

#[derive(Debug)]
pub struct AncestorSiblingIterator {
    nodeIndex: NodeIndex,
}

impl Iterator for AncestorSiblingIterator {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        let current_sibling_index = self.nodeIndex.sibling();
        self.nodeIndex = self.nodeIndex.parent();
        Some(current_sibling_index)
    }
}
