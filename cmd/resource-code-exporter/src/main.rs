// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use cmd_utils::move_struct_serde::MoveStruct;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::MoveValueAnnotator;
use starcoin_state_tree::StateTree;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
use starcoin_types::{
    access_path::DataType, account_state::AccountState, language_storage::StructTag,
};
use starcoin_vm_types::account_address::AccountAddress;
use std::{
    convert::TryInto,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

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
    let value_annotator = MoveValueAnnotator::new(&statedb);

    let state_tree = StateTree::<AccountAddress>::new(storage.clone(), Some(root));

    let mut csv_writer = csv::WriterBuilder::new().from_path(output)?;

    // write csv header.
    {
        csv_writer.write_field("address")?;
        csv_writer.write_field("state_root")?;
        csv_writer.write_field("resource")?;
        csv_writer.write_field("code")?;
        csv_writer.write_record(None::<&[u8]>)?;
    }

    let global_states = state_tree.dump()?;
    println!("Total accounts to process: {}", global_states.len());

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;
    for (address_bytes, account_state_bytes) in global_states.iter() {
        let account: AccountAddress = bcs_ext::from_bytes(address_bytes)?;
        let account_state: AccountState = account_state_bytes.as_slice().try_into()?;

        let storage_roots = account_state.storage_roots();
        let resource_root = storage_roots[DataType::RESOURCE.storage_index()];
        let code_root = storage_roots[DataType::CODE.storage_index()];

        println!("Processing account: {:?}", account);
        println!("State root: {:?}", root);

        let mut resources = Vec::new();
        let mut codes = Vec::new();

        if let Some(root) = resource_root {
            let account_tree: StateTree<StructTag> =
                StateTree::<StructTag>::new(storage.clone(), Some(root));
            let resource_tags = account_tree.dump()?;
            println!(
                "Found {} resource tags for account: {:?}",
                resource_tags.len(),
                account
            );

            for (tag_bytes, _) in resource_tags.iter() {
                let tag: StructTag = bcs_ext::from_bytes(tag_bytes)?;
                let data = account_tree.get(&tag)?;

                // Some(resource_json_value)
                if let Some(d) = data {
                    let annotated_struct =
                        value_annotator.view_struct(tag.clone(), d.as_slice())?;
                    let resource_json_value = serde_json::to_value(MoveStruct(annotated_struct))?;
                    resources.push((tag.to_string(), resource_json_value));
                }
            }
        }

        if let Some(root) = code_root {
            let account_tree = StateTree::<StructTag>::new(storage.clone(), Some(root));
            let code_tags = account_tree.dump()?;
            println!(
                "Found {} code tags for account: {:?}",
                code_tags.len(),
                account
            );

            for (tag_bytes, _) in code_tags.iter() {
                let tag: StructTag = bcs_ext::from_bytes(tag_bytes)?;
                let data = account_tree.get(&tag)?;

                if let Some(d) = data {
                    let annotated_struct =
                        value_annotator.view_struct(tag.clone(), d.as_slice())?;
                    let code_json_value = serde_json::to_value(MoveStruct(annotated_struct))?;
                    codes.push((tag.to_string(), code_json_value));
                }
            }
        }

        // write csv record.
        let record = vec![
            serde_json::to_string(&account)?,
            serde_json::to_string(&root)?,
            serde_json::to_string(&resources)?,
            serde_json::to_string(&codes)?,
        ];
        csv_writer.serialize(record)?;
        processed += 1;
        println!("Processed {}/{} accounts", processed, global_states.len());
    }
    println!("Total processing time: {} ms", now.elapsed().as_millis());
    // flush csv writer
    csv_writer.flush()?;
    Ok(())
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
