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
    account_config::{BalanceResource, STC_TOKEN_CODE_STR},
    genesis_config::ChainId,
    language_storage::StructTag,
    on_chain_config::Version,
    state_view::StateReaderExt,
    token::token_code::TokenCode,
};
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
    statedb.apply(chain_state_set.clone())?;
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

    verify_token_state_is_complete(&new_statedb, &chain_state_set, STC_TOKEN_CODE_STR)?;

    Ok(new_state_root)
}

pub fn verify_token_state_is_complete(
    statedb: &ChainStateDB,
    original_chain_state_set: &ChainStateSet,
    token_code: &str,
) -> anyhow::Result<()> {
    info!("verify_state_is_complete | Starting STC balance verification...");

    let stc_token_code = TokenCode::from_str(token_code)?;
    let stc_balance_struct_tag =
        BalanceResource::struct_tag_for_token(stc_token_code.clone().try_into()?);

    // Extract STC balances from original data
    let mut original_balances = HashMap::new();
    let mut total_original = 0u128;

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
    if total_original != total_current {
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
        Ok(())
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
