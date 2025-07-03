// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::{account_address::AccountAddress, state_set::ChainStateSet};
use starcoin_vm_types::{on_chain_config::Version, state_view::StateReaderExt};
use tempfile::TempDir;

const MIGRATION_FILE_NAME: &str = "24674819.bcs";
const MIGRATION_FILE_HASH: &str =
    "0xfe67714c2de318b48bf11a153b166110ba80f1b8524df01030a1084a99ae963f";

// Include the migration tar.gz file at compile time
const MIGRATION_TAR_GZ: &[u8] = include_bytes!("../migration/24674819.tar.gz");

pub fn migrate_legacy_state_data(
    statedb: &ChainStateDB,
    _net: &ChainNetwork,
) -> anyhow::Result<()> {
    info!("migrate_legacy_state_data | Entered, Extracting tar.gz file from embedded data");

    let temp_dir = TempDir::new()?;

    // Extract the tar.gz file from embedded data
    let tar_file = flate2::read::GzDecoder::new(MIGRATION_TAR_GZ);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    let bcs_path = temp_dir.path().join(MIGRATION_FILE_NAME);
    assert!(
        bcs_path.exists(),
        "{:?} does not exist",
        MIGRATION_FILE_NAME
    );
    let bcs_content = std::fs::read(bcs_path)?;

    assert_eq!(
        HashValue::sha3_256_of(&bcs_content),
        HashValue::from_hex_literal(MIGRATION_FILE_HASH)?,
        "Content hash should be the same"
    );

    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_content)?;
    statedb.apply(chain_state_set)?;
    let new_state_root = statedb.commit()?;
    statedb.flush()?;

    let new_statedb = statedb.fork_at(new_state_root);

    let stdlib_version = new_statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
    info!(
        "check_legecy_data_has_migration | 0x1 stdlib_version = {}",
        stdlib_version
    );

    let balance = new_statedb.get_balance(AccountAddress::ONE)?.unwrap_or(0);
    info!(
        "check_legecy_data_has_migration | 0x1 balance = {}",
        balance
    );
    assert_eq!(stdlib_version, 12, "Replaced version should 12");
    assert_eq!(balance, 10000, "Replaced 0x1 balance should 10000");

    info!(
        "migrate_legacy_state_data | Exited, the state root of after migration is: {:?}",
        new_statedb.state_root()
    );

    Ok(())
}
