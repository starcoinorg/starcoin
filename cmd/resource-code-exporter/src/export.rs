// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
use std::{io::Write, path::Path, sync::Arc};

/// Export resources and code from storage for a specific block
pub fn export(db: &str, output: &Path, block_hash: HashValue) -> anyhow::Result<()> {
    println!("Starting export process for block: {}", block_hash);
    println!("Opening database at: {}", db);
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    println!("Database opened successfully");

    println!("Initializing storage...");
    let storage = Arc::new(Storage::new(StorageInstance::new_db_instance(db_storage))?);

    println!("Fetching block {} from storage...", block_hash);
    let block = storage
        .get_block(block_hash)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_hash))?;
    println!("Block found successfully");

    let root = block.header.state_root();
    println!("State root: {}", root);
    println!("Initializing ChainStateDB...");
    let statedb = ChainStateDB::new(storage, Some(root));

    println!("Creating CSV writer for output: {}", output.display());
    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;
    println!("Starting export from StateDB...");
    export_from_statedb(&statedb, &mut csv_writer)?;
    println!("Export completed successfully");

    Ok(())
}

/// Export resources and code from StateDB to a writer
pub fn export_from_statedb<W: Write>(
    statedb: &ChainStateDB,
    writer: &mut csv::Writer<W>,
) -> anyhow::Result<()> {
    println!("Starting export_from_statedb...");
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
    let global_states = statedb.dump()?;
    println!("Total accounts to process: {}", global_states.len());

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;
    let mut total_code_size = 0;
    let mut total_resource_size = 0;

    for (account_address, account_state_set) in global_states.into_iter() {
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

        if processed % 100 == 0 {
            println!(
                "Progress: {}/{} accounts processed ({}%)",
                processed,
                global_states.len(),
                (processed as f64 / global_states.len() as f64 * 100.0) as u32
            );
        }
    }

    println!("Export completed:");
    println!("  Total accounts processed: {}", processed);
    println!("  Total code size: {} bytes", total_code_size);
    println!("  Total resource size: {} bytes", total_resource_size);
    println!("  Total processing time: {} ms", now.elapsed().as_millis());

    println!("Flushing CSV writer...");
    writer.flush()?;
    println!("CSV writer flushed successfully");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use test_helper::executor::prepare_genesis;

    #[test]
    fn test_export_from_statedb() -> anyhow::Result<()> {
        // Initialize test storage with genesis
        let (chain_statedb, _net) = prepare_genesis();

        // Create a buffer to write CSV data
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut csv_writer = csv::WriterBuilder::new().from_writer(&mut buffer);
            export_from_statedb(&chain_statedb, &mut csv_writer)?;
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
}
