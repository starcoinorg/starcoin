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
pub fn export(
    db: &str,
    output: &Path,
    block_hash: HashValue,
    _start: u64,
    _end: u64,
) -> anyhow::Result<()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use starcoin_types::{language_storage::StructTag, state_set::ChainStateSet};
    use starcoin_vm_types::account_config::{
        association_address, token_code::TokenCode, BalanceResource, STC_TOKEN_CODE_STR,
    };
    use std::str::FromStr;
    use test_helper::executor::prepare_genesis;

    #[test]
    fn test_export_from_statedb() -> anyhow::Result<()> {
        // Initialize logger for test
        starcoin_logger::init_for_test();

        // Initialize test storage with genesis
        let (chain_statedb, _net) = prepare_genesis();

        // Test BCS export
        let test_bcs_path = Path::new("test_dump_state.bcs");
        export_from_statedb(&chain_statedb, test_bcs_path)?;

        // Verify the BCS file was created and contains data
        assert!(test_bcs_path.exists(), "BCS file should be created");
        let file_size = std::fs::metadata(test_bcs_path)?.len();
        assert!(file_size > 0, "BCS file should not be empty");

        // Read back the BCS file and verify data integrity
        info!("Reading back BCS file for verification...");
        let bcs_data = std::fs::read(test_bcs_path)?;
        let deserialized_state: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;

        // Verify that the deserialized state contains data
        assert!(
            !deserialized_state.is_empty(),
            "Deserialized state should not be empty"
        );
        info!(
            "Successfully deserialized {} account states",
            deserialized_state.len()
        );

        // Check if association account exists and has balance
        let association_addr = association_address();
        let mut found_association = false;
        let mut association_balance = None;

        for (address, account_state_set) in deserialized_state.state_sets() {
            if *address != association_addr {
                continue;
            }

            found_association = true;
            info!("Found association account in exported state");

            let stc_balance_resource = BalanceResource::struct_tag_for_token(
                TokenCode::from_str(STC_TOKEN_CODE_STR)?.try_into()?,
            );

            // Check if association account has resource data
            if let Some(resource_set) = account_state_set.resource_set() {
                info!("Association account has {} resources", resource_set.len());

                // Look for balance resource in the resource set
                for (key, value) in resource_set.iter() {
                    // The balance resource key typically contains "Balance" in the path
                    let struct_tag: StructTag = bcs_ext::from_bytes::<StructTag>(key)?;

                    if struct_tag == stc_balance_resource {
                        info!("Found balance resource for association account");
                        // Try to deserialize as BalanceResource
                        match bcs_ext::from_bytes::<BalanceResource>(value) {
                            Ok(balance_resource) => {
                                association_balance = Some(balance_resource.token());
                                info!("Association account balance: {}", balance_resource.token());
                                break;
                            }
                            Err(e) => {
                                info!("Failed to deserialize balance resource: {}", e);
                            }
                        }
                    }
                }
            }
            break;
        }

        assert!(
            found_association,
            "Association account should exist in exported state"
        );
        if let Some(balance) = association_balance {
            assert!(
                balance > 0,
                "Association account should have positive balance, got: {}",
                balance
            );
        } else {
            info!("Could not verify association account balance, but account exists");
        }

        // Clean up test file
        std::fs::remove_file(test_bcs_path)?;

        Ok(())
    }
}
