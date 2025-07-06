// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use log::debug;
use starcoin_config::BuiltinNetworkID;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::{on_chain_config::Version, state_view::StateReaderExt};
use tempfile::TempDir;

/// Description: This implementation used for multi-move vm upgrade that
/// migration state data of specification height from mainnet
/// The process is:
///   1. Use resource-code-exporter to export the data of the specified height and calculate its hash
///   2. Copy it to the chain/migration directory
///   3. After starting the node, the block is generated normally.
///     When the first block is reached, `migrate_data_to_statedb` is automatically executed to write the state data to the state storage

pub fn get_migration_main_snapshot() -> anyhow::Result<(&'static str, HashValue, &'static [u8])> {
    // TODO(BobOng): The specified height to be confirm
    Ok((
        "24674819.bcs",
        HashValue::from_hex_literal(
            "0xfe67714c2de318b48bf11a153b166110ba80f1b8524df01030a1084a99ae963f",
        )?,
        include_bytes!("../snapshot/24674819.tar.gz"),
    ))
}

pub fn get_migration_test_snapshot() -> anyhow::Result<(&'static str, HashValue, &'static [u8])> {
    Ok((
        "64925.bcs",
        HashValue::from_hex_literal(
            "0xb450ae07116c9a38fd44b93ce1d7ddbc5cbb8639e7cd30d2921be793905fb5b1",
        )?,
        include_bytes!("../snapshot/64925.tar.gz"),
    ))
}

pub fn should_do_migration(block_id: u64, chain_id: ChainId) -> bool {
    block_id == 3
        && (chain_id == ChainId::new(BuiltinNetworkID::Main.chain_id().id())
            || chain_id == ChainId::new(BuiltinNetworkID::Proxima.chain_id().id()))
}

pub fn migrate_main_data_to_statedb(statedb: &ChainStateDB) -> anyhow::Result<HashValue> {
    let (file_name, data_hash, snapshot_pack) = get_migration_main_snapshot()?;
    migrate_legacy_state_data(statedb, snapshot_pack, file_name, data_hash)
}

pub fn migrate_test_data_to_statedb(statedb: &ChainStateDB) -> anyhow::Result<HashValue> {
    let (file_name, data_hash, snapshot_pack) = get_migration_test_snapshot()?;
    migrate_legacy_state_data(statedb, snapshot_pack, file_name, data_hash)
}

pub fn migrate_legacy_state_data(
    statedb: &ChainStateDB,
    snapshot_pack: &[u8],
    migration_file_name: &str,
    migration_file_expect_hash: HashValue,
) -> anyhow::Result<HashValue> {
    debug!(
        "migrate_legacy_state_data | Entered, origin state_root:{:?}",
        statedb.state_root()
    );

    let temp_dir = TempDir::new()?;

    // Extract the tar.gz file from embedded data
    let tar_file = flate2::read::GzDecoder::new(snapshot_pack);
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
        migration_file_expect_hash,
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

    let new_state_root = statedb.state_root();

    info!(
        "migrate_legacy_state_data | Exited, the stdlib_version: {:?}, new state root is: {:?}",
        stdlib_version, new_state_root
    );
    Ok(new_state_root)
}
