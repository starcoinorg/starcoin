// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{blob::Blob, mock_tree_store::MockTreeStore, node_type::Node, HashValueKey, NodeKey};
use starcoin_crypto::HashValue;

fn random_leaf_with_key() -> (Node<HashValueKey>, NodeKey) {
    let hash_value = HashValue::random();
    let node = Node::new_leaf(hash_value.into(), Blob::from(HashValue::random().to_vec()));
    let node_key = node.hash();
    (node, node_key)
}

#[test]
fn test_get_node() {
    let db = MockTreeStore::default();
    let cache = TreeCache::new(&db, None);

    let (node, node_key) = random_leaf_with_key();
    db.put_node(node_key, node.clone()).unwrap();

    assert_eq!(cache.get_node(&node_key).unwrap(), node);
}

#[test]
fn test_root_node() {
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(&db, None);
    assert_eq!(*cache.get_root_node_key(), *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let (node, node_key) = random_leaf_with_key();
    db.put_node(node_key, node).unwrap();
    cache.set_root_node_key(node_key);

    assert_eq!(*cache.get_root_node_key(), node_key);
}

#[test]
fn test_freeze_with_delete() {
    let db = MockTreeStore::default();
    let mut cache = TreeCache::new(&db, None);

    assert_eq!(*cache.get_root_node_key(), *SPARSE_MERKLE_PLACEHOLDER_HASH);

    let (node1, node1_key) = random_leaf_with_key();
    cache.put_node(node1_key, node1.clone()).unwrap();
    let (node2, node2_key) = random_leaf_with_key();
    cache.put_node(node2_key, node2.clone()).unwrap();
    assert_eq!(cache.get_node(&node1_key).unwrap(), node1);
    assert_eq!(cache.get_node(&node2_key).unwrap(), node2);
    cache.freeze();
    assert_eq!(cache.get_node(&node1_key).unwrap(), node1);
    assert_eq!(cache.get_node(&node2_key).unwrap(), node2);

    cache.delete_node(&node1_key, true /* is_leaf */);
    cache.freeze();
    let (_, update_batch) = cache.into();
    assert_eq!(update_batch.node_batch.len(), 3);
    assert_eq!(update_batch.stale_node_index_batch.len(), 1);
}
