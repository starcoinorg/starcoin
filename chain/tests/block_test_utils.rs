// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::HashValue;
use logger::prelude::*;
use proptest::{collection::vec, prelude::*};
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::ChainWriter;
use starcoin_config::{ChainNetwork, NodeConfig};
use starcoin_executor::{block_execute, DEFAULT_EXPIRATION_TIME};
use starcoin_genesis::Genesis;
use starcoin_statedb::ChainStateDB;
use starcoin_transaction_builder::build_empty_script;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::proptest_types::{AccountInfoUniverse, Index, SignatureCheckedTransactionGen};
use starcoin_types::transaction::{SignedUserTransaction, Transaction, TransactionPayload};
use starcoin_types::{
    block::{Block, BlockBody, BlockHeader},
    block_metadata::BlockMetadata,
    U256,
};
use std::convert::TryFrom;
use std::sync::Arc;
use storage::storage::StorageInstance;
use storage::Storage;

type LinearizedBlockForest = Vec<Block>;

fn get_storage() -> impl Strategy<Value = Storage> {
    Just(Storage::new(StorageInstance::new_cache_instance()).unwrap())
}

/// This produces the genesis block
pub fn genesis_strategy(storage: Arc<Storage>) -> impl Strategy<Value = Block> {
    let net = &ChainNetwork::new_test();
    let genesis = Genesis::load_or_build(net).unwrap();
    genesis.execute_genesis_block(net, storage).unwrap();
    Just(genesis.block().clone())
}

/// Offers the genesis block.
pub fn leaf_strategy(storage: Arc<Storage>) -> impl Strategy<Value = Block> {
    genesis_strategy(storage).boxed()
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
        Block::new(header, body)
    }
}
///gen header by given parent_header
fn gen_header(
    parent_header: BlockHeader,
    acc_root: HashValue,
    state_root: HashValue,
    body_hash: HashValue,
) -> BlockHeader {
    BlockHeader::new(
        parent_header.id(),
        parent_header.timestamp() + 1,
        parent_header.number() + 1,
        parent_header.author(),
        acc_root,
        //TODO fixme
        HashValue::random(),
        state_root,
        0,
        U256::zero(),
        body_hash,
        parent_header.chain_id(),
        0,
        BlockHeaderExtra::new([0u8; 4]),
    )
}

fn gen_script_payload() -> TransactionPayload {
    TransactionPayload::ScriptFunction(build_empty_script())
}

fn txn_transfer(
    universe: &mut AccountInfoUniverse,
    gens: Vec<(Index, SignatureCheckedTransactionGen)>,
) -> Vec<Transaction> {
    let mut temp_index: Option<Index> = None;
    let expired = universe.time_service().now_secs() + DEFAULT_EXPIRATION_TIME;
    gens.into_iter()
        .map(|(index, gen)| {
            if temp_index.is_none() {
                temp_index = Some(index);
            }
            Transaction::UserTransaction(
                gen.materialize(
                    temp_index.unwrap(),
                    universe,
                    expired,
                    Some(gen_script_payload()),
                )
                .into_inner(),
            )
        })
        .collect::<Vec<_>>()
}

prop_compose! {
    pub fn new_block_with_parent_header(
        storage: Arc<Storage>,
        parent_header: BlockHeader,
        max_txn_per_block: usize,
    )
    (
        gens in vec(
                (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
                1..max_txn_per_block
            ),
        parent_header in Just(parent_header),
        storage in Just(storage),
    ) -> Block {
    //transfer transactions
    let mut account = AccountInfoUniverse::default().unwrap();
    let mut txns = txn_transfer(&mut account, gens);
    let user_txns = {
            let mut t=   vec![];
            t.extend(
                txns
                    .iter()
                    .cloned()
                    .map(|txn| SignedUserTransaction::try_from(txn).unwrap()),
            );
            t
        };
    let p_header = parent_header.clone();
    let block_metadata = BlockMetadata::new(
        p_header.parent_hash(),
        account.time_service().now_millis(),
        p_header.author(),
        p_header.author_auth_key(),
        0,
        p_header.number() + 1,
        account.chain_id(),
        p_header.gas_used(),
    );
    txns.insert(0, Transaction::BlockMetadata(block_metadata));

    //gen state_root, acc_root
    let (state_root, acc_root) = gen_root_hashes(
        storage,
        parent_header.txn_accumulator_root(),
        parent_header.state_root(),
        txns,
        u64::max_value(), /*block_gas_limit*/
    );
    let body = BlockBody::new(user_txns, None);
    let header = gen_header(parent_header, state_root, acc_root, body.hash());
    Block::new(header, body)
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
       gen_header(parent_header, HashValue::zero(), HashValue::zero(), HashValue::zero())
    }
}

prop_compose! {
    /// This creates a child with a parent on its left
    pub fn child(storage: Arc<Storage>,
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
        block in new_block_with_parent_header(storage.clone(), forest_vec[parent_idx].header().clone(), 2),
        mut forest in Just(forest_vec)
    ) -> LinearizedBlockForest {
        forest.push(block);
        forest
    }
}

/// This creates a block forest with keys extracted from a specific
/// vector
pub fn block_forest(depth: u32) -> impl Strategy<Value = LinearizedBlockForest> {
    let temp_depth = depth;
    get_storage().prop_flat_map(move |storage| {
        let store = Arc::new(storage);
        leaf_strategy(store.clone())
            .prop_map(|block| vec![block])
            .prop_recursive(temp_depth, temp_depth, 4, move |inner| {
                child(store.clone(), inner)
            })
    })
    // leaf.prop_recursive(depth, depth, 4, child)
}

fn gen_root_hashes(
    storage: Arc<Storage>,
    pre_accumulator_root: HashValue,
    pre_state_root: HashValue,
    block_txns: Vec<Transaction>,
    block_gat_limit: u64,
) -> (HashValue, HashValue) {
    //state_db
    let chain_state = ChainStateDB::new(storage.clone(), Some(pre_state_root));

    match block_execute(&chain_state, block_txns, block_gat_limit, None) {
        Ok(executed_data) => {
            let txn_accumulator = MerkleAccumulator::new(
                pre_accumulator_root,
                vec![],
                0,
                0,
                Arc::new(storage.get_transaction_accumulator_storage()),
            );

            let included_txn_info_hashes: Vec<_> = executed_data
                .txn_infos
                .iter()
                .map(|info| info.id())
                .collect();
            let accumulator_root = txn_accumulator.append(&included_txn_info_hashes).unwrap();
            (accumulator_root, executed_data.state_root)
        }
        // Err(err) => {
        //     (HashValue::zero(), HashValue::zero())
        // }
        _ => (HashValue::zero(), HashValue::zero()),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_block_gen_and_insert(
        blocks in block_forest(
            // recursion depth
            10)) {
        let config = Arc::new(NodeConfig::random_for_test());
        let mut block_chain = test_helper::gen_blockchain_for_test(config.net()).unwrap();
        // blocks in ;
        for block in blocks {
            if !block.header().is_genesis() {
                let result = block_chain.apply(block.clone());
                info!("{:?}", result);
            }
        }
    }

    #[test]
    fn test_txn_execute(
        storage in get_storage(),
        gens in vec(
                (any::<Index>(), any::<SignatureCheckedTransactionGen>()),
                1..2
            ), ) {
        let chain_state = ChainStateDB::new(Arc::new(storage), None);
        let mut account = AccountInfoUniverse::default().unwrap();
        let txns = txn_transfer(&mut account, gens);
        let result = block_execute(&chain_state, txns, 0, None);
        info!("execute result: {:?}", result);
    }
}
