// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::{
    account_address::AccountAddress,
    state_set::{AccountStateSet, ChainStateSet, StateSet},
};
use std::{path::Path, sync::Arc};

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
    import_from_statedb(&statedb, csv_path, expect_root_hash, start, end, true)
}

/// Import resources and code from CSV file to a new statedb
pub fn import_from_statedb(
    statedb: &ChainStateDB,
    csv_path: &Path,
    expect_state_root_hash: HashValue,
    start: u64,
    end: u64,
    check_state_root: bool,
) -> anyhow::Result<()> {
    println!(
        "Starting import_from_statedb...， start: {}, end: {}",
        start, end
    );

    // Read CSV file
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let mut chain_state_set_data = Vec::new();
    let mut processed = 0;

    for result in csv_reader.records() {
        // Skip records before start index
        if processed < start {
            processed += 1;
            continue;
        }
        // Stop processing after end index
        if processed >= end && end > 0 {
            break;
        }

        let record = result?;
        let account_address: AccountAddress = serde_json::from_str(&record[0])?;
        assert_eq!(record.len(), 5);
        println!(
            "Processing record {}: account {}",
            processed, account_address
        );

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
        processed += 1;

        if processed % 100 == 0 {
            println!("Progress: {} records processed", processed);
        }
    }

    println!(
        "Applying {} state sets to statedb...",
        chain_state_set_data.len()
    );
    statedb.apply(ChainStateSet::new(chain_state_set_data))?;
    statedb.commit()?;
    statedb.flush()?;

    // Get new state root
    let new_state_root = statedb.state_root();

    // Verify state root matches
    if check_state_root {
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
    use starcoin_config::ChainNetwork;
    use starcoin_genesis::Genesis;
    use starcoin_storage::db_storage::DBStorage;
    use starcoin_types::{
        account::Account,
        account_config::{association_address, core_code_address, stc_type_tag},
        identifier::Identifier,
        language_storage::ModuleId,
        transaction::{ScriptFunction, TransactionPayload},
    };
    use std::fs::create_dir_all;
    use std::sync::Arc;
    use tempfile::TempDir;
    use test_helper::executor::{association_execute_should_success, get_balance, prepare_genesis};

    /// Create a ChainStateDB with real storage from a test directory
    /// This function creates a temporary directory and initializes a real storage backend
    /// instead of using mock storage, which is useful for testing export/import functionality
    fn create_test_statedb_with_genesis() -> anyhow::Result<(ChainStateDB, ChainNetwork, TempDir)> {
        create_test_statedb_with_genesis_custom(None)
    }

    /// Create a ChainStateDB with real storage from a test directory with custom options
    ///
    /// # Arguments
    /// * `db_name` - Optional custom name for the database directory (default: "test_db")
    ///
    /// # Returns
    /// * `ChainStateDB` - The initialized state database
    /// * `ChainNetwork` - The test network configuration
    /// * `TempDir` - The temporary directory containing the database (will be auto-cleaned)
    fn create_test_statedb_with_genesis_custom(
        db_name: Option<&str>,
    ) -> anyhow::Result<(ChainStateDB, ChainNetwork, TempDir)> {
        let temp_dir = TempDir::new()?;
        let db_name = db_name.unwrap_or("test_db");
        let test_db_path = temp_dir.path().join(db_name);
        if !test_db_path.exists() {
            create_dir_all(&test_db_path)?;
        }

        // Create real storage and statedb
        let db_storage = DBStorage::open_with_cfs(
            &test_db_path,
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

        // Build and execute genesis
        let net = ChainNetwork::new_test();
        let genesis_txn = Genesis::build_genesis_transaction(&net)?;
        Genesis::execute_genesis_txn(&statedb, genesis_txn)?;

        Ok((statedb, net, temp_dir))
    }

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
            export_from_statedb(&export_chain_statedb, &mut csv_writer, 0, u64::MAX, None)?;
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
        import_from_statedb(
            &imported_statedb,
            &export_path,
            export_state_root,
            0,
            u64::MAX,
            true,
        )?;

        Ok(())
    }

    #[test]
    fn test_import_with_transfer() -> anyhow::Result<()> {
        //////////////////////////////////////////////////////
        // Step 1: Initialize test storage with genesis using real storage
        let (export_chain_statedb, net, temp_dir) = create_test_statedb_with_genesis()?;

        // Create a random account
        let random_account = Account::new();
        println!("Created random account: {}", random_account.address());

        // Transfer 1 STC from association to random account
        let transfer_amount = 1_000_000_000; // 1 STC in nano units
        let transfer_script = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("TransferScripts").unwrap(),
            ),
            Identifier::new("peer_to_peer_v2").unwrap(),
            vec![stc_type_tag()],
            vec![
                bcs_ext::to_bytes(random_account.address()).unwrap(),
                bcs_ext::to_bytes(&transfer_amount).unwrap(),
            ],
        );

        // Execute transfer transaction
        association_execute_should_success(
            &net,
            &export_chain_statedb,
            TransactionPayload::ScriptFunction(transfer_script),
        )?;
        // commit/flush to persist state
        export_chain_statedb.commit()?;
        export_chain_statedb.flush()?;

        // Verify the transfer was successful
        let balance_after_transfer = get_balance(*random_account.address(), &export_chain_statedb);
        println!(
            "Balance after transfer: {} nano STC",
            balance_after_transfer
        );
        assert_eq!(
            balance_after_transfer, transfer_amount,
            "Transfer should be successful"
        );

        let export_state_root = export_chain_statedb.state_root();

        //////////////////////////////////////////////////////
        // Step 2: Export data
        let export_path = temp_dir.path().join("export_with_transfer.csv");
        {
            let mut csv_writer = csv::WriterBuilder::new().from_path(&export_path)?;
            export_from_statedb(&export_chain_statedb, &mut csv_writer, 0, u64::MAX, None)?;
        }

        //////////////////////////////////////////////////////
        // Step 3: Import data
        let import_db_path = temp_dir.path().join("import_db_with_transfer");
        if !import_db_path.exists() {
            create_dir_all(&import_db_path)?;
        }
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
        import_from_statedb(
            &imported_statedb,
            &export_path,
            export_state_root,
            0,
            u64::MAX,
            false,
        )?;

        //////////////////////////////////////////////////////
        // Step 4: Verify the imported state
        let balance_after_import = get_balance(*random_account.address(), &imported_statedb);
        println!("Balance after import: {} nano STC", balance_after_import);
        assert_eq!(
            balance_after_import, transfer_amount,
            "Balance should be preserved after import"
        );

        // Also verify association account balance is reduced
        let association_balance_after_import =
            get_balance(association_address(), &imported_statedb);
        println!(
            "Association balance after import: {} nano STC",
            association_balance_after_import
        );
        assert!(
            association_balance_after_import < 1_000_000_000_000_000_000,
            "Association balance should be reduced after transfer"
        );

        // temp_dir will be automatically cleaned up when it goes out of scope
        Ok(())
    }

    #[test]
    fn test_create_test_statedb_helper() -> anyhow::Result<()> {
        // Example of using the helper function with default settings
        let (statedb, _net, _temp_dir) = create_test_statedb_with_genesis()?;

        // Verify that genesis was executed properly
        let association_balance = get_balance(association_address(), &statedb);
        println!(
            "Association balance after genesis: {} nano STC",
            association_balance
        );
        assert!(
            association_balance > 0,
            "Association should have balance after genesis"
        );

        // Example of using the helper function with custom database name
        let (statedb2, _net2, _temp_dir2) =
            create_test_statedb_with_genesis_custom(Some("custom_db"))?;

        // Verify that both databases work independently
        let balance1 = get_balance(association_address(), &statedb);
        let balance2 = get_balance(association_address(), &statedb2);
        assert_eq!(
            balance1, balance2,
            "Both databases should have same genesis state"
        );

        // temp_dir and temp_dir2 will be automatically cleaned up when they go out of scope
        Ok(())
    }
}
