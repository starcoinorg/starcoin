// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::nibble::Nibble;
use mock_tree_store::MockTreeStore;

use crate::node_type::SparseMerkleInternalNode;
use proptest::{
    collection::{btree_map, hash_map, vec},
    prelude::*,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use starcoin_crypto::hash::{HashValue, *};
use std::collections::HashMap;
use std::ops::Bound;
use test_helper::{init_mock_db, plus_one};

fn update_nibble(original_key: &HashValue, n: usize, nibble: u8) -> HashValue {
    assert!(nibble < 16);
    let mut key = original_key.to_vec();
    key[n / 2] = if n % 2 == 0 {
        key[n / 2] & 0x0f | nibble << 4
    } else {
        key[n / 2] & 0xf0 | nibble
    };
    HashValue::from_slice(&key).unwrap()
}

#[test]
fn test_insert_to_empty_tree() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    // Tree is initially empty. Root is a null node. We'll insert a key-value pair which creates a
    // leaf node.
    let key = HashValue::random();
    let value = Blob::from(vec![1u8, 2u8, 3u8, 4u8]);

    let (_new_root_hash, batch) = tree
        .put_blob_set(None, vec![(key.into(), value.clone())])
        .unwrap();
    assert!(batch.stale_node_index_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();

    assert_eq!(tree.get(_new_root_hash, key).unwrap().unwrap(), value);
}

#[test]
fn test_delete_from_tree() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    // Tree is initially empty. Root is a null node. We'll insert a key-value pair which creates a
    // leaf node.
    let key = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value = Blob::from(vec![1u8, 2u8, 3u8, 4u8]);

    let (_new_root_hash, batch) = tree.put_blob_set(None, vec![(key.into(), value)]).unwrap();
    db.write_tree_update_batch(batch).unwrap();

    let (new_root, batch) = tree.delete(Some(_new_root_hash), key.into()).unwrap();
    assert_eq!(new_root, *SPARSE_MERKLE_PLACEHOLDER_HASH);
    assert_eq!(batch.num_stale_leaves, 1);
    assert_eq!(batch.stale_node_index_batch.len(), 1);
    assert_eq!(batch.num_new_leaves, 0);
    assert_eq!(batch.node_batch.len(), 0);

    let key2 = update_nibble(&key, 0, 15);
    let value2 = Blob::from(vec![3u8, 4u8]);

    let (_root1_hash, batch) = tree
        .put_blob_set(Some(_new_root_hash), vec![(key2.into(), value2)])
        .unwrap();
    assert_eq!(batch.stale_node_index_batch.len(), 0);
    db.write_tree_update_batch(batch).unwrap();

    let (new_root, batch) = tree.delete(Some(_root1_hash), key2.into()).unwrap();
    assert_eq!(new_root, _new_root_hash);
    assert_eq!(batch.num_stale_leaves, 1);
    assert_eq!(batch.stale_node_index_batch.len(), 2);
    assert_eq!(batch.num_new_leaves, 0);
    assert_eq!(batch.node_batch.len(), 0);
}

#[test]
fn test_insert_at_leaf_with_internal_created() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let key1 = HashValueKey(HashValue::new([0x00u8; HashValue::LENGTH]));
    let value1 = Blob::from(vec![1u8, 2u8]);

    let (_root0_hash, batch) = tree
        .put_blob_set(None, vec![(key1, value1.clone())])
        .unwrap();

    assert!(batch.stale_node_index_batch.is_empty());
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(
        tree.get(_root0_hash, key1.key_hash()).unwrap().unwrap(),
        value1
    );
    assert_eq!(db.num_nodes(), 1);
    // Insert at the previous leaf node. Should generate an internal node at the root.
    // Change the 1st nibble to 15.
    let key2 = HashValueKey(update_nibble(&key1.key_hash(), 0, 15));
    let value2 = Blob::from(vec![3u8, 4u8]);

    let (_root1_hash, batch) = tree
        .put_blob_set(Some(_root0_hash), vec![(key2, value2.clone())])
        .unwrap();
    assert_eq!(batch.stale_node_index_batch.len(), 0);
    db.write_tree_update_batch(batch).unwrap();

    assert_eq!(
        tree.get(_root1_hash, key1.key_hash()).unwrap().unwrap(),
        value1
    );
    assert!(tree.get(_root0_hash, key2.key_hash()).unwrap().is_none());
    assert_eq!(
        tree.get(_root1_hash, key2.key_hash()).unwrap().unwrap(),
        value2
    );

    // get # of nodes
    assert_eq!(db.num_nodes(), 3);

    let leaf1 = Node::new_leaf(key1, value1);
    let leaf2 = Node::new_leaf(key2, value2);
    let mut children = HashMap::new();
    children.insert(
        Nibble::from(0),
        Child::new(leaf1.hash(), true /* is_leaf */),
    );
    children.insert(
        Nibble::from(15),
        Child::new(leaf2.hash(), true /* is_leaf */),
    );
    let internal = Node::new_internal(children);
    assert_eq!(db.get_node(&leaf1.hash()).unwrap(), leaf1);
    assert_eq!(db.get_node(&leaf2.hash()).unwrap(), leaf2);
    assert_eq!(db.get_node(&internal.hash()).unwrap(), internal);
}

#[test]
fn test_insert_at_leaf_with_multiple_internals_created() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    // 1. Insert the first leaf into empty tree
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = Blob::from(vec![1u8, 2u8]);

    let (_root0_hash, batch) = tree
        .put_blob_set(None, vec![(key1.into(), value1.clone())])
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(_root0_hash, key1).unwrap().unwrap(), value1);

    // 2. Insert at the previous leaf node. Should generate a branch node at root.
    // Change the 2nd nibble to 1.
    let key2 = update_nibble(&key1, 1 /* nibble_index */, 1 /* nibble */);
    let value2 = Blob::from(vec![3u8, 4u8]);

    let (_root1_hash, batch) = tree
        .put_blob_set(Some(_root0_hash), vec![(key2.into(), value2.clone())])
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(_root0_hash, key1,).unwrap().unwrap(), value1);
    assert!(tree.get(_root0_hash, key2,).unwrap().is_none());
    assert_eq!(tree.get(_root1_hash, key2,).unwrap().unwrap(), value2);
    assert_eq!(tree.get(_root1_hash, key1,).unwrap().unwrap(), value1);

    assert_eq!(db.num_nodes(), 4);
    tree.print_tree(_root1_hash, key1).unwrap();

    let leaf1: Node<HashValueKey> = Node::new_leaf(key1.into(), value1);
    let leaf2: Node<HashValueKey> = Node::new_leaf(key2.into(), value2.clone());
    let internal = {
        let mut children = HashMap::new();
        children.insert(
            Nibble::from(0),
            Child::new(leaf1.hash(), true /* is_leaf */),
        );
        children.insert(Nibble::from(1), Child::new(leaf2.hash(), true));
        Node::new_internal(children)
    };

    let root_internal = {
        let mut children = HashMap::new();
        children.insert(
            Nibble::from(0),
            Child::new(internal.hash(), false /* is_leaf */),
        );
        Node::new_internal(children)
    };

    assert_eq!(db.get_node(&internal.hash()).unwrap(), internal);
    assert_eq!(db.get_node(&root_internal.hash()).unwrap(), root_internal,);

    // 3. Update leaf2 with new value
    let value2_update = Blob::from(vec![5u8, 6u8]);
    let (_root2_hash, batch) = tree
        .put_blob_set(
            Some(_root1_hash),
            vec![(key2.into(), value2_update.clone())],
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert!(tree.get(_root0_hash, key2,).unwrap().is_none());
    assert_eq!(tree.get(_root1_hash, key2,).unwrap().unwrap(), value2);
    assert_eq!(
        tree.get(_root2_hash, key2,).unwrap().unwrap(),
        value2_update
    );

    tree.print_tree(_root2_hash, key1).unwrap();
    // Get # of nodes.
    assert_eq!(db.num_nodes(), 7);

    // Purge retired nodes.
    db.purge_stale_nodes(_root0_hash).unwrap();
    db.purge_stale_nodes(_root1_hash).unwrap();
    assert_eq!(db.num_nodes(), 7);
    db.purge_stale_nodes(_root2_hash).unwrap();
    tree.print_tree(_root2_hash, key1).unwrap();
    assert_eq!(db.num_nodes(), 4);
}

#[test]
fn test_batch_insertion() {
    // ```text
    //                             internal(root)
    //                            /        \
    //                       internal       2        <- nibble 0
    //                      /   |   \
    //              internal    3    4               <- nibble 1
    //                 |
    //              internal                         <- nibble 2
    //              /      \
    //        internal      6                        <- nibble 3
    //           |
    //        internal                               <- nibble 4
    //        /      \
    //       1        5                              <- nibble 5
    //
    // Total: 12 nodes
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = Blob::from(vec![1u8]);

    let key2 = update_nibble(&key1, 0, 2);
    let value2 = Blob::from(vec![2u8]);
    let value2_update = Blob::from(vec![22u8]);

    let key3 = update_nibble(&key1, 1, 3);
    let value3 = Blob::from(vec![3u8]);

    let key4 = update_nibble(&key1, 1, 4);
    let value4 = Blob::from(vec![4u8]);

    let key5 = update_nibble(&key1, 5, 5);
    let value5 = Blob::from(vec![5u8]);

    let key6 = update_nibble(&key1, 3, 6);
    let value6 = Blob::from(vec![6u8]);

    let batches: Vec<Vec<(HashValueKey, Blob)>> = vec![
        vec![(key1.into(), value1)],
        vec![(key2.into(), value2)],
        vec![(key3.into(), value3)],
        vec![(key4.into(), value4)],
        vec![(key5.into(), value5)],
        vec![(key6.into(), value6)],
        vec![(key2.into(), value2_update)],
    ];
    let one_batch = batches.iter().flatten().cloned().collect::<Vec<_>>();

    let mut to_verify = one_batch.clone();
    // key2 was updated so we remove it.
    to_verify.remove(1);
    let verify_fn = |tree: &JellyfishMerkleTree<HashValueKey, MockTreeStore>, root: HashValue| {
        to_verify
            .iter()
            .for_each(|(k, v)| assert_eq!(tree.get(root, k.0).unwrap().unwrap(), *v))
    };

    // Insert as one batch.
    {
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);

        let (root, batch) = tree.put_blob_set(None, one_batch).unwrap();
        db.write_tree_update_batch(batch).unwrap();
        verify_fn(&tree, root);

        // get # of nodes
        assert_eq!(db.num_nodes(), 12);
        tree.print_tree(root, key1).unwrap();
    }

    // Insert in multiple batches.
    {
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);
        let mut batches2 = vec![];

        for (_idx, sub_vec) in batches.iter().enumerate() {
            for x in sub_vec {
                batches2.push(vec![(x.0, Some(x.1.clone()))]);
            }
        }
        let (mut roots, batch) = tree.puts(None, batches2).unwrap();
        db.write_tree_update_batch(batch).unwrap();
        let root_hash = roots.pop().unwrap();
        verify_fn(&tree, root_hash);

        // get # of nodes
        assert_eq!(db.num_nodes(), 23 /* 1 + 2 + 3 + 3 + 7 + 5 + 2 */);
        tree.print_tree(root_hash, key1).unwrap();

        // Purge retired nodes('p' means purged and 'a' means added).
        // The initial state of the tree at version 0
        // ```test
        //   1(root)
        // ```
        db.purge_stale_nodes(key1).unwrap();
        // ```text
        //   1 (p)           internal(a)
        //           ->     /        \
        //                 1(a)       2(a)
        // add 3, prune 1
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(key2).unwrap();
        // ```text
        //     internal(p)             internal(a)
        //    /        \              /        \
        //   1(p)       2   ->   internal(a)    2
        //                       /       \
        //                      1(a)      3(a)
        // add 4, prune 2
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(key3).unwrap();
        // ```text
        //         internal(p)                internal(a)
        //        /        \                 /        \
        //   internal(p)    2   ->     internal(a)     2
        //   /       \                /   |   \
        //  1         3              1    3    4(a)
        // add 3, prune 2
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(key4).unwrap();
        // ```text
        //            internal(p)                         internal(a)
        //           /        \                          /        \
        //     internal(p)     2                    internal(a)    2
        //    /   |   \                            /   |   \
        //   1(p) 3    4           ->      internal(a) 3    4
        //                                     |
        //                                 internal(a)
        //                                     |
        //                                 internal(a)
        //                                     |
        //                                 internal(a)
        //                                 /      \
        //                                1(a)     5(a)
        // add 8, prune 3
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(key5).unwrap();
        // ```text
        //                  internal(p)                             internal(a)
        //                 /        \                              /        \
        //            internal(p)    2                        internal(a)    2
        //           /   |   \                               /   |   \
        //   internal(p) 3    4                      internal(a) 3    4
        //       |                                      |
        //   internal(p)                 ->          internal(a)
        //       |                                   /      \
        //   internal                          internal      6(a)
        //       |                                |
        //   internal                          internal
        //   /      \                          /      \
        //  1        5                        1        5
        // add 5, prune 4
        // ```
        assert_eq!(db.num_nodes(), 23);
        db.purge_stale_nodes(key6).unwrap();
        // ```text
        //                         internal(p)                               internal(a)
        //                        /        \                                /        \
        //                   internal       2(p)                       internal       2(a)
        //                  /   |   \                                 /   |   \
        //          internal    3    4                        internal    3    4
        //             |                                         |
        //          internal                      ->          internal
        //          /      \                                  /      \
        //    internal      6                           internal      6
        //       |                                         |
        //    internal                                  internal
        //    /      \                                  /      \
        //   1        5                                1        5
        // add 2, prune 2
        // ```
        assert_eq!(db.num_nodes(), 23);
        verify_fn(&tree, root_hash);
    }
}

#[test]
fn test_non_existence() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);
    // ```text
    //                     internal(root)
    //                    /        \
    //                internal      2
    //                   |
    //                internal
    //                /      \
    //               1        3
    // Total: 7 nodes
    // ```
    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = Blob::from(vec![1u8]);

    let key2 = update_nibble(&key1, 0, 15);
    let value2 = Blob::from(vec![2u8]);

    let key3 = update_nibble(&key1, 2, 3);
    let value3 = Blob::from(vec![3u8]);

    let (root, batch) = tree
        .put_blob_set(
            None,
            vec![
                (key1.into(), value1.clone()),
                (key2.into(), value2.clone()),
                (key3.into(), value3.clone()),
            ],
        )
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(root, key1).unwrap().unwrap(), value1);
    assert_eq!(tree.get(root, key2).unwrap().unwrap(), value2);
    assert_eq!(tree.get(root, key3).unwrap().unwrap(), value3);
    // get # of nodes
    assert_eq!(db.num_nodes(), 6);

    // test non-existing nodes.
    // 1. Non-existing node at root node
    {
        let non_existing_key = update_nibble(&key1, 0, 1);
        let (value, proof) = tree.get_with_proof(root, non_existing_key).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
    // 2. Non-existing node at non-root internal node
    {
        let non_existing_key = update_nibble(&key1, 1, 15);
        let (value, proof) = tree.get_with_proof(root, non_existing_key).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
    // 3. Non-existing node at leaf node
    {
        let non_existing_key = update_nibble(&key1, 2, 4);
        let (value, proof) = tree.get_with_proof(root, non_existing_key).unwrap();
        assert_eq!(value, None);
        assert!(proof.verify(root, non_existing_key, None).is_ok());
    }
}

#[test]
fn test_non_existence_and_build_new_root_with_proof() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);
    // ```text
    //                     internal(root)
    //                    /        \
    //                internal      2
    //                   |
    //                internal
    //                /      \
    //               1        3
    // Total: 7 nodes
    // ```

    //test one key in the tree

    let key1 = HashValue::new([0x00u8; HashValue::LENGTH]);
    let value1 = Blob::from(vec![1u8]);

    let (root, batch) = tree
        .put_blob_set(None, vec![(key1.into(), value1.clone())])
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(root, key1).unwrap().unwrap(), value1);

    let key2 = update_nibble(&key1, 0, 15);
    let value2 = Blob::from(vec![2u8]);

    let root = test_nonexistent_key_value_update_impl(&tree, &db, root, (key2, value2));

    let key3 = update_nibble(&key1, 2, 3);
    let value3 = Blob::from(vec![3u8]);

    let root = test_nonexistent_key_value_update_impl(&tree, &db, root, (key3, value3));

    // test random key
    let key4 = HashValue::random();
    let value4 = Blob::from(vec![4u8]);

    let _root = test_nonexistent_key_value_update_impl(&tree, &db, root, (key4, value4));
}

#[test]
fn test_non_existence_and_build_new_root_with_proof_many() {
    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let key1 = HashValue::random();
    let value1 = Blob::from(vec![1u8]);

    let (mut root, batch) = tree
        .put_blob_set(None, vec![(key1.into(), value1.clone())])
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(root, key1).unwrap().unwrap(), value1);

    for _i in 0..1000 {
        let key = HashValue::random();
        let value = Blob::from(key1.to_vec());
        root = test_nonexistent_key_value_update_impl(&tree, &db, root, (key, value));
    }
}

#[test]
fn test_put_blob_sets() {
    let mut keys = vec![];
    let mut values = vec![];
    let total_updates = 20;
    for _i in 0..total_updates {
        keys.push(HashValue::random());
        values.push(Blob::from(HashValue::random().to_vec()));
    }

    let mut root_hashes_one_by_one = vec![];
    let mut batch_one_by_one = TreeUpdateBatch::default();
    {
        let mut iter = keys.clone().into_iter().zip(values.clone().into_iter());
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);

        let mut temp_root = None;
        for _version in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..2 {
                let next = iter.next().unwrap();
                keyed_blob_set.push((next.0.into(), next.1));
            }
            let (root, batch) = tree.put_blob_set(temp_root, keyed_blob_set).unwrap();
            db.write_tree_update_batch(batch.clone()).unwrap();
            temp_root = Some(root);
            root_hashes_one_by_one.push(root);
            batch_one_by_one.node_batch.extend(batch.node_batch);
            batch_one_by_one
                .stale_node_index_batch
                .extend(batch.stale_node_index_batch);
            batch_one_by_one.num_new_leaves += batch.num_new_leaves;
            batch_one_by_one.num_stale_leaves += batch.num_stale_leaves;
        }
    }
    {
        let mut iter = keys.into_iter().zip(values.into_iter());
        let db = MockTreeStore::default();
        let tree = JellyfishMerkleTree::new(&db);
        let mut blob_sets = vec![];
        for _ in 0..10 {
            let mut keyed_blob_set = vec![];
            for _ in 0..2 {
                let val = iter.next().unwrap();
                keyed_blob_set.push((val.0.into(), Some(val.1)));
            }
            blob_sets.push(keyed_blob_set);
        }
        let (root_hashes, batch) = tree.puts(None, blob_sets).unwrap();
        assert_eq!(root_hashes, root_hashes_one_by_one);
        assert_eq!(batch, batch_one_by_one);
    }
}

fn many_keys_get_proof_and_verify_tree_root(seed: &[u8], num_keys: usize) {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut kvs = vec![];
    for _i in 0..num_keys {
        let key = HashValue::random_with_rng(&mut rng);
        let value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((HashValueKey(key), value));
    }

    let (root, batch) = tree.put_blob_set(None, kvs.clone()).unwrap();
    db.write_tree_update_batch(batch).unwrap();

    for (k, v) in &kvs {
        let (value, proof) = tree.get_with_proof(root, k.key_hash()).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(root, k.key_hash(), Some(v)).is_ok());
    }
}

#[test]
fn test_1000_keys() {
    let seed: &[_] = &[1, 2, 3, 4];
    many_keys_get_proof_and_verify_tree_root(seed, 1000);
}

fn many_versions_get_proof_and_verify_tree_root(seed: &[u8], num_versions: usize) {
    assert!(seed.len() < 32);
    let mut actual_seed = [0u8; 32];
    actual_seed[..seed.len()].copy_from_slice(seed);
    let mut rng: StdRng = StdRng::from_seed(actual_seed);

    let db = MockTreeStore::default();
    let tree = JellyfishMerkleTree::new(&db);

    let mut kvs = vec![];

    for _i in 0..num_versions {
        let key = HashValue::random_with_rng(&mut rng);
        let value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
        let new_value = Blob::from(HashValue::random_with_rng(&mut rng).to_vec());
        kvs.push((key, value.clone(), new_value.clone()));
    }

    let mut roots = vec![];
    let mut current_root = None;
    for (_idx, kvs) in kvs.iter().enumerate() {
        let (root, batch) = tree
            .put_blob_set(current_root, vec![(kvs.0.into(), kvs.1.clone())])
            .unwrap();
        roots.push(root);
        db.write_tree_update_batch(batch).unwrap();
        current_root = Some(root);
    }

    // Update value of all keys
    for (_idx, kvs) in kvs.iter().enumerate() {
        let (root, batch) = tree
            .put_blob_set(current_root, vec![(kvs.0.into(), kvs.2.clone())])
            .unwrap();
        roots.push(root);
        db.write_tree_update_batch(batch).unwrap();
        current_root = Some(root);
    }

    for (i, (k, v, _)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i..i + num_versions);
        let history_root = roots[random_version];
        let (value, proof) = tree.get_with_proof(history_root, *k).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(history_root, *k, Some(v)).is_ok());
    }

    for (i, (k, _, v)) in kvs.iter().enumerate() {
        let random_version = rng.gen_range(i + num_versions..2 * num_versions);
        let history_root = roots[random_version];
        let (value, proof) = tree.get_with_proof(history_root, *k).unwrap();
        assert_eq!(value.unwrap(), *v);
        assert!(proof.verify(history_root, *k, Some(v)).is_ok());
    }
}

#[test]
fn test_1000_versions() {
    let seed: &[_] = &[1, 2, 3, 4];
    many_versions_get_proof_and_verify_tree_root(seed, 1000);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_get_with_proof1(
        (existent_kvs, nonexistent_keys) in hash_map(
            any::<HashValueKey>(),
            any::<Blob>(),
            1..1000,
        )
            .prop_flat_map(|kvs| {
                let kvs_clone = kvs.clone();
                (
                    Just(kvs),
                    vec(
                        any::<HashValueKey>().prop_filter(
                            "Make sure these keys do not exist in the tree.",
                            move |key| !kvs_clone.contains_key(key),
                        ),
                        100,
                    ),
                )
            })
    ) {
        let (db, root_hash_option) = init_mock_db(&existent_kvs);
        let tree = JellyfishMerkleTree::new(&db);
        let root_hash = root_hash_option.unwrap();
        test_existent_keys_impl(&tree, root_hash, &existent_kvs);
        test_nonexistent_keys_impl(&tree, root_hash, &nonexistent_keys);
    }

    #[test]
    fn test_get_with_proof2(
        key1 in any::<HashValueKey>()
            .prop_filter(
                "Can't be 0xffffff...",
                |key| *key != HashValue::new([0xff; HashValue::LENGTH]).into(),
            ),
        accounts in vec(any::<Blob>(), 2),
    ) {
        let key2 = HashValueKey(plus_one(key1.0));

        let mut kvs = HashMap::new();
        kvs.insert(key1, accounts[0].clone());
        kvs.insert(key2, accounts[1].clone());

        let (db, root_hash_option) = init_mock_db(&kvs);
        let tree = JellyfishMerkleTree::new(&db);

        test_existent_keys_impl(&tree, root_hash_option.unwrap(), &kvs);
    }

    #[test]
    fn test_get_range_proof(
        (btree, n) in btree_map(any::<HashValueKey>(), any::<Blob>(), 1..50)
            .prop_flat_map(|btree| {
                let len = btree.len();
                (Just(btree), 0..len)
            })
    ) {
        let (db, root_hash_option) = init_mock_db(&btree.clone().into_iter().collect());
        let tree = JellyfishMerkleTree::new(&db);
        let root_hash = root_hash_option.unwrap();
        let nth_key = *btree.keys().nth(n).unwrap();
        let proof = tree.get_range_proof(root_hash, nth_key.key_hash()).unwrap();
        verify_range_proof(
            root_hash,
            btree.into_iter().take(n + 1).collect(),
            proof,
        );
    }
}

fn test_existent_keys_impl<'a>(
    tree: &JellyfishMerkleTree<'a, HashValueKey, MockTreeStore>,
    root_hash: HashValue,
    existent_kvs: &HashMap<HashValueKey, Blob>,
) {
    for (key, value) in existent_kvs {
        let (value_in_tree, proof) = tree.get_with_proof(root_hash, key.key_hash()).unwrap();
        assert!(proof
            .verify(root_hash, key.key_hash(), value_in_tree.as_ref())
            .is_ok());
        assert_eq!(value_in_tree.unwrap(), *value);
    }
}

fn test_nonexistent_keys_impl<'a>(
    tree: &JellyfishMerkleTree<'a, HashValueKey, MockTreeStore>,
    root_hash: HashValue,
    nonexistent_keys: &[HashValueKey],
) {
    for key in nonexistent_keys {
        let (value_in_tree, proof) = tree.get_with_proof(root_hash, key.key_hash()).unwrap();
        assert!(proof
            .verify(root_hash, key.key_hash(), value_in_tree.as_ref())
            .is_ok());
        assert!(value_in_tree.is_none());
    }
}

fn test_nonexistent_key_value_update_impl<'a>(
    tree: &JellyfishMerkleTree<'a, HashValueKey, MockTreeStore>,
    db: &MockTreeStore,
    root_hash: HashValue,
    noneexistent_kv: (HashValue, Blob),
) -> HashValue {
    let (key, value) = noneexistent_kv;
    let (value_in_tree, mut proof) = tree.get_with_proof(root_hash, key).unwrap();
    assert!(proof.verify(root_hash, key, value_in_tree.as_ref()).is_ok());
    assert!(value_in_tree.is_none());

    let new_root_by_proof = proof.update_leaf(key, &value).unwrap();

    let (root, batch) = tree
        .put_blob_set(Some(root_hash), vec![(key.into(), value.clone())])
        .unwrap();
    db.write_tree_update_batch(batch).unwrap();
    assert_eq!(tree.get(root, key).unwrap().unwrap(), value);

    let (value, new_proof) = tree.get_with_proof(root, key).unwrap();
    assert!(value.is_some());
    assert_eq!(proof, new_proof);

    assert_eq!(new_root_by_proof, root);
    root
}

/// Checks if we can construct the expected root hash using the entries in the btree and the proof.
fn verify_range_proof<K: RawKey>(
    expected_root_hash: HashValue,
    btree: BTreeMap<K, Blob>,
    proof: SparseMerkleRangeProof,
) {
    // For example, given the following sparse Merkle tree:
    //
    //                   root
    //                  /     \
    //                 /       \
    //                /         \
    //               o           o
    //              / \         / \
    //             a   o       o   h
    //                / \     / \
    //               o   d   e   X
    //              / \         / \
    //             b   c       f   g
    //
    // we transform the keys as follows:
    //   a => 00,
    //   b => 0100,
    //   c => 0101,
    //   d => 011,
    //   e => 100,
    //   X => 101
    //   h => 11
    //
    // Basically, the suffixes that doesn't affect the common prefix of adjacent leaves are
    // discarded. In this example, we assume `btree` has the keys `a` to `e` and the proof has `X`
    // and `h` in the siblings.

    // Now we want to construct a set of key-value pairs that covers the entire set of leaves. For
    // `a` to `e` this is simple -- we just insert them directly into this set. For the rest of the
    // leaves, they are represented by the siblings, so we just make up some keys that make sense.
    // For example, for `X` we just use 101000... (more zeros omitted), because that is one key
    // that would cause `X` to end up in the above position.
    let mut btree1 = BTreeMap::new();
    for (key, blob) in &btree {
        let leaf = LeafNode::new(key.clone(), blob.clone());
        btree1.insert(key.key_hash(), leaf.crypto_hash());
    }
    // Using the above example, `last_proven_key` is `e`. We look at the path from root to `e`.
    // For each 0-bit, there should be a sibling in the proof. And we use the path from root to
    // this position, plus a `1` as the key.
    let last_proven_key = btree
        .keys()
        .last()
        .expect("We are proving at least one key.")
        .key_hash();
    for (i, sibling) in last_proven_key
        .iter_bits()
        .enumerate()
        .filter_map(|(i, bit)| if !bit { Some(i) } else { None })
        .zip(proof.right_siblings().iter().rev())
    {
        // This means the `i`-th bit is zero. We take `i` bits from `last_proven_key` and append a
        // one to make up the key for this sibling.
        let mut buf: Vec<_> = last_proven_key.iter_bits().take(i).collect();
        buf.push(true);
        // The rest doesn't matter, because they don't affect the position of the node. We just
        // add zeros.
        buf.resize(HashValue::LENGTH_IN_BITS, false);
        let key = HashValue::from_bit_iter(buf.into_iter()).unwrap();
        btree1.insert(key, *sibling);
    }

    // Now we do the transformation (removing the suffixes) described above.
    let mut kvs = vec![];
    for (key, value) in &btree1 {
        // The length of the common prefix of the previous key and the current key.
        let prev_common_prefix_len =
            prev_key(&btree1, key).map(|pkey| pkey.common_prefix_bits_len(*key));
        // The length of the common prefix of the next key and the current key.
        let next_common_prefix_len =
            next_key(&btree1, key).map(|nkey| nkey.common_prefix_bits_len(*key));

        // We take the longest common prefix of the current key and its neighbors. That's how much
        // we need to keep.
        let len = match (prev_common_prefix_len, next_common_prefix_len) {
            (Some(plen), Some(nlen)) => std::cmp::max(plen, nlen),
            (Some(plen), None) => plen,
            (None, Some(nlen)) => nlen,
            (None, None) => 0,
        };
        let transformed_key: Vec<_> = key.iter_bits().take(len + 1).collect();
        kvs.push((transformed_key, *value));
    }

    assert_eq!(compute_root_hash(kvs), expected_root_hash);
}

/// Computes the root hash of a sparse Merkle tree. `kvs` consists of the entire set of key-value
/// pairs stored in the tree.
fn compute_root_hash(kvs: Vec<(Vec<bool>, HashValue)>) -> HashValue {
    let mut kv_ref = vec![];
    for (key, value) in &kvs {
        kv_ref.push((&key[..], *value));
    }
    compute_root_hash_impl(kv_ref)
}

fn compute_root_hash_impl(kvs: Vec<(&[bool], HashValue)>) -> HashValue {
    assert!(!kvs.is_empty());

    // If there is only one entry, it is the root.
    if kvs.len() == 1 {
        return kvs[0].1;
    }

    // Otherwise the tree has more than one leaves, which means we can find which ones are in the
    // left subtree and which ones are in the right subtree. So we find the first key that starts
    // with a 1-bit.
    let left_hash;
    let right_hash;
    match kvs.iter().position(|(key, _value)| key[0]) {
        Some(0) => {
            // Every key starts with a 1-bit, i.e., they are all in the right subtree.
            left_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
            right_hash = compute_root_hash_impl(reduce(&kvs));
        }
        Some(index) => {
            // Both left subtree and right subtree have some keys.
            left_hash = compute_root_hash_impl(reduce(&kvs[..index]));
            right_hash = compute_root_hash_impl(reduce(&kvs[index..]));
        }
        None => {
            // Every key starts with a 0-bit, i.e., they are all in the left subtree.
            left_hash = compute_root_hash_impl(reduce(&kvs));
            right_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
        }
    }

    SparseMerkleInternalNode::new(left_hash, right_hash).hash()
}

/// Reduces the problem by removing the first bit of every key.
fn reduce<'a>(kvs: &'a [(&[bool], HashValue)]) -> Vec<(&'a [bool], HashValue)> {
    kvs.iter().map(|(key, value)| (&key[1..], *value)).collect()
}

/// Returns the key immediately before `key` in `btree`.
fn prev_key<K, V>(btree: &BTreeMap<K, V>, key: &K) -> Option<K>
where
    K: Clone + Ord,
{
    btree
        .range((Bound::Unbounded, Bound::Excluded(key)))
        .next_back()
        .map(|(k, _v)| k.clone())
}

/// Returns the key immediately after `key` in `btree`.
fn next_key<K, V>(btree: &BTreeMap<K, V>, key: &K) -> Option<K>
where
    K: Clone + Ord,
{
    btree
        .range((Bound::Excluded(key), Bound::Unbounded))
        .next()
        .map(|(k, _v)| k.clone())
}
