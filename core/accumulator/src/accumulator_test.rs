// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    node::ACCUMULATOR_PLACEHOLDER_HASH, node_index::NodeIndex, Accumulator, AccumulatorNode,
    LeafCount, MerkleAccumulator, MockAccumulatorStore,
};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::{collections::HashMap, sync::Arc};

#[test]
fn test_accumulator_append() {
    // expected_root_hashes[i] is the root hash of an accumulator that has the first i leaves.
    let expected_root_hashes: Vec<HashValue> = (0..100)
        .map(|x| {
            let leaves = create_leaves(0..x);
            compute_root_hash_naive(&leaves)
        })
        .collect();

    let leaves = create_leaves(0..100);
    let mock_store = MockAccumulatorStore::new();
    let accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();
    // Append the leaves one at a time and check the root hashes match.
    for (i, (leaf, expected_root_hash)) in
        itertools::zip_eq(leaves.into_iter(), expected_root_hashes.into_iter()).enumerate()
    {
        assert_eq!(accumulator.root_hash(), expected_root_hash);
        assert_eq!(accumulator.num_leaves(), i as LeafCount);
        accumulator.append(&[leaf]);
    }
}

#[test]
fn test_error_on_bad_parameters() {
    let mock_store = MockAccumulatorStore::new();
    let accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();
    assert!(accumulator.get_proof(10).is_err());
}

#[test]
fn test_one_leaf() {
    let hash = HashValue::random();
    let mock_store = MockAccumulatorStore::new();
    let accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();
    let (root_hash, _) = accumulator.append(&[hash]).unwrap();
    assert_eq!(hash, root_hash);
    proof_verify(&accumulator, root_hash, &[hash], 0);
    let new_hash = HashValue::random();
    let (new_root_hash, _) = accumulator.append(&[new_hash]).unwrap();
    proof_verify(&accumulator, new_root_hash, &[new_hash], 1);
    let vec = vec![hash, new_hash];
    proof_verify(&accumulator, new_root_hash, &vec, 0);
}

#[test]
fn test_multiple_leaves() {
    let mut batch1 = create_leaves(0..8);
    let mock_store = MockAccumulatorStore::new();
    let mut accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();
    let (root_hash1, _) = accumulator.append(&batch1).unwrap();
    proof_verify(&accumulator, root_hash1, &batch1, 0);
    let batch2 = create_leaves(0..4);
    let (root_hash2, _) = accumulator.append(&batch2).unwrap();
    batch1.extend_from_slice(&batch2);
    proof_verify(&accumulator, root_hash2, &batch1, 0);
}

#[test]
fn test_multiple_tree() {
    let batch1 = create_leaves(0..8);
    let mock_store = MockAccumulatorStore::new();
    let arc_store = Arc::new(mock_store);
    let accumulator = MerkleAccumulator::new(vec![], 0, 0, arc_store.clone()).unwrap();
    let (root_hash1, _) = accumulator.append(&batch1).unwrap();
    proof_verify(&accumulator, root_hash1, &batch1, 0);
    let frozen_hash = accumulator.get_frozen_subtree_roots().unwrap();
    let accumulator2 =
        MerkleAccumulator::new(frozen_hash.clone(), 8, 15, arc_store.clone()).unwrap();
    let root_hash2 = accumulator2.root_hash();
    assert_eq!(root_hash1, root_hash2);
    proof_verify(&accumulator2, root_hash2, &batch1, 0);
}

#[ignore]
#[test]
fn test_update_leaf() {
    // construct a accumulator
    let leaves = create_leaves(0..8);
    let mock_store = MockAccumulatorStore::new();
    let accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();
    let (root_hash, _) = accumulator.append(&leaves).unwrap();
    proof_verify(&accumulator, root_hash, &leaves, 0);
    // update index from 6
    let new_leaves = create_leaves(0..4);
    let (new_root_hash, first_idx) = accumulator.update(6, &new_leaves).unwrap();
    proof_verify(&accumulator, new_root_hash, &leaves, first_idx);
}

// FIXME: proptest is not implemented for HashValue.
// Delete it or make HashValue right.

// //batch test
// proptest! {
//     #![proptest_config(ProptestConfig::with_cases(1))]

//     #[test]
//     fn test_proof(
//         batch1 in vec(any::<HashValue>(), 1..10),
//         batch2 in vec(any::<HashValue>(), 1..10),
//     ) {
//         let total_leaves = batch1.len() + batch2.len();
//         let mock_store = MockAccumulatorStore::new();
//         let mut accumulator = MerkleAccumulator::new(vec![], 0, 0, Arc::new(mock_store)).unwrap();

//         // insert all leaves in two batches
//         let (root_hash1, _) = accumulator.append(&batch1).unwrap();
//         proof_verify(&accumulator, root_hash1, &batch1, 0);

//         let (root_hash2, _) = accumulator.append(&batch2).unwrap();
//         // verify proofs for all leaves towards current root

//         proof_verify(&accumulator, root_hash2, &batch2, batch1.len() as u64);
//         let mut total_vec = batch1.clone();
//         total_vec.extend_from_slice(&batch2);
//         proof_verify(&accumulator, root_hash2, &total_vec, 0);
//     }
// }

fn proof_verify(
    accumulator: &MerkleAccumulator,
    root_hash: HashValue,
    leaves: &[HashValue],
    first_leaf_idx: u64,
) {
    leaves.iter().enumerate().for_each(|(i, hash)| {
        let leaf_index = first_leaf_idx + i as u64;
        let proof = accumulator.get_proof(leaf_index).unwrap().unwrap();
        proof.verify(root_hash, *hash, leaf_index).unwrap();
    });
}

// Helper function to create a list of leaves.
fn create_leaves(nums: std::ops::Range<usize>) -> Vec<HashValue> {
    nums.map(|x| x.to_be_bytes().as_ref().crypto_hash())
        .collect()
}

// Computes the root hash of an accumulator with given leaves.
fn compute_root_hash_naive(leaves: &[HashValue]) -> HashValue {
    let position_to_hash = compute_hashes_for_all_positions(leaves);
    if position_to_hash.is_empty() {
        return *ACCUMULATOR_PLACEHOLDER_HASH;
    }

    let rightmost_leaf_index = leaves.len() as u64 - 1;
    *position_to_hash
        .get(&NodeIndex::root_from_leaf_index(rightmost_leaf_index))
        .expect("Root position should exist in the map.")
}

/// Given a list of leaves, constructs the smallest accumulator that has all the leaves and
/// computes the hash of every node in the tree.
fn compute_hashes_for_all_positions(leaves: &[HashValue]) -> HashMap<NodeIndex, HashValue> {
    if leaves.is_empty() {
        return HashMap::new();
    }

    let mut current_leaves = leaves.to_vec();
    current_leaves.resize(
        leaves.len().next_power_of_two(),
        *ACCUMULATOR_PLACEHOLDER_HASH,
    );
    let mut position_to_hash = HashMap::new();
    let mut current_level = 0;

    while current_leaves.len() > 1 {
        assert!(current_leaves.len().is_power_of_two());

        let mut parent_leaves = vec![];
        for (index, _hash) in current_leaves.iter().enumerate().step_by(2) {
            let left_hash = current_leaves[index];
            let right_hash = current_leaves[index + 1];
            let left_pos = NodeIndex::from_level_and_pos(current_level, index as u64);
            let right_pos = NodeIndex::from_level_and_pos(current_level, index as u64 + 1);
            let parent_index = left_pos.parent();
            let parent_hash = compute_parent_hash(parent_index, left_hash, right_hash);
            parent_leaves.push(parent_hash);

            assert_eq!(position_to_hash.insert(left_pos, left_hash), None);
            assert_eq!(position_to_hash.insert(right_pos, right_hash), None);
        }

        assert_eq!(current_leaves.len(), parent_leaves.len() << 1);
        current_leaves = parent_leaves;
        current_level += 1;
    }

    assert_eq!(
        position_to_hash.insert(
            NodeIndex::from_level_and_pos(current_level, 0),
            current_leaves[0],
        ),
        None,
    );
    position_to_hash
}

fn compute_parent_hash(
    node_index: NodeIndex,
    left_hash: HashValue,
    right_hash: HashValue,
) -> HashValue {
    if left_hash == *ACCUMULATOR_PLACEHOLDER_HASH && right_hash == *ACCUMULATOR_PLACEHOLDER_HASH {
        *ACCUMULATOR_PLACEHOLDER_HASH
    } else {
        AccumulatorNode::new_internal(node_index, left_hash, right_hash).hash()
    }
}
