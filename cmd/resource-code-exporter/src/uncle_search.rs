// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;

use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion, Store,
};
use std::{path::Path, sync::Arc};

/// Return Block HashValue and block height that have uncle blocks
pub fn run(
    db_path: &Path,
    anchor_hash: HashValue,
    max_search_count: Option<usize>,
) -> anyhow::Result<Vec<(HashValue, usize)>> {
    let db_storage = DBStorage::open_with_cfs(
        db_path,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        false,
        Default::default(),
        None,
    )?;
    let storage = Arc::new(Storage::new(StorageInstance::new_db_instance(db_storage))?);
    search_uncles_with_db(
        storage.clone(),
        anchor_hash,
        max_search_count.unwrap_or(1000),
    )
}

pub fn search_uncles_with_db(
    storage: Arc<dyn Store>,
    anchor_block_hash: HashValue,
    max_search_count: usize,
) -> anyhow::Result<Vec<(HashValue, usize)>> {
    let anchor_block = storage
        .get_block_by_hash(anchor_block_hash)?
        .ok_or_else(|| anyhow::format_err!("Anchor block not found: {}", anchor_block_hash))?;

    let mut result = Vec::new();
    let mut current_block = anchor_block;
    let mut searched_count = 0;

    while searched_count < max_search_count {
        if let Some(uncles) = current_block.uncles() {
            if !uncles.is_empty() {
                result.push((current_block.id(), current_block.header().number() as usize));
            }
        }
        let parent_hash = current_block.header().parent_hash();
        let parent_block = match storage.get_block_by_hash(parent_hash)? {
            Some(block) => block,
            None => break,
        };

        current_block = parent_block;
        searched_count += 1;
    }

    Ok(result)
}

#[test]
fn test_uncle_search_db() {
    use starcoin_chain::ChainReader;
    use starcoin_chain::{BlockChain, ChainWriter};
    use starcoin_config::ChainNetwork;
    use starcoin_consensus::Consensus;
    use starcoin_types::account_address::AccountAddress;
    use test_helper::chain::gen_chain_for_test_and_return_statedb;

    // Build the necessary facilities
    let net = ChainNetwork::new_test();

    let (mut chain, _statedb) = gen_chain_for_test_and_return_statedb(&net, None).unwrap();
    let storage = chain.get_storage();
    let storage2 = chain.get_storage2();

    // Now that our chain is ready after genesis, we need to generate a new block for chain
    let miner_account_address = AccountAddress::random();

    // Generate a few blocks on the main chain
    for _ in 0..3 {
        let (block_template, _) = chain
            .create_block_template_simple(miner_account_address)
            .unwrap();
        let block = chain
            .consensus()
            .create_block(block_template, net.time_service().as_ref())
            .unwrap();
        chain.apply(block).unwrap();
    }

    // Generate another block and assign an uncle block to it
    // First, create a fork from an earlier block to create an uncle
    let fork_block_hash = chain.get_hash_by_number(1).unwrap().unwrap();
    let mut fork_chain = BlockChain::new(
        net.time_service(),
        fork_block_hash,
        storage.clone(),
        storage2.clone(),
        None,
    )
    .unwrap();

    // Create a block on the fork (this will be our uncle)
    let (uncle_block_template, _) = fork_chain
        .create_block_template_simple(miner_account_address)
        .unwrap();
    let uncle_block = fork_chain
        .consensus()
        .create_block(uncle_block_template, net.time_service().as_ref())
        .unwrap();
    let uncle_header = uncle_block.header().clone();
    fork_chain.apply(uncle_block).unwrap();

    // Now create a block on the main chain that includes the uncle
    let (block_template, _) = chain
        .create_block_template_simple_with_uncles(miner_account_address, vec![uncle_header.clone()])
        .unwrap();
    let block_with_uncle = chain
        .consensus()
        .create_block(block_template, net.time_service().as_ref())
        .unwrap();
    chain.apply(block_with_uncle).unwrap();

    // using search_uncles_with_db to test this chain
    let head_block_hash = chain.current_header().id();
    let result = search_uncles_with_db(storage.clone(), head_block_hash, 10).unwrap();

    // check the result list has 1 block hash
    assert_eq!(result.len(), 1, "Should find exactly one block with uncles");

    let (found_block_hash, found_block_number) = result[0];
    let found_block = storage
        .get_block_by_hash(found_block_hash)
        .unwrap()
        .unwrap();

    // Verify the found block actually contains uncles
    assert!(
        found_block.uncles().is_some(),
        "Found block should have uncles"
    );
    assert!(
        !found_block.uncles().unwrap().is_empty(),
        "Found block should have non-empty uncles"
    );
    assert_eq!(found_block.header().number() as usize, found_block_number);

    // Verify the uncle header is in the found block
    let uncles = found_block.uncles().unwrap();
    assert!(
        uncles.contains(&uncle_header),
        "Found block should contain our uncle header"
    );
}
