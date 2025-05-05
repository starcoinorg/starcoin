use clap::Parser;
use starcoin_crypto::HashValue;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_storage::{
    db_storage::DBStorage, storage::StorageInstance, BlockStore, Storage, StorageVersion,
};
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
    // write csv header
    {
        writer.write_field("address")?;
        writer.write_field("state_root")?;
        writer.write_field("account_state")?;
        writer.write_record(None::<&[u8]>)?;
    }

    let global_states = statedb.dump()?;
    println!("Total accounts to process: {}", global_states.len());

    use std::time::Instant;
    let now = Instant::now();
    let mut processed = 0;

    for (account_address, account_state) in global_states.into_iter() {
        // Serialize the entire account state
        let account_state_json = serde_json::to_string(&account_state)?;

        // write csv record
        let record = vec![
            serde_json::to_string(&account_address)?,
            serde_json::to_string(&root)?,
            account_state_json,
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

#[cfg(test)]
mod test {
    use super::*;
    use starcoin_config::ChainNetwork;
    use test_helper::executor::prepare_genesis;
    use std::io::Cursor;

    #[test]
    fn test_export_from_statedb() -> anyhow::Result<()> {
        // Initialize test storage with genesis
        let net = ChainNetwork::new_test();
        let (chain_statedb, _net) = prepare_genesis();

        // Create a buffer to write CSV data
        let mut buffer = Cursor::new(Vec::new());
        {
            let mut csv_writer = csv::WriterBuilder::new().from_writer(&mut buffer);
            export_from_statedb(&chain_statedb, chain_statedb.state_root(), &mut csv_writer)?;
        }

        // Get the written data
        let data = buffer.into_inner();
        let data_str = String::from_utf8(data)?;
        println!("Exported CSV data:\n{}", data_str);

        // Verify the data contains expected content
        let mut csv_reader = csv::Reader::from_reader(data_str.as_bytes());
        let mut has_data = false;
        for result in csv_reader.records() {
            let record = result?;
            // println!("Record: {:?}", record);
            has_data = true;
        }

        assert!(has_data, "CSV should contain exported data");
        Ok(())
    }
}
