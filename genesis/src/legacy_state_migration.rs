// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use flate2::read::GzDecoder;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    account_address::AccountAddress,
    state_set::{AccountStateSet, ChainStateSet, StateSet},
};
use std::fs::File;
use tar::Archive;
use tempfile::tempdir;

const CSV_FILE_HASH: &str = "0xab47a1acc0ad8ee89af6222f36828f834cbab5273211aa5b0fbcf1d6f3f19554";
const CSV_FILE_NAME: &str = "legacy-state-data.csv";

fn prepare_csv_content() -> anyhow::Result<String> {
    // 1. Create a temporary directory
    let dir = tempdir()?;
    let dir_path = dir.path();

    // 2. Open the original tar.gz file directly
    let tar_gz_file = File::open("migration/legacy-state-data.csv.tar.gz")?;
    let decompressed = GzDecoder::new(tar_gz_file);
    let mut archive = Archive::new(decompressed);

    // 3. unpack it to a temporary directory
    archive.unpack(dir_path)?;

    // 4. Read the unpacked csv file
    let csv_path = dir_path.join(CSV_FILE_NAME);
    if !csv_path.exists() {
        anyhow::bail!(
            "CSV file not found after extraction: {}",
            csv_path.display()
        );
    }
    let csv_content = std::fs::read_to_string(&csv_path)?;

    // Hash check
    let csv_file_hash = HashValue::sha3_256_of(csv_content.as_bytes());
    if csv_file_hash.to_string() != CSV_FILE_HASH {
        anyhow::bail!(
            "CSV file hash mismatch: expected {}, got {}",
            CSV_FILE_HASH,
            csv_file_hash
        );
    }
    Ok(csv_content)
}

pub fn legacy_account_state_migration(
    statedb: &ChainStateDB,
    maxium_count: Option<u64>,
) -> anyhow::Result<()> {
    info!("legacy_account_state_migration | entered");
    let csv_content = prepare_csv_content()?;
    let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());
    let mut chain_state_set_data = Vec::new();
    let mut processed = 0;
    let maxium_process_count = maxium_count.unwrap_or(u64::MAX);

    for result in csv_reader.records() {
        let record = result?;
        let account_address: AccountAddress = serde_json::from_str(&record[0])?;
        assert_eq!(record.len(), 5);
        info!(
            "legacy_account_state_migration | Processing record {}: account {}",
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

        info!(
            "legacy_account_state_migration | Progress: {} records processed",
            processed
        );
        if processed >= maxium_process_count {
            break;
        }
    }
    info!(
        "legacy_account_state_migration | Applying {} state sets to statedb...",
        chain_state_set_data.len()
    );
    statedb.apply(ChainStateSet::new(chain_state_set_data))?;
    info!("legacy_account_state_migration | exited");
    Ok(())
}
