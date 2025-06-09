// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::state_set::StateSet;
use starcoin_types::{
    account_address::AccountAddress,
    state_set::{AccountStateSet, ChainStateSet},
};
use std::path::Path;
use std::sync::Arc;

pub fn import(
    csv_path: &Path,
    db_path: &Path,
    expect_root_hash: HashValue,
    start: u64,
    end: u64,
) -> anyhow::Result<()> {
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
    import_from_statedb(&statedb, csv_path, expect_root_hash, start, end)
}

/// Import resources and code from CSV file to a new statedb
pub fn import_from_statedb(
    statedb: &ChainStateDB,
    csv_path: &Path,
    expect_state_root_hash: HashValue,
    start: u64,
    end: u64,
) -> anyhow::Result<()> {
    // Read CSV file
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let mut chain_state_set_data = Vec::new();

    for result in csv_reader.records() {
        let record = result?;
        let account_address: AccountAddress = serde_json::from_str(&record[0])?;
        assert_eq!(record.len(), 5);
        println!("record len: {:?}", record.len());

        let code_state_set = if !record[1].is_empty() && !record[2].is_empty() {
            let code_state_hash = &record[1];
            let code_state_set_str = &record[2];
            assert_eq!(
                code_state_hash,
                HashValue::sha3_256_of(code_state_set_str.as_bytes()).to_hex_literal()
            );
            Some(serde_json::from_str::<StateSet>(code_state_set_str)?)
        } else {
            None
        };

        let resource_state_set = if !record[3].is_empty() && !record[4].is_empty() {
            let resource_blob_hash = &record[3];
            let resource_state_set_str = &record[4];
            assert_eq!(
                resource_blob_hash,
                HashValue::sha3_256_of(resource_state_set_str.as_bytes()).to_hex_literal()
            );
            Some(serde_json::from_str::<StateSet>(resource_state_set_str)?)
        } else {
            None
        };

        chain_state_set_data.push((
            account_address,
            AccountStateSet::new(vec![code_state_set, resource_state_set]),
        ));
    }

    statedb.apply(ChainStateSet::new(chain_state_set_data))?;

    // Get new state root
    let new_state_root = statedb.state_root();

    // Verify state root matches
    {
        assert_eq!(
            expect_state_root_hash, new_state_root,
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
            export_from_statedb(&export_chain_statedb, &mut csv_writer, 0, 0)?;
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
        let imported_statedb = ChainStateDB::new(
            Arc::new(Storage::new(StorageInstance::new_db_instance(db_storage))?),
            None,
        );
        import_from_statedb(&imported_statedb, &export_path, export_state_root, 0, 0)?;

        Ok(())
    }
}
