// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    account_address::AccountAddress,
    state_set::{AccountStateSet, ChainStateSet, StateSet},
};

const CSV_FILE_HASH: &str = "54426e3df888aae87f41d3f5908d406f90a8a25cd1389e3a33f30d0f5217d8c6";
const CSV_FILE_NAME: &str = "legecy-state-data.csv";

pub fn legecy_account_state_migration(statedb: &ChainStateDB) -> anyhow::Result<()> {
    info!("legecy_account_state_migration | entered");

    // Read CSV file content based on compilation mode
    let csv_content = if cfg!(feature = "embed_csv") {
        // In production, read from embedded file
        let csv_file = include_bytes!("../migration/legecy-state-data.csv");
        // Calculate hash directly from the bytes
        let csv_file_hash = HashValue::sha3_256_of(csv_file);
        assert_eq!(csv_file_hash.to_string(), CSV_FILE_HASH);
        std::str::from_utf8(csv_file)?.to_string()
    } else {
        // In development, read from file system
        let file_path = format!("migration/{}", CSV_FILE_NAME);
        let csv_file = std::fs::read(&file_path)?;
        // Calculate hash directly from the bytes
        let csv_file_hash = HashValue::sha3_256_of(&csv_file);
        assert_eq!(csv_file_hash.to_string(), CSV_FILE_HASH);
        String::from_utf8(csv_file)?
    };

    let mut csv_reader = csv::Reader::from_reader(csv_content.as_bytes());
    let mut chain_state_set_data = Vec::new();
    let mut processed = 0;

    for result in csv_reader.records() {
        let record = result?;
        let account_address: AccountAddress = serde_json::from_str(&record[0])?;
        assert_eq!(record.len(), 5);
        info!(
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

        info!(
            "legecy_account_state_migration | Progress: {} records processed",
            processed
        );
    }
    info!(
        "Applying {} state sets to statedb...",
        chain_state_set_data.len()
    );
    statedb.apply(ChainStateSet::new(chain_state_set_data))?;
    info!("legecy_account_state_migration | exited");
    Ok(())
}