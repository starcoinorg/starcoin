// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use network_p2p_core::export::log::debug;
use starcoin_account_api::AccountInfo;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_config::{
    upgrade_config::vm1_offline_height, BuiltinNetworkID, ChainNetwork, RocksdbConfig,
    DEFAULT_CACHE_SIZE,
};
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    cache_storage::CacheStorage, db_storage::DBStorage, storage::StorageInstance, Storage,
};
use starcoin_types::startup_info::ChainInfo;
use starcoin_vm2_storage::{
    cache_storage::CacheStorage as CacheStorage2,
    db_storage::{DBStorage as DBStorage2, RocksdbConfig as RocksdbConfig2},
    storage::StorageInstance as StorageInstance2,
    Storage as Storage2,
};
use std::{path::Path, path::PathBuf, sync::Arc};

pub fn gen_chain_for_test_and_return_statedb(
    net: &ChainNetwork,
) -> Result<(BlockChain, ChainStateDB)> {
    gen_chain_for_test_and_return_statedb_with_storage_type(net, StorageType::Cache)
}

/// Storage type for testing
enum StorageType {
    Cache,
    TempDir(PathBuf),
}

/// Initialize storage for test with temporary directory (similar to Genesis::init_and_check_storage)
/// This function creates a temporary directory and uses it for persistent storage instead of cache
fn init_storage_for_test_with_temp_dir(
    net: &ChainNetwork,
    temp_dir: &Path,
) -> Result<(Arc<Storage>, Arc<Storage2>, ChainInfo, Genesis)> {
    debug!("init storage by genesis for test with temp dir.");

    // Create temporary directory
    let db_path = temp_dir.join("starcoindb");
    let db_path2 = temp_dir.join("starcoindb2");

    // Create storage instances with temporary directories
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new_with_capacity(DEFAULT_CACHE_SIZE, None), // Small cache for testing
        DBStorage::new(&db_path, RocksdbConfig::default(), None)?,
    ))?);

    let storage2 = Arc::new(Storage2::new(StorageInstance2::new_cache_and_db_instance(
        CacheStorage2::new_with_capacity(DEFAULT_CACHE_SIZE, None), // Small cache for testing
        DBStorage2::new(&db_path2, RocksdbConfig2::default(), None)?,
    ))?);

    // Load or build genesis
    let genesis = Genesis::load_or_build(net)?;

    // Execute genesis block
    let chain_info = genesis.execute_genesis_block(net, storage.clone(), storage2.clone())?;

    Ok((storage, storage2, chain_info, genesis))
}

/// Generate chain for test with specified storage type
fn gen_chain_for_test_and_return_statedb_with_storage_type(
    net: &ChainNetwork,
    storage_type: StorageType,
) -> Result<(BlockChain, ChainStateDB)> {
    match storage_type {
        StorageType::Cache => {
            let (storage, storage2, chain_info, _) =
                Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

            let block_chain = BlockChain::new(
                net.time_service(),
                chain_info.head().id(),
                storage.clone(),
                storage2.clone(),
                None,
            )?;
            let state_root = block_chain.chain_state_reader().state_root();
            Ok((block_chain, ChainStateDB::new(storage, Some(state_root))))
        }
        StorageType::TempDir(temp_dir) => {
            let (storage, storage2, chain_info, _) =
                init_storage_for_test_with_temp_dir(net, &temp_dir)?;

            let block_chain = BlockChain::new(
                net.time_service(),
                chain_info.head().id(),
                storage.clone(),
                storage2.clone(),
                None,
            )?;
            let state_root = block_chain.chain_state_reader().state_root();
            Ok((block_chain, ChainStateDB::new(storage, Some(state_root))))
        }
    }
}

pub fn vm1_testnet() -> Result<ChainNetwork> {
    let chain_name = "vm1-testnet".to_string();
    let net = ChainNetwork::new_custom(
        chain_name,
        124.into(),
        BuiltinNetworkID::Test.genesis_config().clone(),
        BuiltinNetworkID::Test.genesis_config2().clone(),
    )
    .unwrap();

    let vm1_offline_height = vm1_offline_height(124.into());
    assert_eq!(vm1_offline_height, u64::MAX);

    Ok(net)
}

/// Generate chain for test with temporary directory storage (for large data scenarios)
pub fn gen_chain_for_test_and_return_statedb_with_temp_storage(
    net: &ChainNetwork,
    temp_dir: PathBuf,
) -> Result<(BlockChain, ChainStateDB)> {
    gen_chain_for_test_and_return_statedb_with_storage_type(net, StorageType::TempDir(temp_dir))
}

pub fn gen_blockchain_for_test(net: &ChainNetwork) -> Result<BlockChain> {
    let (chain, _) =
        gen_chain_for_test_and_return_statedb_with_storage_type(net, StorageType::Cache)?;
    Ok(chain)
}

pub fn gen_blockchain_with_blocks_for_test(count: u64, net: &ChainNetwork) -> Result<BlockChain> {
    let mut block_chain = gen_blockchain_for_test(net)?;
    let miner_account = AccountInfo::random();
    for _i in 0..count {
        let (block_template, _) = block_chain
            .create_block_template_simple(*miner_account.address())
            .unwrap();
        let block = block_chain
            .consensus()
            .create_block(block_template, net.time_service().as_ref())?;
        block_chain.apply(block)?;
    }

    Ok(block_chain)
}
