// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::db_storage::DBStorage;
use crate::storage::StorageInstance;
use crate::Storage;
use crypto::HashValue;
use starcoin_accumulator::node_index::NodeIndex;
use starcoin_accumulator::{AccumulatorNode, AccumulatorTreeStore};
use starcoin_config::RocksdbConfig;

#[test]
fn test_storage() {
    let storage = Storage::new(StorageInstance::new_db_instance(
        DBStorage::new(
            starcoin_config::temp_path().as_ref(),
            RocksdbConfig::default(),
            None,
        )
        .unwrap(),
    ))
    .unwrap();

    let acc_node = AccumulatorNode::new_leaf(NodeIndex::from_inorder_index(1), HashValue::random());
    let node_hash = acc_node.hash();
    storage
        .transaction_accumulator_storage
        .save_node(acc_node.clone())
        .unwrap();
    let acc_node2 = storage
        .transaction_accumulator_storage
        .get_node(node_hash)
        .unwrap()
        .unwrap();
    assert_eq!(acc_node, acc_node2);
    storage
        .block_accumulator_storage
        .save_node(acc_node.clone())
        .unwrap();
    let acc_node3 = storage
        .block_accumulator_storage
        .get_node(node_hash)
        .unwrap()
        .unwrap();
    assert_eq!(acc_node, acc_node3);
}
