// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use starcoin_account::account_storage::AccountStorage;
use starcoin_account::AccountManager;
use starcoin_account_api::AccountInfo;
use starcoin_config::{NodeConfig, StarcoinOpt};
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::Storage;
use starcoin_types::startup_info::ChainInfo;
use std::sync::Arc;

pub mod cli_state;
pub mod gen_data;
pub mod gen_genesis;
pub mod gen_genesis_config;

pub fn init_or_load_data_dir(
    global_opt: &StarcoinOpt,
    password: Option<String>,
) -> Result<(NodeConfig, Arc<Storage>, ChainInfo, AccountInfo)> {
    let config = NodeConfig::load_with_opt(global_opt)?;
    if config.base().base_data_dir().is_temp() {
        bail!("Please set data_dir option.")
    }
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new_with_capacity(config.storage.cache_size(), None),
        DBStorage::new(config.storage.dir(), config.storage.rocksdb_config(), None)?,
    ))?);
    let (chain_info, _genesis) =
        Genesis::init_and_check_storage(config.net(), storage.clone(), config.data_dir())?;
    let vault_config = &config.vault;
    let account_storage =
        AccountStorage::create_from_path(vault_config.dir(), config.storage.rocksdb_config())?;
    let manager = AccountManager::new(account_storage, config.net().chain_id())?;
    let account = match manager.default_account_info()? {
        Some(account) => account,
        None => manager
            .create_account(&password.unwrap_or_else(|| "".to_string()))?
            .info(),
    };
    Ok((config, storage, chain_info, account))
}
