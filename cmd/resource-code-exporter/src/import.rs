// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::state_set::ChainStateSet;

use std::{path::Path, sync::Arc};

pub fn import(bcs_path: &Path, db_path: &Path, expect_root_hash: HashValue) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        db_path,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        false,
        Default::default(),
        None,
    )?;
    let statedb = ChainStateDB::new(
        Arc::new(Storage::new(StorageInstance::new_db_instance(db_storage))?),
        None,
    );
    import_from_statedb(&statedb, bcs_path, expect_root_hash, false)
}

/// Import ChainStateSet from BCS file to a new statedb
pub fn import_from_statedb(
    statedb: &ChainStateDB,
    bcs_path: &Path,
    expect_state_root_hash: HashValue,
    check_state_root: bool,
) -> anyhow::Result<()> {
    info!("Starting import_from_statedb from: {}", bcs_path.display());

    // Read BCS file
    info!("Reading BCS file...");
    let bcs_data = std::fs::read(bcs_path)?;
    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

    info!(
        "Loaded {} account states from BCS file",
        chain_state_set.len()
    );

    // Apply the state set to statedb
    info!("Applying state sets to statedb...");

    for (address, account_state_set) in chain_state_set.state_sets() {
        info!("Processing account: {}", address);

        // Handle resource set
        if let Some(resource_set) = account_state_set.resource_set() {
            for (key, value) in resource_set.iter() {
                let access_path = starcoin_vm_types::access_path::AccessPath::new(
                    *address,
                    starcoin_vm_types::access_path::DataPath::Resource(bcs_ext::from_bytes::<
                        starcoin_vm_types::language_storage::StructTag,
                    >(key)?),
                );
                statedb.set(&access_path, value.clone())?;
            }
        }

        // Handle code set
        if let Some(code_set) = account_state_set.code_set() {
            for (key, value) in code_set.iter() {
                let access_path = starcoin_vm_types::access_path::AccessPath::new(
                    *address,
                    starcoin_vm_types::access_path::DataPath::Code(bcs_ext::from_bytes::<
                        starcoin_vm_types::access_path::ModuleName,
                    >(key)?),
                );
                statedb.set(&access_path, value.clone())?;
            }
        }
    }

    // Commit and flush
    info!("Committing changes...");
    let new_state_root = statedb.commit()?;
    statedb.flush()?;

    info!("Import completed. New state root: {}", new_state_root);

    // Verify state root matches if requested
    if check_state_root {
        assert_eq!(
            expect_state_root_hash, new_state_root,
            "Imported state root does not match expected state root"
        );
        info!("State root verification successful!");
    }

    Ok(())
}
