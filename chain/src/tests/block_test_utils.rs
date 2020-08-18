// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_helper;
use config::NodeConfig;
use crypto::HashValue;
use ethereum_types::U256;
use proptest::{collection::vec, prelude::*};
use starcoin_genesis::Genesis;
use starcoin_types::block::{Block, BlockBody, BlockHeader};
use starcoin_types::chain_config::ChainNetwork;
use starcoin_types::transaction::SignedUserTransaction;
use std::sync::Arc;
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
        0, //block_gas_limit
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
        let header = gen_header(parent_header, HashValue::zero(), HashValue::zero());
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
    leaf.prop_recursive(depth, depth, 2, child)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn test_block_gen_and_insert(
        blocks in block_forest(
            // recursion depth
            2)
    ){
    let config = Arc::new(NodeConfig::random_for_test());
    let mut _block_chain = test_helper::gen_blockchain_for_test(config).unwrap();

    for _block in blocks {
        // TODO check parent is exist
        // TODO execute block,
        // let result = block_chain.apply(block);
        // TODO insert to storage
    }
    }
}
