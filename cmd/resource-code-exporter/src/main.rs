// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use cmd_utils::move_struct_serde::MoveStruct;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
use starcoin_types::language_storage::StructTag;
use starcoin_types::state_set::AccountStateSet;
use starcoin_vm_types::account_address::AccountAddress;
use std::{
    convert::TryInto,
    fmt::Debug,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    export_from_statedb(&statedb, root, &mut csv_writer)?;

    Ok(())
}

/// Export resources and code from StateDB to a writer
pub fn export_from_statedb<W: Write>(
    statedb: &ChainStateDB,
    root: HashValue,
    writer: &mut csv::Writer<W>,
) -> anyhow::Result<()> {
    let value_annotator = MoveValueAnnotator::new(statedb);

    // write csv header
    {
        writer.write_field("address")?;
        writer.write_field("state_root")?;
        writer.write_field("resource_state_root")?;
        writer.write_field("resource_blob")?;
        writer.write_field("code_state_root")?;
        writer.write_field("code_blob")?;
        writer.write_record(None::<&[u8]>)?;
    }

    let global_states = statedb.dump()?;
    println!("Total accounts to process: {}", global_states.len());

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;

    for (account_address, account_state) in global_states.into_iter() {
        let (resource_root_hash, code_root_hash, resources, codes) =
            process_account(statedb, account_address, &account_state, &value_annotator)?;

        // write csv record
        let record = vec![
            serde_json::to_string(&account_address)?,
            serde_json::to_string(&root)?,
            resource_root_hash,
            serde_json::to_string(&resources)?,
            code_root_hash,
            serde_json::to_string(&codes)?,
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
pub fn process_account(
    statedb: &ChainStateDB,
    account: &AccountAddress,
    account_state_set: &AccountStateSet,
    value_annotator: &MoveValueAnnotator,
) -> anyhow::Result<(
    String,
    String,
    Vec<(String, serde_json::Value)>,
    Vec<(String, serde_json::Value)>,
)> {
    // Handle resource set
    let mut resources = Vec::new();
    if let Some(resource_set) = account_state_set.resource_set() {
        for (tag_bytes, data) in resource_set.iter() {
            let tag: StructTag = bcs_ext::from_bytes(tag_bytes)?;
            let annotated_struct = value_annotator.view_struct(tag.clone(), data.as_slice())?;
            let resource_json_value = serde_json::to_value(MoveStruct(annotated_struct))?;
            resources.push((tag.to_string(), resource_json_value));
        }
    }

    // Handle code set
    let mut codes = Vec::new();
    if let Some(code_set) = account_state_set.code_set() {
        for (tag_bytes, data) in code_set.iter() {
            let tag: StructTag = bcs_ext::from_bytes(tag_bytes)?;
            let annotated_struct = value_annotator.view_struct(tag.clone(), data.as_slice())?;
            let code_json_value = serde_json::to_value(MoveStruct(annotated_struct))?;
            codes.push((tag.to_string(), code_json_value));
        }
    }

    let account_state = statedb
        .get_account_state(account)?
        .ok_or_else(|| anyhow::anyhow!("account state set not found"))?;

    Ok((
        account_state.resource_root().to_hex_literal(),
        account_state.code_root().unwrap_or_default().to_string(),
        resources,
        codes,
    ))
}

#[derive(Debug, Clone, Parser)]
#[clap(
    name = "resource-code-exporter",
    about = "onchain resource and code exporter"
)]
pub struct ExporterOptions {
    #[clap(long, short = 'o', parse(from_os_str))]
    /// output file, like accounts.csv
    pub output: PathBuf,
    #[clap(long, short = 'i', parse(from_os_str))]
    /// starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    pub db_path: PathBuf,

    #[clap(long)]
    /// block id which snapshot at.
    pub block_id: HashValue,
}

fn main() -> anyhow::Result<()> {
    let option: ExporterOptions = ExporterOptions::parse();
    let output = option.output.as_path();
    let block_id = option.block_id;
    export(
        option.db_path.display().to_string().as_str(),
        output,
        block_id,
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use starcoin_config::ChainNetwork;
    use starcoin_resource_viewer::MoveValueAnnotator;
    use starcoin_statedb::ChainStateReader;
    use starcoin_vm_types::account_config::core_code_address;
    use test_helper::executor::prepare_genesis;

    #[test]
    fn test_process_genesis_account() -> anyhow::Result<()> {
        // Initialize test storage with genesis
        let net = ChainNetwork::new_test();
        let (chain_statedb, _net) = prepare_genesis();

        // Create statedb from genesis state
        let value_annotator = MoveValueAnnotator::new(&chain_statedb);

        // Get account state for 0x1 address
        let global_states = chain_statedb.dump()?;

        // Find 0x1 account state
        let mut found = false;
        for (account_address, account_state) in global_states.into_iter() {
            if account_address.to_hex_literal() != core_code_address().to_hex_literal() {
                continue;
            }
            found = true;

            // Process 0x1 account
            let (_resource_state_root, _code_state_root, resources_blob, codes_blob) =
                process_account(
                    &chain_statedb,
                    &account_address,
                    account_state,
                    &value_annotator,
                )?;

            // Verify 0x1 has resources and code
            assert!(
                !resources_blob.is_empty(),
                "0x1 account should have resources"
            );
            assert!(
                !codes_blob.is_empty(),
                "0x1 account should have code modules"
            );

            // Print some debug info
            println!("Found {} resources in 0x1", resources_blob.len());
            println!("Found {} code modules in 0x1", codes_blob.len());

            break;
        }

        assert!(found, "0x1 account should exist in genesis state");

        Ok(())
    }

    // #[test]
    // fn test_export_from_statedb() -> anyhow::Result<()> {
    //     // Initialize test storage with genesis
    //     let net = ChainNetwork::new_test();
    //     let (storage, _, chain_info, _) = Genesis::init_storage_for_test_v2(&net)?;
    //
    //     // Get genesis block
    //     let genesis_block = storage
    //         .get_block(chain_info.status().head().id())?
    //         .expect("Genesis block must exist");
    //
    //     // Create statedb from genesis state
    //     let state_root = genesis_block.header().state_root();
    //     let statedb = ChainStateDB::new(storage.clone(), Some(state_root));
    //
    //     // Create a temporary CSV file for output
    //     let temp_dir = TempDir::new()?;
    //     let output_path = temp_dir.path().join("export.csv");
    //     let mut csv_writer = csv::WriterBuilder::new().from_path(&output_path)?;
    //
    //     // Export from statedb
    //     export_from_statedb(&statedb, storage.clone(), state_root, &mut csv_writer)?;
    //
    //     // Verify the CSV file exists and has content
    //     let metadata = std::fs::metadata(&output_path)?;
    //     assert!(metadata.len() > 0, "Export CSV should not be empty");
    //
    //     // Read back the CSV to verify content
    //     let mut csv_reader = csv::ReaderBuilder::new().from_path(&output_path)?;
    //     let mut has_core_address = false;
    //
    //     for result in csv_reader.records() {
    //         let record = result?;
    //         let address: String = record.get(0).unwrap().trim_matches('"').to_string();
    //         if address == "\"0x1\""
    //             || address == "0x1"
    //             || address == "0x00000000000000000000000000000001"
    //         {
    //             has_core_address = true;
    //
    //             // Verify resources and code are not empty
    //             let resources = record.get(2).unwrap();
    //             let codes = record.get(3).unwrap();
    //
    //             assert!(resources.len() > 2, "Resources for 0x1 should not be empty");
    //             assert!(codes.len() > 2, "Code for 0x1 should not be empty");
    //
    //             break;
    //         }
    //     }
    //     assert!(
    //         has_core_address,
    //         "Export should contain the core 0x1 address"
    //     );
    //
    //     Ok(())
    // }
}
