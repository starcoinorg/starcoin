// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::ChainStateWriter;
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::{on_chain_config::Version, state_view::StateReaderExt};
use tempfile::TempDir;

const MIGRATION_FILE_NAME: &str = "24674819.bcs";
const MIGRATION_FILE_HASH: &str =
    "0xfe67714c2de318b48bf11a153b166110ba80f1b8524df01030a1084a99ae963f";

pub fn migrate_legacy_state_data(
    statedb: &ChainStateDB,
    _net: &ChainNetwork,
) -> anyhow::Result<()> {
    let tar_gz_path = std::path::Path::new("migration/24674819.tar.gz");
    info!(
        "migrate_legacy_state_data | Entered, Extracting tar.gz file from: {}",
        tar_gz_path.display()
    );

    let temp_dir = TempDir::new()?;

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    let bcs_path = temp_dir.path().join(MIGRATION_FILE_NAME);
    let bcs_content = std::fs::read_to_string(&bcs_path)?;

    assert_eq!(
        HashValue::sha3_256_of(bcs_content.as_bytes()),
        HashValue::from_hex_literal(MIGRATION_FILE_HASH)?,
        "Content hash should be the same"
    );

    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(bcs_content.as_bytes())?;
    statedb.apply(chain_state_set)?;

    let stdlib_version = statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
    info!(
        "check_legecy_data_has_migration | stdlib_version = {}",
        stdlib_version
    );
    assert_eq!(stdlib_version, 12, "Replaced version should 12");

    info!("migrate_legacy_state_data | Exited");

    Ok(())
}
