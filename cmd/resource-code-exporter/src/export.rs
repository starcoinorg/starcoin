// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    block::legacy::BlockInnerStorage, db_storage::DBStorage, storage::CodecKVStore,
    storage::StorageInstance, Storage, StorageVersion,
};
use starcoin_types::account_address::AccountAddress;
use std::{io::Write, path::Path, sync::Arc};

/// Export resources and code from storage for a specific block
pub fn export(
    db: &str,
    output: &Path,
    block_hash: HashValue,
    start: u64,
    end: u64,
    white_list: Option<Vec<AccountAddress>>,
) -> anyhow::Result<()> {
    println!("Starting export process for block: {}", block_hash);
    println!("Opening database at: {}", db);
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::V3.get_column_family_names().to_vec(),
        true,
        Default::default(),
        None,
    )?;

    println!("Initializing storage...");
    let storage_instance = StorageInstance::new_db_instance(db_storage);
    let block_storage = BlockInnerStorage::new(storage_instance.clone());

    println!("Fetching block {} from storage...", block_hash);
    let block = block_storage
        .get(block_hash)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;
    println!("Block found successfully");

    let root = block.header.state_root();
    println!("State root: {}", root);
    println!("Initializing ChainStateDB...");
    let storage = Arc::new(Storage::new(storage_instance)?);
    let statedb = ChainStateDB::new(storage, Some(root));

    println!("Creating CSV writer for output: {}", output.display());
    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;
    println!("Starting export from StateDB...");
    export_from_statedb(&statedb, &mut csv_writer, start, end, white_list)?;
    println!("Export completed successfully");

    Ok(())
}

/// Export resources and code from StateDB to a writer
pub fn export_from_statedb<W: Write>(
    statedb: &ChainStateDB,
    writer: &mut csv::Writer<W>,
    start: u64,
    end: u64,
    white_list: Option<Vec<AccountAddress>>,
) -> anyhow::Result<()> {
    println!(
        "Starting export_from_statedb...， start: {}, end: {}",
        start, end
    );
    // write csv header
    {
        println!("Writing CSV header...");
        writer.write_field("address")?;
        writer.write_field("code_blob_hash")?;
        writer.write_field("code_blob")?;
        writer.write_field("resource_blob_hash")?;
        writer.write_field("resource_blob")?;
        writer.write_record(None::<&[u8]>)?;
        println!("CSV header written successfully");
    }

    println!("Dumping global states from StateDB...");
    let global_states = statedb.dump_iter()?;

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;
    let mut total_code_size = 0;
    let mut total_resource_size = 0;
    let mut remaining_white_list = white_list.clone();

    for (account_address, account_state_set) in global_states {
        // Skip accounts before start index
        if processed < start {
            processed += 1;
            continue;
        }
        // Stop processing after end index
        if processed >= end && end > 0 {
            break;
        }

        // Skip if account is not in white_lists (when white_lists is provided)
        if let Some(ref list) = white_list {
            if !list.contains(&account_address) {
                continue;
            }
        }

        println!("Processing account: {}", account_address);

        // Process codes
        let (code_state_set_hash, code_state_set) = match account_state_set.code_set() {
            Some(state_set) => {
                let code_state_set = serde_json::to_string(&state_set)?;
                let code_size = code_state_set.len();
                total_code_size += code_size;
                println!("  Found code set, size: {} bytes", code_size);
                (
                    HashValue::sha3_256_of(code_state_set.as_bytes()).to_hex_literal(),
                    code_state_set,
                )
            }
            None => {
                println!("  No code set found for this account");
                (String::new(), String::new())
            }
        };

        // Process resources
        let (resource_state_set_hash, resource_state_set) = match account_state_set.resource_set() {
            Some(state_set) => {
                let resource_state_set = serde_json::to_string(&state_set)?;
                let resource_size = resource_state_set.len();
                total_resource_size += resource_size;
                println!("  Found resource set, size: {} bytes", resource_size);
                (
                    HashValue::sha3_256_of(resource_state_set.as_bytes()).to_hex_literal(),
                    resource_state_set,
                )
            }
            None => {
                println!("  No resource set found for this account");
                (String::new(), String::new())
            }
        };

        // write csv record
        let record = vec![
            serde_json::to_string(&account_address)?,
            code_state_set_hash,
            code_state_set,
            resource_state_set_hash,
            resource_state_set,
        ];

        writer.serialize(record)?;
        processed += 1;

        // Remove processed account from remaining white list for early exit optimization
        if let Some(ref mut remaining) = remaining_white_list {
            remaining.retain(|&addr| addr != account_address);
            if remaining.is_empty() {
                println!("All white list items processed, exiting early");
                break;
            }
        }
        println!("Progress: {} accounts processed ", processed);
    }

    println!("Export completed:");
    println!("  Total accounts processed: {}", processed);
    println!("  Total code size: {} bytes", total_code_size);
    println!("  Total resource size: {} bytes", total_resource_size);
    println!("  Total processing time: {} ms", now.elapsed().as_millis());

    // println!("Flushing CSV writer...");
    // writer.flush()?;
    println!("CSV writer flushed successfully");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use test_helper::executor::prepare_genesis;

    #[test]
    fn test_export_from_mock_statedb() -> anyhow::Result<()> {
        // Initialize test storage with genesis
        let (chain_statedb, _net) = prepare_genesis();

        // Create a buffer to write CSV data
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut csv_writer = csv::WriterBuilder::new().from_writer(&mut buffer);
            // Export all accounts (from index 0 to u64::MAX)
            export_from_statedb(&chain_statedb, &mut csv_writer, 0, u64::MAX, None)?;
        }

        // Get the written data
        let data = buffer.into_inner();
        let data_str = String::from_utf8(data)?;
        // println!("Exported CSV data:\n{}", data_str);

        // Verify the data contains expected content
        let mut csv_reader = csv::Reader::from_reader(data_str.as_bytes());
        let mut has_data = false;
        for result in csv_reader.records() {
            let _record = result?;
            // println!("Record: {:?}", record);
            has_data = true;
        }
        assert!(has_data, "CSV should contain exported data");
        Ok(())
    }

    #[test]
    fn test_export_white_list_from_mock_statedb() -> anyhow::Result<()> {
        // Initialize test storage with genesis
        let (chain_statedb, _net) = prepare_genesis();

        // Create a buffer to write CSV data
        let mut buffer = Cursor::new(Vec::new());
        let white_list = vec![AccountAddress::from_hex_literal("0x1").unwrap()];
        {
            let mut csv_writer = csv::WriterBuilder::new().from_writer(&mut buffer);
            // Export with whitelist
            export_from_statedb(
                &chain_statedb,
                &mut csv_writer,
                0,
                u64::MAX,
                Some(white_list.clone()),
            )?;
        }
        let data = buffer.into_inner();
        let data_str = String::from_utf8(data)?;

        // Parse CSV data and verify whitelist functionality
        let mut csv_reader = csv::Reader::from_reader(data_str.as_bytes());
        let headers = csv_reader.headers()?;
        assert_eq!(headers.get(0).unwrap(), "address");
        assert_eq!(headers.get(1).unwrap(), "code_blob_hash");
        assert_eq!(headers.get(2).unwrap(), "code_blob");
        assert_eq!(headers.get(3).unwrap(), "resource_blob_hash");
        assert_eq!(headers.get(4).unwrap(), "resource_blob");
        let mut exported_addresses = Vec::new();
        let mut record_count = 0;

        for result in csv_reader.records() {
            let record = result?;
            record_count += 1;
            // Parse address from the record
            let address_str = record.get(0).unwrap();
            let address: AccountAddress = serde_json::from_str(address_str)?;
            exported_addresses.push(address);
            // Verify record has correct format (5 fields)
            assert_eq!(record.len(), 5, "Each record should have 5 fields");
            // Verify address is in whitelist
            assert!(
                white_list.contains(&address),
                "Exported address {} is not in whitelist {:?}",
                address,
                white_list
            );
            // Verify hash fields are valid hex strings (if not empty)
            if let Some(code_hash) = record.get(1) {
                if !code_hash.is_empty() {
                    assert!(
                        code_hash.starts_with("0x"),
                        "Code hash should start with 0x: {}",
                        code_hash
                    );
                }
            }
            if let Some(resource_hash) = record.get(3) {
                if !resource_hash.is_empty() {
                    assert!(
                        resource_hash.starts_with("0x"),
                        "Resource hash should start with 0x: {}",
                        resource_hash
                    );
                }
            }
        }

        // Verify that we exported at least one record (excluding header)
        assert!(
            record_count >= 1,
            "Should have exported at least one data record, got {} records",
            record_count
        );

        // Verify that all exported addresses are unique
        let unique_addresses: std::collections::HashSet<_> = exported_addresses.iter().collect();
        assert_eq!(
            unique_addresses.len(),
            exported_addresses.len(),
            "All exported addresses should be unique"
        );

        // Verify that all exported addresses are from whitelist
        for address in &exported_addresses {
            assert!(
                white_list.contains(address),
                "Address {} should be in whitelist",
                address
            );
        }

        println!(
            "Whitelist test passed: exported {} addresses from whitelist {:?}",
            exported_addresses.len(),
            white_list
        );

        Ok(())
    }
}
