// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
use std::{io::Write, path::Path, sync::Arc};

/// Export resources and code from storage for a specific block
pub fn export(db: &str, output: &Path, block_id: HashValue) -> anyhow::Result<()> {
    let db_storage = DBStorage::open_with_cfs(
        db,
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let storage = Storage::new(StorageInstance::new_db_instance(db_storage))?;
    let storage = Arc::new(storage);
    let block = storage
        .get_block(block_id)?
        .ok_or_else(|| anyhow::anyhow!("block {} not exist", block_id))?;

    let root = block.header.state_root();
    let statedb = ChainStateDB::new(storage.clone(), Some(root));

    // Create writer and export
    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;
    export_from_statedb(&statedb, &mut csv_writer)?;

    Ok(())
}

/// Export resources and code from StateDB to a writer
pub fn export_from_statedb<W: Write>(
    statedb: &ChainStateDB,
    writer: &mut csv::Writer<W>,
) -> anyhow::Result<()> {
    // write csv header
    {
        writer.write_field("address")?;
        writer.write_field("code_blob_hash")?;
        writer.write_field("code_blob")?;
        writer.write_field("resource_blob_hash")?;
        writer.write_field("resrouce_blob")?;
        writer.write_record(None::<&[u8]>)?;
    }

    let global_states = statedb.dump()?;
    println!("Total accounts to process: {}", global_states.len());

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;

    for (account_address, account_state_set) in global_states.into_iter() {
        // Process codes
        let (code_state_set_hash, code_state_set) = match account_state_set.code_set() {
            Some(state_set) => {
                let code_state_set = serde_json::to_string(&state_set)?;
                (
                    HashValue::sha3_256_of(code_state_set.as_bytes()).to_hex_literal(),
                    code_state_set,
                )
            }
            None => (String::new(), String::new()),
        };

        // Process resources
        let (resource_state_set_hash, resource_state_set) = match account_state_set.resource_set() {
            Some(state_set) => {
                let resource_state_set = serde_json::to_string(&state_set)?;
                (
                    HashValue::sha3_256_of(resource_state_set.as_bytes()).to_hex_literal(),
                    resource_state_set,
                )
            }
            None => (String::new(), String::new()),
        };

        // write csv record
        let record = vec![
            // account address
            serde_json::to_string(&account_address)?,
            code_state_set_hash,
            code_state_set,
            resource_state_set_hash,
            resource_state_set,
        ];

        writer.serialize(record)?;
        processed += 1;
        println!("Processed {}/{} accounts", processed, global_states.len());
    }

    println!("Total processing time: {} ms", now.elapsed().as_millis());
    // flush csv writer
    writer.flush()?;
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
        println!("Exported CSV data:\n{}", data_str);

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
