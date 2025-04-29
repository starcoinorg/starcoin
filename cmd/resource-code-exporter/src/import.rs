// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::{
    account_address::AccountAddress,
    state_set::{AccountStateSet, ChainStateSet},
};
use std::path::Path;
use std::sync::Arc;

pub fn import(csv_path: &Path, db_path: &Path) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        db_path,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        false,
        Default::default(),
        None,
    )?;
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage))?;
    let storage = Arc::new(storage);
    let statedb = ChainStateDB::new(storage.clone(), None);
    import_from_statedb(&statedb, csv_path)
}

/// Import resources and code from CSV file to a new statedb
pub fn import_from_statedb(statedb: &ChainStateDB, csv_path: &Path) -> anyhow::Result<()> {
    // Read CSV file
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let mut expected_state_root = None;
    let mut state_sets = Vec::new();

    for result in csv_reader.records() {
        let record = result?;
        let address: AccountAddress = serde_json::from_str(&record[0])?;
        let state_root: HashValue = serde_json::from_str(&record[1])?;
        let account_state: AccountStateSet = serde_json::from_str(&record[2])?;

        // Store the first state root as expected
        if expected_state_root.is_none() {
            expected_state_root = Some(state_root);
        }

        // Add to state sets
        state_sets.push((address, account_state));
    }

    // Create chain state set and apply it
    let chain_state_set = ChainStateSet::new(state_sets);
    statedb.apply(chain_state_set)?;

    // Get new state root
    let new_state_root = statedb.state_root();

    // Verify state root matches
    if let Some(expected) = expected_state_root {
        assert_eq!(
            new_state_root, expected,
            "Imported state root does not match expected state root"
        );
        println!("Import successful! State root: {}", new_state_root);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::export::export_from_statedb;
    use starcoin_storage::db_storage::DBStorage;
    use std::fs::create_dir_all;
    use std::sync::Arc;
    use tempfile::TempDir;
    use test_helper::executor::prepare_genesis;

    #[test]
    fn test_import_from_csv() -> anyhow::Result<()> {
        //////////////////////////////////////////////////////
        // Step 1: Do Export
        // Initialize test storage with genesis
        let (export_chain_statedb, _net) = prepare_genesis();
        let export_state_root = export_chain_statedb.state_root();

        // Create a temporary directory for test files
        let temp_dir = TempDir::new()?;
        let export_path = temp_dir.path().join("export.csv");
        // Export data
        {
            let mut csv_writer = csv::WriterBuilder::new().from_path(&export_path)?;
            export_from_statedb(&export_chain_statedb, export_state_root, &mut csv_writer)?;
        }

        //////////////////////////////////////////////////////
        // Step 2: Do Import
        let import_db_path = temp_dir.path().join("import_db");
        if !import_db_path.exists() {
            create_dir_all(&import_db_path)?;
        }
        // Create new statedb from imported data
        let db_storage = DBStorage::open_with_cfs(
            &import_db_path,
            StorageVersion::current_version()
                .get_column_family_names()
                .to_vec(),
            false,
            Default::default(),
            None,
        )?;
        let storage = Storage::new(StorageInstance::new_db_instance(db_storage))?;
        let storage = Arc::new(storage);
        let imported_statedb = ChainStateDB::new(storage.clone(), None);
        import_from_statedb(&imported_statedb, &export_path)?;

        // Verify state root matches
        assert_eq!(
            imported_statedb.state_root(),
            export_state_root,
            "Imported state root does not match genesis state root"
        );

        Ok(())
    }
}
