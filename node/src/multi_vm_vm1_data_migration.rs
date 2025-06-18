// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    block::legacy::BlockInnerStorage, db_storage::DBStorage, storage::CodecKVStore,
    storage::StorageInstance, Storage, StorageVersion,
};
use starcoin_types::state_set::ChainStateSet;
use std::sync::Arc;

pub fn migrate_from_data_dir(
    target_statedb: &ChainStateDB,
    block_hash: HashValue,
    db_path: std::path::PathBuf,
) -> anyhow::Result<()> {
    info!("=== Multi-VM 1 account data start migration === ");

    let db_storage = DBStorage::open_with_cfs(
        db_path,
        StorageVersion::V3.get_column_family_names().to_vec(),
        true,
        Default::default(),
        None,
    )?;
    info!("Database opened successfully");

    let storage_instance = StorageInstance::new_db_instance(db_storage);
    let block_storage = BlockInnerStorage::new(storage_instance.clone());

    info!("Fetching block {} from storage...", block_hash);
    let block = block_storage
        .get(block_hash)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;
    info!("Block found successfully");

    let storage = Arc::new(Storage::new(storage_instance)?);
    let old_statedb = ChainStateDB::new(storage, Some(block.header.state_root()));

    migrate_accounts_data_from_statedb(target_statedb, &old_statedb)?;

    info!("=== Multi-VM 1 account data finished migration === ");
    Ok(())
}

pub fn migrate_accounts_data_from_statedb(
    target_statedb: &ChainStateDB,
    source_statedb: &ChainStateDB,
) -> anyhow::Result<()> {
    let global_states = source_statedb.dump_iter()?;
    let mut state_set_data = Vec::new();
    for (account_address, account_state_set) in global_states {
        state_set_data.push((account_address, account_state_set))
    }
    target_statedb.apply(ChainStateSet::new(state_set_data))?;
    Ok(())
}
