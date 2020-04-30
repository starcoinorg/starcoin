// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::LeafCount;
use mirai_annotations::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct NodeIndex(u64);
pub const MAX_ACCUMULATOR_PROOF_DEPTH: usize = 63;
pub static NODE_ERROR_INDEX: Lazy<NodeIndex> = Lazy::new(|| NodeIndex::new(u64::max_value()));

#[derive(Debug, Eq, PartialEq)]
pub enum NodeDirection {
    Left,
    Right,
}

impl NodeIndex {
    pub fn new(index: u64) -> Self {
        NodeIndex(index)
    }

    pub fn is_freezable(self, leaf_index: u64) -> bool {
        let leaf = Self::from_leaf_index(leaf_index);
        let right_most_child = self.right_most_child();
        right_most_child.0 <= leaf.0
    }

    fn is_leaf(self) -> bool {
        self.0 & 1 == 0
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
        let level_one_bits = (1u64 << level as u64) - 1;
        let shifted_pos = pos << (level + 1) as u64;
        NodeIndex(shifted_pos | level_one_bits)
    }
    pub fn from_leaf_index(leaf_index: u64) -> Self {
        Self::from_level_and_pos(0, leaf_index)
    }

    pub fn from_inorder_index(index: u64) -> Self {
        NodeIndex(index)
    }

    // Given a leaf index, calculate the position of a minimum root which contains this leaf
    /// This method calculates the index of the smallest root which contains this leaf.
    /// Observe that, the root position is composed by a "height" number of ones
    ///
    /// For example
    /// ```text
    ///     0010010(node)
    ///     0011111(smearing)
    ///     -------
    ///     0001111(root)
    /// ```
    pub fn root_from_leaf_index(leaf_index: u64) -> Self {
        let leaf = Self::from_leaf_index(leaf_index);
        Self(smear_ones_for_u64(leaf.0) >> 1)
    }

    pub fn root_from_leaf_count(leaf_count: LeafCount) -> Self {
        assert!(leaf_count > 0);
        Self::root_from_leaf_index((leaf_count - 1) as u64)
    }

    pub fn root_level_from_leaf_count(leaf_count: LeafCount) -> u32 {
        assert!(leaf_count > 0);
        let index = (leaf_count - 1) as u64;
        MAX_ACCUMULATOR_PROOF_DEPTH as u32 + 1 - index.leading_zeros()
    }

    /// Get leaves count end from index
    pub fn leaves_count_end_from_index(leaf_index: u64) -> u64 {
        let mut count = 0u64;
        for index in 0..leaf_index {
            let leaf = Self::new(index);
            if leaf.is_leaf() {
                count += 1;
            }
        }
        count
    }

    /// Creates an `AncestorSiblingIterator` using this node_index.
    pub fn iter_ancestor_sibling(self) -> AncestorSiblingIterator {
        AncestorSiblingIterator { node_index: self }
    }

    /// Given a node, find its left most child in its subtree
    /// Left most child is a node, could be itself, at level 0
    pub fn left_most_child(self) -> Self {
        // Turn off its right most x bits. while x=level of node
        let level = self.level();
        Self(turn_off_right_most_n_bits(self.0, level))
    }

    /// Given a node, find its right most child in its subtree.
    /// Right most child is a Position, could be itself, at level 0
    pub fn right_most_child(self) -> Self {
        let level = self.level();
        Self(self.0 + (1_u64 << level as u64) - 1)
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

    /// What is the left node of this node? Will overflow if the node is a leaf
    pub fn left_child(self) -> Self {
        checked_precondition!(!self.is_leaf());
        Self::child(self, NodeDirection::Left)
    }

    /// What is the right node of this node? Will overflow if the node is a leaf
    pub fn right_child(self) -> Self {
        checked_precondition!(!self.is_leaf());
        Self::child(self, NodeDirection::Right)
    }

    fn child(self, dir: NodeDirection) -> Self {
        checked_precondition!(!self.is_leaf());
        assume!(self.0 < u64::max_value() - 1); // invariant

        let direction_bit = match dir {
            NodeDirection::Left => 0,
            NodeDirection::Right => isolate_rightmost_zero_bit(self.0),
        };
        Self((self.0 | direction_bit) & !(isolate_rightmost_zero_bit(self.0) >> 1))
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
    /// Whether this node_index is a left child of its parent.  The observation is that,
    /// after stripping out all right-most 1 bits, a left child will have a bit pattern
    /// of xxx00(11..), while a right child will be represented by xxx10(11..)
    pub fn is_left_child(self) -> bool {
        assume!(self.0 < u64::max_value() - 1); // invariant
        self.0 & (isolate_rightmost_zero_bit(self.0) << 1) == 0
    }

    pub fn is_right_child(self) -> bool {
        !self.is_left_child()
    }
}

/// Traverse leaves from left to right in groups that forms full subtrees, yielding root positions
/// of such subtrees.
/// Note that each 1-bit in num_leaves corresponds to a full subtree.
/// For example, in the below tree of 5=0b101 leaves, the two 1-bits corresponds to Fzn2 and L4
/// accordingly.
///
/// ```text
///            Non-fzn
///           /       \
///          /         \
///         /           \
///       Fzn2         Non-fzn
///      /   \           /   \
///     /     \         /     \
///    Fzn1    Fzn3  Non-fzn  [Placeholder]
///   /  \    /  \    /    \
///  L0  L1  L2  L3 L4   [Placeholder]
/// ```
pub struct FrozenSubTreeIterator {
    bitmap: u64,
    seen_leaves: u64,
    // invariant seen_leaves < u64::max_value() - bitmap
}

impl FrozenSubTreeIterator {
    pub fn new(num_leaves: LeafCount) -> Self {
        Self {
            bitmap: num_leaves,
            seen_leaves: 0,
        }
    }
}

impl Iterator for FrozenSubTreeIterator {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        assume!(self.seen_leaves < u64::max_value() - self.bitmap); // invariant

        if self.bitmap == 0 {
            return None;
        }

        // Find the remaining biggest full subtree.
        // The MSB of the bitmap represents it. For example for a tree of 0b1010=10 leaves, the
        // biggest and leftmost full subtree has 0b1000=8 leaves, which can be got by smearing all
        // bits after MSB with 1-bits (got 0b1111), right shift once (got 0b0111) and add 1 (got
        // 0b1000=8). At the same time, we also observe that the in-order numbering of a full
        // subtree root is (num_leaves - 1) greater than that of the leftmost leaf, and also
        // (num_leaves - 1) less than that of the rightmost leaf.
        let root_offset = smear_ones_for_u64(self.bitmap) >> 1;
        assume!(root_offset < self.bitmap); // relate bit logic to integer logic
        let num_leaves = root_offset + 1;
        let leftmost_leaf = NodeIndex::from_leaf_index(self.seen_leaves);
        let root = NodeIndex::from_inorder_index(leftmost_leaf.to_inorder_index() + root_offset);

        // Mark it consumed.
        self.bitmap &= !num_leaves;
        self.seen_leaves += num_leaves;

        Some(root)
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
    (v >> n as u64) << n as u64
}

#[derive(Debug)]
pub struct AncestorSiblingIterator {
    node_index: NodeIndex,
}

impl Iterator for AncestorSiblingIterator {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<NodeIndex> {
        let current_sibling_index = self.node_index.sibling();
        self.node_index = self.node_index.parent();
        Some(current_sibling_index)
    }
}
