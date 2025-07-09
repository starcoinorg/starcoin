// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use bcs_ext;
use log::{debug, error};
use starcoin_config::BuiltinNetworkID;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::{
    access_path::AccessPath,
    account_config::{BalanceResource, TokenInfo, G_STC_TOKEN_CODE, STC_TOKEN_CODE_STR},
    genesis_config::ChainId,
    language_storage::StructTag,
    move_resource::MoveResource,
    on_chain_config::Version,
    state_view::StateReaderExt,
    token::token_code::TokenCode,
};
use std::path::Path;
use std::{collections::HashMap, str::FromStr};
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

pub fn get_migration_main_0x1_snapshot() -> anyhow::Result<(&'static str, HashValue, &'static [u8])>
{
    // TODO(BobOng): The specified height to be confirm
    Ok((
        "24674819-0x1.bcs",
        HashValue::from_hex_literal(
            "5efc1f27548fc1d46a2c86272f1fbc567a32746e04a1b1a608c17f60cf58771d",
        )?,
        include_bytes!("../snapshot/24674819-0x1.tar.gz"),
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

/// Determine whether to use test migration data instead of main migration data
/// This function checks various conditions to decide which migration data to use:
/// 1. If running in test environment (cfg(test))
/// 2. If specific environment variable is set
/// 3. If chain_id indicates test network
pub fn should_use_test_migration(_chain_id: ChainId) -> bool {
    // Check for environment variable override
    if let Ok(env_value) = std::env::var("STARCOIN_USE_TEST_MIGRATION") {
        if env_value == "true" || env_value == "1" {
            return true;
        }
    }

    // Check if this is a test network
    let test_networks = [
        BuiltinNetworkID::Test.chain_id(),
        BuiltinNetworkID::Dev.chain_id(),
    ];
    test_networks.contains(&_chain_id)
}

pub fn migrate_main_data_to_statedb(
    statedb: &ChainStateDB,
) -> anyhow::Result<(HashValue, ChainStateSet)> {
    let (file_name, data_hash, snapshot_pack) = get_migration_main_snapshot()?;
    migrate_legacy_state_data(statedb, snapshot_pack, file_name, data_hash)
}

pub fn migrate_main_0x1_data_to_statedb(
    statedb: &ChainStateDB,
) -> anyhow::Result<(HashValue, ChainStateSet)> {
    let (file_name, data_hash, snapshot_pack) = get_migration_main_0x1_snapshot()?;
    migrate_legacy_state_data(statedb, snapshot_pack, file_name, data_hash)
}

pub fn migrate_test_data_to_statedb(
    statedb: &ChainStateDB,
) -> anyhow::Result<(HashValue, ChainStateSet)> {
    let (file_name, data_hash, snapshot_pack) = get_migration_test_snapshot()?;
    migrate_legacy_state_data(statedb, snapshot_pack, file_name, data_hash)
}

pub fn do_migration(statedb: &ChainStateDB, chain_id: ChainId) -> anyhow::Result<HashValue> {
    let (state_root, stateset_bcs_from_file) = if should_use_test_migration(chain_id) {
        migrate_test_data_to_statedb(&statedb)
    } else {
        migrate_main_0x1_data_to_statedb(&statedb)
    }?;

    let statedb = statedb.fork_at(state_root);

    debug!(
        "do_migration | appying data completed, state root: {:?}, version: {}",
        statedb.state_root(),
        get_version_from_statedb(&statedb)?,
    );

    let stdlib_version = statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
    debug!(
        "do_migration | Exited, the stdlib_version: {:?}, stc_token_info: {:?}",
        stdlib_version,
        statedb.get_stc_info()?.unwrap()
    );
    // Verify STC should
    let state_root =
        verify_token_state_is_complete(&statedb, &stateset_bcs_from_file, STC_TOKEN_CODE_STR)?;
    maybe_replace_chain_id_after_migration(&statedb.fork_at(state_root), chain_id)
}

pub fn get_version_from_statedb(statedb: &ChainStateDB) -> anyhow::Result<u64> {
    Ok(statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?)
}

pub fn migrate_legacy_state_data(
    statedb: &ChainStateDB,
    snapshot_tar_pack: &[u8],
    migration_file_name: &str,
    migration_file_expect_hash: HashValue,
) -> anyhow::Result<(HashValue, ChainStateSet)> {
    debug!(
        "migrate_legacy_state_data | Entered, origin state_root:{:?}",
        statedb.state_root()
    );

    let temp_dir = TempDir::new()?;
    let bcs_content = unpack_from_tar(snapshot_tar_pack, temp_dir.path(), migration_file_name)?;

    assert_eq!(
        HashValue::sha3_256_of(&bcs_content),
        migration_file_expect_hash,
        "Content hash should be the same"
    );

    let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_content)?;

    debug!("migrate_legacy_state_data | start applying data ...");
    statedb.apply(chain_state_set.clone())?;
    let state_root = statedb.commit()?;
    statedb.flush()?;

    Ok((state_root, chain_state_set))
}

pub fn verify_token_state_is_complete(
    statedb: &ChainStateDB,
    original_chain_state_set: &ChainStateSet,
    token_code: &str,
) -> anyhow::Result<HashValue> {
    info!("verify_state_is_complete | Starting STC balance verification...");

    let stc_token_code = TokenCode::from_str(token_code)?;
    let stc_balance_struct_tag =
        BalanceResource::struct_tag_for_token(stc_token_code.clone().try_into()?);

    // Extract STC balances from original data
    let mut original_balances = HashMap::new();
    let mut total_original = 0u128;
    let mut state_root = statedb.state_root();

    debug!(
        "verify_state_is_complete | stc_balance_struct_tag: {:?}",
        stc_balance_struct_tag
    );

    for (address, account_state_set) in original_chain_state_set.state_sets() {
        if let Some(resource_set) = account_state_set.resource_set() {
            for (key, value) in resource_set.iter() {
                if let Ok(struct_tag) = bcs_ext::from_bytes::<StructTag>(key) {
                    if struct_tag != stc_balance_struct_tag {
                        // Ignore the tag that not balance type
                        continue;
                    }
                    if let Ok(balance_resource) = bcs_ext::from_bytes::<BalanceResource>(value) {
                        let balance = balance_resource.token();
                        original_balances.insert(*address, balance);
                        total_original += balance;
                    }
                }
            }
        }
    }

    info!(
        "verify_state_is_complete | Found {} accounts with STC balances, total: {}",
        original_balances.len(),
        total_original
    );

    // Verify balances in current state
    let mut errors = Vec::new();
    let mut total_current = 0u128;

    for (address, expected_balance) in &original_balances {
        match statedb.get_balance_by_token_code(*address, stc_token_code.clone()) {
            Ok(Some(actual_balance)) => {
                if actual_balance != *expected_balance {
                    errors.push(format!(
                        "Balance mismatch for {}: expected {}, got {}",
                        address, expected_balance, actual_balance
                    ));
                } else {
                    total_current += actual_balance;
                }
            }
            Ok(None) => {
                errors.push(format!(
                    "Balance not found for {}: expected {}",
                    address, expected_balance
                ));
            }
            Err(e) => {
                errors.push(format!("Error reading balance for {}: {}", address, e));
            }
        }
    }

    // Check total balance consistency
    if total_original == total_current {
        debug!("verify_token_state_is_complete | Update total token issue count of statistic to TokenInfo, current: {:?}", total_current);
        state_root = replace_chain_stc_token_amount(&statedb, total_current)?;
    } else {
        errors.push(format!(
            "Total balance mismatch: original {}, current {}",
            total_original, total_current
        ));
    }

    if errors.is_empty() {
        debug!(
            "STC balance verification passed! {} accounts, total: {}",
            original_balances.len(),
            total_current
        );
        debug!("verify_state_is_complete | Exited");
        Ok(state_root)
    } else {
        error!(
            "STC balance verification failed with {} errors:",
            errors.len()
        );
        for error in &errors {
            error!("  {}", error);
        }
        Err(format_err!("STC balance verification failed"))
    }
}

/// Extract the tar.gz file from embedded data
fn unpack_from_tar(
    snapshot_tar_pack: &[u8],
    extract_dir: &Path,
    file_name: &str,
) -> anyhow::Result<Vec<u8>> {
    let tar_file = flate2::read::GzDecoder::new(snapshot_tar_pack);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(extract_dir)?;

    let bcs_path = extract_dir.join(file_name);
    assert!(bcs_path.exists(), "{:?} does not exist", file_name);

    debug!("unpack_from_tar | Read bcs from path: {:?}", bcs_path);
    Ok(std::fs::read(bcs_path)?)
}
fn replace_chain_stc_token_amount(
    statedb: &ChainStateDB,
    total_current: u128,
) -> anyhow::Result<HashValue> {
    let mut stc_info = statedb.get_stc_info()?.unwrap();
    stc_info.total_value = total_current;
    statedb.set(
        &TokenInfo::resource_path_for(G_STC_TOKEN_CODE.clone().try_into()?),
        bcs_ext::to_bytes(&stc_info)?,
    )?;
    let state_root = statedb.commit()?;
    statedb.flush()?;
    Ok(state_root)
}

/// Since the imported data is the main network,
/// if it is not modified to the context network id after applying the data,
/// it will fail when checking block_meta.
fn maybe_replace_chain_id_after_migration(
    statedb: &ChainStateDB,
    chain_id: ChainId,
) -> anyhow::Result<HashValue> {
    debug!(
        "replace_chain_id_after_migration | Entered, replacing chain_id to: {}",
        chain_id
    );

    // Get the current ChainId resource from genesis address
    let current_chain_id = statedb.get_chain_id()?;
    debug!(
        "replace_chain_id_after_migration | Current chain_id: {}, new chain_id: {}",
        current_chain_id, chain_id
    );

    // If the chain_id is already correct, no need to change
    if current_chain_id == chain_id {
        debug!("replace_chain_id_after_migration | ChainId already matches, no change needed");
        return Ok(statedb.state_root());
    }

    // Create the access path for ChainId resource at genesis address
    statedb.set(
        &AccessPath::resource_access_path(
            starcoin_vm_types::account_config::genesis_address(),
            ChainId::struct_tag(),
        ),
        bcs_ext::to_bytes(&chain_id)?,
    )?;

    // Commit the changes and get new state root
    let new_state_root = statedb.commit()?;
    statedb.flush()?;

    debug!(
        "replace_chain_id_after_migration | Exited, ChainId replaced successfully, new state_root: {:?}",
        new_state_root
    );
    Ok(new_state_root)
}
