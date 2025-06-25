// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_storage::{db_storage::DBStorage, storage::StorageInstance, Storage, StorageVersion};
use starcoin_types::state_set::ChainStateSet;
use std::{path::Path, sync::Arc};
use bcs_ext;

pub fn import(
    bcs_path: &Path,
    db_path: &Path,
    expect_root_hash: HashValue,
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
    statedb.apply(chain_state_set)?;
    statedb.commit()?;
    statedb.flush()?;

    // Get new state root
    let new_state_root = statedb.state_root();
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::export::export_from_statedb;
    use starcoin_storage::db_storage::DBStorage;
    use starcoin_transaction_builder::encode_transfer_script_function;
    use starcoin_types::{
        account_address::AccountAddress,
        transaction::{TransactionPayload, TransactionStatus},
        vm_error::KeptVMStatus,
    };
    use starcoin_vm_types::state_view::StateReaderExt;
    use std::fs::create_dir_all;
    use std::sync::Arc;
    use tempfile::TempDir;
    use test_helper::executor::{association_execute_should_success, prepare_genesis};

    #[test]
    fn test_import_from_bcs() -> anyhow::Result<()> {
        starcoin_logger::init_for_test();

        //////////////////////////////////////////////////////
        // Step 1: Do Export
        // Initialize test storage with genesis
        let (export_chain_statedb, net) = prepare_genesis();

        let recipient = AccountAddress::random();
        let transfer_amount = 1000000000;
        let transaction_output = association_execute_should_success(
            &net,
            &export_chain_statedb,
            TransactionPayload::ScriptFunction(encode_transfer_script_function(
                recipient,
                transfer_amount,
            )),
        )?;

        assert_eq!(
            *transaction_output.status(),
            TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        let after_transfer = export_chain_statedb.get_balance(recipient)?.unwrap();
        assert_eq!(after_transfer, transfer_amount);

        let export_state_root = export_chain_statedb.state_root();

        // Create a temporary directory for test files
        let temp_dir = TempDir::new()?;
        let export_path = temp_dir.path().join("export.bcs");
        // Export data
        export_from_statedb(&export_chain_statedb, &export_path)?;

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
            true, // Check state root
        )?;

        // Verify that the imported balance matches the original
        let imported_balance = imported_statedb
            .get_balance(recipient)
            .expect("read balance resource should ok")
            .unwrap_or_default();
        assert_eq!(imported_balance, transfer_amount);
        
        info!("Import test successful! Recipient balance: {}", imported_balance);

        Ok(())
    }
}
