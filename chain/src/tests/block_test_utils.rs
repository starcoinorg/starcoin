// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_helper;
use config::NodeConfig;
use crypto::HashValue;
use ethereum_types::U256;
use logger::prelude::*;
use proptest::{collection::vec, prelude::*};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_genesis::Genesis;
use starcoin_statedb::ChainStateDB;
use starcoin_traits::ChainWriter;
use starcoin_types::block::{Block, BlockBody, BlockHeader};
use starcoin_types::chain_config::ChainNetwork;
use starcoin_types::transaction::{SignedUserTransaction, Transaction};
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;

type LinearizedBlockForest = Vec<Block>;

/// This produces the genesis block
pub fn genesis_strategy() -> impl Strategy<Value = Block> {
    Just(Genesis::load(ChainNetwork::Test).unwrap().block().clone())
}

/// Offers the genesis block.
pub fn leaf_strategy() -> impl Strategy<Value = Block> {
    genesis_strategy().boxed()
}

prop_compose! {
    pub fn new_block_with_header(
        header: BlockHeader,
        max_txn_per_block: usize,
    )
    (
        user_txns in vec(any::<SignedUserTransaction>(), 1..=max_txn_per_block),
        header in Just(header),
        // block in block_strategy
    ) -> Block {
        let body = BlockBody::new(user_txns, None);
        Block::new_with_body(header, body)
    }
}
///gen header by given parent_header
fn gen_header(
    parent_header: BlockHeader,
    acc_root: HashValue,
    state_root: HashValue,
) -> BlockHeader {
    BlockHeader::new(
        parent_header.id(),
        parent_header.accumulator_root(),
        parent_header.timestamp() + 1,
        parent_header.number() + 1,
        parent_header.author,
        acc_root,
        state_root,
        0,
        10_000, //block_gas_limit
        U256::zero(),
        0,
        None,
    )
}

prop_compose! {
    pub fn new_block_with_parent_header(
        parent_header: BlockHeader,
        max_txn_per_block: usize,
    )
    (
        user_txns in vec(any::<SignedUserTransaction>(), 1..=max_txn_per_block),
        parent_header in Just(parent_header),
        // block in block_strategy
    ) -> Block {
    //transfer transactions
    let txns = {
            let mut t = vec![];
            t.extend(
                user_txns
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };
    //gen state_root, acc_root
        let (state_root, acc_root) = gen_root_hashes(
            parent_header.accumulator_root(),
            parent_header.state_root(),
            txns,
            10_000 /*block_gas_limit*/
        );
        let header = gen_header(parent_header, state_root, acc_root);
        let body = BlockBody::new(user_txns, None);
        Block::new_with_body(header, body)
    }
}

prop_compose! {
    pub fn new_header(
        parent_header: BlockHeader,
    )
    (
        parent_header in Just(parent_header),
    )
       -> BlockHeader {
       gen_header(parent_header, HashValue::zero(), HashValue::zero())
    }
}

prop_compose! {
    /// This creates a child with a parent on its left
    pub fn child(
        block_forest_strategy: impl Strategy<Value = LinearizedBlockForest>,
    )
    (
        (forest_vec, parent_idx) in block_forest_strategy
            .prop_flat_map(|forest_vec| {
                let len = forest_vec.len();
                (Just(forest_vec), 0..len)
            })
    )
    (
        block in new_block_with_parent_header(forest_vec[parent_idx].header().clone(), 5),
        mut forest in Just(forest_vec)
    ) -> LinearizedBlockForest {
        forest.push(block);
        forest
    }
}

/// This creates a block forest with keys extracted from a specific
/// vector
pub fn block_forest(depth: u32) -> impl Strategy<Value = LinearizedBlockForest> {
    let leaf = leaf_strategy().prop_map(|block| vec![block]);
    leaf.prop_recursive(depth, depth, 4, child)
}

fn gen_root_hashes(
    pre_accumulator_root: HashValue,
    pre_state_root: HashValue,
    block_txns: Vec<Transaction>,
    block_gat_limit: u64,
) -> (HashValue, HashValue) {
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    //state_db
    let chain_state = ChainStateDB::new(storage.clone(), Some(pre_state_root));
    if let Ok(executed_data) = executor::block_execute(&chain_state, block_txns, block_gat_limit) {
        let txn_accumulator = MerkleAccumulator::new(
            pre_accumulator_root,
            vec![],
            0,
            0,
            AccumulatorStoreType::Transaction,
            storage,
        )
        .unwrap();

        let included_txn_info_hashes: Vec<_> = executed_data
            .txn_infos
            .iter()
            .map(|info| info.id())
            .collect();
        let (accumulator_root, _first_leaf_idx) =
            txn_accumulator.append(&included_txn_info_hashes).unwrap();
        (accumulator_root, executed_data.state_root)
    } else {
        (HashValue::zero(), HashValue::zero())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_block_gen_and_insert(
        blocks in block_forest(
            // recursion depth
            10)
    ){
    let config = Arc::new(NodeConfig::random_for_test());
    let mut block_chain = test_helper::gen_blockchain_for_test(config).unwrap();

    for block in blocks {
        let result = block_chain.apply(block);
        info!("{:?}", result);
    }
    }
}
