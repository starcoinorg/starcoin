// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use log::debug;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::{on_chain_config::Version, state_view::StateReaderExt};
use tempfile::TempDir;

const MIGRATION_FILE_NAME: &str = "24674819.bcs";
const MIGRATION_FILE_HASH: &str =
    "0xfe67714c2de318b48bf11a153b166110ba80f1b8524df01030a1084a99ae963f";

// Include the migration tar.gz file at compile time
const MIGRATION_TAR_GZ: &[u8] = include_bytes!("../migration/24674819.tar.gz");

pub fn migrate_data_to_statedb(statedb: &ChainStateDB) -> anyhow::Result<HashValue> {
    migrate_legacy_state_data(
        &statedb,
        MIGRATION_TAR_GZ,
        MIGRATION_FILE_NAME,
        MIGRATION_FILE_HASH,
    )
}

pub fn migrate_legacy_state_data(
    statedb: &ChainStateDB,
    tar_gz_contents: &[u8],
    migration_file_name: &str,
    migration_file_expect_hash: &str,
) -> anyhow::Result<HashValue> {
    debug!(
        "migrate_legacy_state_data | Entered, origin state_root:{:?}",
        statedb.state_root()
    );

    let temp_dir = TempDir::new()?;

    // Extract the tar.gz file from embedded data
    let tar_file = flate2::read::GzDecoder::new(tar_gz_contents);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    let bcs_path = temp_dir.path().join(migration_file_name);
    assert!(
        bcs_path.exists(),
        "{:?} does not exist",
        migration_file_name
    );

    debug!(
        "migrate_legacy_state_data | Read bcs from path: {:?}",
        bcs_path
    );
    let bcs_content = std::fs::read(bcs_path)?;

    assert_eq!(
        HashValue::sha3_256_of(&bcs_content),
        HashValue::from_hex_literal(migration_file_expect_hash)?,
        "Content hash should be the same"
    );

    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_content)?;
    debug!("migrate_legacy_state_data | start applying data ...");
    statedb.apply(chain_state_set)?;
    let new_state_root = statedb.commit()?;
    statedb.flush()?;

    debug!(
        "migrate_legacy_state_data | applying data completed, new state root is: {:?}",
        new_state_root
    );

    let new_statedb = statedb.fork_at(new_state_root);

    let stdlib_version = new_statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
    debug!(
        "migrate_legacy_state_data | 0x1 stdlib_version = {}",
        stdlib_version
    );

    // let balance = statedb.get_balance(AccountAddress::ONE)?.unwrap_or(0);
    // info!(
    //     "check_legecy_data_has_migration | 0x1 balance = {}",
    //     balance
    // );
    // assert_eq!(stdlib_version, 12, "Replaced version should 12");
    // assert_eq!(balance, 10000, "Replaced 0x1 balance should 10000");

    let new_state_root = statedb.state_root();

    info!(
        "migrate_legacy_state_data | Exited, the state root of after migration is: {:?}",
        new_state_root
    );
    Ok(new_state_root)
}
