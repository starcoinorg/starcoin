// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use log::{debug, error};
use starcoin_config::BuiltinNetworkID;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::{
    account_config::{BalanceResource, TokenInfo, G_STC_TOKEN_CODE, STC_TOKEN_CODE_STR},
    genesis_config::ChainId,
    language_storage::StructTag,
    on_chain_config::Version,
    state_view::StateReaderExt,
    token::token_code::TokenCode,
};
use std::path::Path;
use std::{collections::HashMap, str::FromStr};
use tempfile::TempDir;

mod state_filter;
pub use state_filter::filter_chain_state_set;

/// Description: This implementation used for multi-move vm upgrade that
/// migration state data of specification height from mainnet
/// The process is:
///   1. Use resource-code-exporter to export the data of the specified height and calculate its hash
///   2. Copy it to the chain/migration directory
///   3. After starting the node, the block is generated normally.
///     When the first block is reached, `migrate_data_to_statedb` is automatically executed to write the state data to the state storage

//
#[derive(Clone)]
pub enum MigrationDataSet {
    Main(&'static str, HashValue, &'static [u8]),
    Main0x1(&'static str, HashValue, &'static [u8]),
    Test(&'static str, HashValue, &'static [u8]),
}

impl std::fmt::Debug for MigrationDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main(file_name, hash, _) => {
                write!(
                    f,
                    "MigrationDataSet::Main(file: {}, hash: {})",
                    file_name, hash
                )
            }
            Self::Main0x1(file_name, hash, _) => {
                write!(
                    f,
                    "MigrationDataSet::Main0x1(file: {}, hash: {})",
                    file_name, hash
                )
            }
            Self::Test(file_name, hash, _) => {
                write!(
                    f,
                    "MigrationDataSet::Test(file: {}, hash: {})",
                    file_name, hash
                )
            }
        }
    }
}

impl MigrationDataSet {
    /// Create a migration dataset based on chain ID
    pub fn from_chain_id(chain_id: ChainId) -> Self {
        match chain_id {
            id if id == BuiltinNetworkID::Main.chain_id() => Self::main(),
            id if id == BuiltinNetworkID::Proxima.chain_id() => Self::main(),
            id if id == BuiltinNetworkID::Test.chain_id()
                || id == BuiltinNetworkID::Dev.chain_id() =>
            {
                Self::test()
            }
            _ => Self::test(),
        }
    }

    /// Create Main network migration dataset
    pub fn main() -> Self {
        Self::Main(
            "24674819.bcs",
            HashValue::from_hex_literal(
                "0xfe67714c2de318b48bf11a153b166110ba80f1b8524df01030a1084a99ae963f",
            )
            .unwrap(),
            include_bytes!("../snapshot/24674819.tar.gz"),
        )
    }

    /// Create Main0x1 network migration dataset
    pub fn main_0x1() -> Self {
        Self::Main0x1(
            "24674819-0x1.bcs",
            HashValue::from_hex_literal(
                "5efc1f27548fc1d46a2c86272f1fbc567a32746e04a1b1a608c17f60cf58771d",
            )
            .unwrap(),
            include_bytes!("../snapshot/24674819-0x1.tar.gz"),
        )
    }

    /// Create Test network migration dataset
    pub fn test() -> Self {
        Self::Test(
            "64925.bcs",
            HashValue::from_hex_literal(
                "0xb450ae07116c9a38fd44b93ce1d7ddbc5cbb8639e7cd30d2921be793905fb5b1",
            )
            .unwrap(),
            include_bytes!("../snapshot/64925.tar.gz"),
        )
    }

    /// Extract the migration data tuple
    pub fn as_tuple(&self) -> (&'static str, HashValue, &'static [u8]) {
        match self {
            Self::Main(f, h, d) => (f, *h, d),
            Self::Main0x1(f, h, d) => (f, *h, d),
            Self::Test(f, h, d) => (f, *h, d),
        }
    }
}

/// Migration executor that handles the complete migration process
pub struct MigrationExecutor {
    chain_id: ChainId,
}

impl MigrationExecutor {
    /// Create a new migration executor
    pub fn new(chain_id: ChainId) -> Self {
        Self { chain_id }
    }

    /// Execute the migration process
    pub fn execute(
        &self,
        statedb: &ChainStateDB,
        data_set: Option<MigrationDataSet>,
    ) -> anyhow::Result<HashValue> {
        info!(
            "MigrationExecutor::execute | Starting migration for chain_id: {:?}",
            self.chain_id
        );

        // Get migration dataset from contexts
        let data_set = data_set.unwrap_or(MigrationDataSet::from_chain_id(self.chain_id));
        info!(
            "MigrationExecutor::execute | Selected migration dataset: {:?}",
            data_set
        );

        // Apply migration
        let (file, hash, pack) = data_set.as_tuple();
        let (state_root, stateset_bcs_from_file) =
            Self::migrate_legacy_state_data(statedb, file, hash, pack)?;

        let statedb = statedb.fork_at(state_root);

        debug!(
            "Migration data apply completed, state root: {:?}, version: {}",
            statedb.state_root(),
            get_version_from_statedb(&statedb)?,
        );

        // Verify STC balance consistency
        let final_state_root = if !stateset_bcs_from_file.state_sets().is_empty() {
            Self::verify_token_state_is_complete(
                &statedb,
                &stateset_bcs_from_file,
                STC_TOKEN_CODE_STR,
            )?
        } else {
            state_root
        };

        let statedb = statedb.fork_at(final_state_root);
        assert_eq!(
            statedb.get_chain_id()?,
            self.chain_id,
            "it should be target chain id"
        );
        assert_eq!(
            final_state_root,
            statedb.state_root(),
            "it should be state root"
        );

        info!(
            "MigrationExecutor::execute | Exited, migration completed successfully with state root: {:?}",
            final_state_root
        );
        Ok(final_state_root)
    }

    pub fn migrate_legacy_state_data(
        statedb: &ChainStateDB,
        migration_file_name: &str,
        migration_file_expect_hash: HashValue,
        snapshot_tar_pack: &[u8],
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

        // Apply state filtering before migration
        info!(
            "migrate_legacy_state_data | Filtering state data, original accounts: {}",
            chain_state_set.len()
        );
        let filtered_chain_state_set = filter_chain_state_set(chain_state_set.clone(), statedb)?;
        info!(
            "migrate_legacy_state_data | State filtering completed, filtered accounts: {}",
            filtered_chain_state_set.len()
        );

        debug!("migrate_legacy_state_data | start applying filtered data ...");
        statedb.apply(filtered_chain_state_set.clone())?;

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
                        if let Ok(balance_resource) = bcs_ext::from_bytes::<BalanceResource>(value)
                        {
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
            state_root = Self::replace_statistic_amount_with_actual_amount(statedb, total_current)?;
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

    fn replace_statistic_amount_with_actual_amount(
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
}

/// Legacy function for backward compatibility
pub fn do_migration(
    statedb: &ChainStateDB,
    chain_id: ChainId,
    data_set: Option<MigrationDataSet>,
) -> anyhow::Result<HashValue> {
    debug!(
        "do_migration | Entered, statedb current state root: {:?}",
        statedb.state_root()
    );
    let executor = MigrationExecutor::new(chain_id);
    let ret = executor.execute(statedb, data_set);

    debug!("do_migration | Executed");
    ret
}

pub fn should_do_migration(block_id: u64, chain_id: ChainId) -> bool {
    block_id == 3
        && (chain_id == ChainId::new(BuiltinNetworkID::Main.chain_id().id())
            || chain_id == ChainId::new(BuiltinNetworkID::Proxima.chain_id().id()))
}

pub fn get_version_from_statedb(statedb: &ChainStateDB) -> anyhow::Result<u64> {
    statedb
        .get_on_chain_config::<Version>()?
        .map(|version| version.major)
        .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))
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
