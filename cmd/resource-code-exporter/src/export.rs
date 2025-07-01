// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    block::legacy::BlockInnerStorage, db_storage::DBStorage, storage::CodecKVStore,
    storage::StorageInstance, Storage, StorageVersion,
};
use std::fs::File;
use std::{io::Write, path::Path, sync::Arc};

/// Export resources and code from storage for a specific block
pub fn export(db: &str, output: &Path, block_hash: HashValue) -> anyhow::Result<()> {
    info!("Starting export process for block: {}", block_hash);
    info!("Opening database at: {}", db);
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::V3.get_column_family_names().to_vec(),
        true,
        Default::default(),
        None,
    )?;

    info!("Initializing storage...");
    let storage_instance = StorageInstance::new_db_instance(db_storage);
    let block_storage = BlockInnerStorage::new(storage_instance.clone());

    info!("Fetching block {} from storage...", block_hash);
    let block = block_storage
        .get(block_hash)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;
    info!("Block found successfully");

    let root = block.header.state_root();
    info!("State root: {}", root);
    info!("Initializing ChainStateDB...");
    let storage = Arc::new(Storage::new(storage_instance)?);
    let statedb = ChainStateDB::new(storage, Some(root));

    info!("Starting export from StateDB to: {}", output.display());
    export_from_statedb(&statedb, output)?;

    info!("Export completed successfully");

    Ok(())
}

/// Export ChainStateSet as BCS format to specified path
pub fn export_from_statedb(statedb: &ChainStateDB, bcs_output_path: &Path) -> anyhow::Result<()> {
    info!(
        "Starting export_from_statedb to: {}",
        bcs_output_path.display()
    );

    info!("Dumping global states from StateDB...");
    let dump_state = statedb.dump()?;

    // Write dump state as bcs format to file
    info!(
        "Writing dump state to BCS file: {}",
        bcs_output_path.display()
    );
    let bcs_bytes = bcs_ext::to_bytes(&dump_state)?;
    let mut bcs_file = File::create(bcs_output_path)?;
    bcs_file.write_all(&bcs_bytes)?;
    info!("Successfully wrote {} bytes to BCS file", bcs_bytes.len());

    info!("BCS export completed successfully");
    Ok(())
}
