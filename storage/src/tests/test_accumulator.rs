// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cache_storage::CacheStorage;
use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::Storage;
use crypto::HashValue;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{AccumulatorNode, AccumulatorReader, AccumulatorWriter};
use std::sync::Arc;

#[test]
fn test_storage() {
    // let cache_storage = Arc::new(CacheStorage::new());
    let tmpdir = libra_temppath::TempPath::new();
    let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage)).unwrap();

    let acc_node = AccumulatorNode::new_leaf(NodeIndex::new(1), HashValue::random());
    let node_hash = acc_node.hash();
    storage
        .accumulator_storage
        .save_node(AccumulatorStoreType::Transaction, acc_node.clone())
        .unwrap();
    let acc_node2 = storage
        .accumulator_storage
        .get_node(AccumulatorStoreType::Transaction, node_hash)
        .unwrap()
        .unwrap();
    assert_eq!(acc_node, acc_node2);
    storage
        .accumulator_storage
        .save_node(AccumulatorStoreType::Block, acc_node.clone())
        .unwrap();
    let acc_node3 = storage
        .accumulator_storage
        .get_node(AccumulatorStoreType::Block, node_hash.clone())
        .unwrap()
        .unwrap();
    assert_eq!(acc_node, acc_node3);
}
