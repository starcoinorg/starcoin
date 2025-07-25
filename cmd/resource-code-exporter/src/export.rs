// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::storage::{ColumnFamilyName, InnerStore};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage};
use starcoin_types::block::legacy::Block;
use std::fs::File;
use std::{io::Write, path::Path, sync::Arc};

const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";

/// Export resources and code from storage for a specific block
pub fn export(
    db: &str,
    output: &Path,
    block_hash: HashValue,
    white_list: Option<Vec<AccountAddress>>,
) -> anyhow::Result<()> {
    info!("Starting export process for block: {}", block_hash);
    info!("Opening database at: {}", db);
    let db_storage = DBStorage::open_with_cfs(
        db,
        vec![BLOCK_PREFIX_NAME, STATE_NODE_PREFIX_NAME],
        true,
        Default::default(),
        None,
    )?;

    info!("Initializing storage...");
    let storage_instance = StorageInstance::new_db_instance(db_storage);

    info!("Fetching block {} from storage...", block_hash);
    let block = storage_instance
        .get(BLOCK_PREFIX_NAME, block_hash.to_vec())?
        .map(|data| Block::decode(&data))
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;
    info!("Block found successfully");

    let root = block.header.state_root();
    info!("State root: {}", root);
    info!("Initializing ChainStateDB...");
    let storage = Arc::new(Storage::new(storage_instance)?);
    let statedb = ChainStateDB::new(storage, Some(root));

    info!("Starting export from StateDB to: {}", output.display());
    export_from_statedb(&statedb, output, white_list)?;

    info!("Export completed successfully");

    Ok(())
}

/// Export ChainStateSet as BCS format to specified path
pub fn export_from_statedb(
    statedb: &ChainStateDB,
    bcs_output_path: &Path,
    white_list: Option<Vec<AccountAddress>>,
) -> anyhow::Result<()> {
    info!(
        "Starting export_from_statedb to: {}",
        bcs_output_path.display()
    );

    info!("Dumping global states from StateDB...");

    let mut filtered_account_states = vec![];

    if let Some(white_list) = white_list {
        info!("Using whitelist with {} accounts", white_list.len());
        for address in white_list {
            if let Some(account_state_set) = statedb.get_account_state_set(&address)? {
                let code_count = account_state_set.code_set().map(|s| s.len()).unwrap_or(0);
                let resource_code = account_state_set
                    .resource_set()
                    .map(|s| s.len())
                    .unwrap_or(0);
                info!(
                    "Exporting: account {:?}, Code count: {}, Resource count: {:?}",
                    address, code_count, resource_code
                );
                filtered_account_states.push((address, account_state_set));
                info!("Added account {} to export, ", address);
            } else {
                info!("Account {} not found in state, skipping", address);
            }
        }
    } else {
        info!("No whitelist provided, exporting all accounts");
        let global_states_iter = statedb.dump_iter()?;
        for (account_address, account_state_set) in global_states_iter {
            let code_count = account_state_set.code_set().map(|s| s.len()).unwrap_or(0);
            let resource_code = account_state_set
                .resource_set()
                .map(|s| s.len())
                .unwrap_or(0);
            info!(
                "Exporting: account {:?}, Code count: {}, Resource count: {:?}",
                account_address, code_count, resource_code
            );
            filtered_account_states.push((account_address, account_state_set));
        }
    }

    let dump_state = ChainStateSet::new(filtered_account_states);

    // Write dump state as bcs format to file
    info!(
        "Filtered {} accounts for export, and writing dump state to BCS file: {}",
        dump_state.len(),
        bcs_output_path.display()
    );

    let bcs_bytes = bcs_ext::to_bytes(&dump_state)?;
    let mut bcs_file = File::create(bcs_output_path)?;
    bcs_file.write_all(&bcs_bytes)?;
    info!(
        "BCS export completed successfully, wrote {} bytes to BCS file",
        bcs_bytes.len()
    );

    Ok(())
}
