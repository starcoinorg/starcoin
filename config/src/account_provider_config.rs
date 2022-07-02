use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountProviderStrategy;
use starcoin_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Parser)]
pub struct AccountProviderConfig {
    /// Path to the local account provider dir, load the accounts from local dir path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "local-account-dir", parse(from_os_str))]
    pub account_dir: Option<PathBuf>,

    #[clap(
        long = "secret-file",
        help = "file path of private key",
        parse(from_os_str)
    )]
    pub secret_file: Option<PathBuf>,

    #[clap(long = "account-address", requires("secret-file"))]
    pub account_address: Option<AccountAddress>,

    #[serde(skip)]
    #[clap(skip)]
    provider_strategy: AccountProviderStrategy,
}

impl ConfigModule for AccountProviderConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        if opt.account_provider.account_dir.is_some() {
            self.account_dir = opt.account_provider.account_dir.clone()
        }
        if opt.account_provider.secret_file.is_some() {
            self.secret_file = opt.account_provider.secret_file.clone();
            self.account_address = opt.account_provider.account_address;
        }
        assert!(
            !(self.account_dir.is_some() && self.secret_file.is_some()),
            "LocalDB account provider and private key account provider conflict.",
        );
        if self.account_dir.is_some() {
            self.provider_strategy = AccountProviderStrategy::Local
        } else if self.secret_file.is_some() {
            self.provider_strategy = AccountProviderStrategy::PrivateKey
        } else {
            self.provider_strategy = AccountProviderStrategy::RPC
        }
        Ok(())
    }
}

impl AccountProviderConfig {
    pub fn new_local_provider_config(account_dir: PathBuf) -> Self {
        Self {
            account_dir: Some(account_dir),
            secret_file: None,
            account_address: None,
            provider_strategy: AccountProviderStrategy::Local,
        }
    }

    pub fn new_private_key_provider_config(
        secret_file: PathBuf,
        account_address: Option<AccountAddress>,
    ) -> Self {
        Self {
            account_dir: None,
            secret_file: Some(secret_file),
            account_address,
            provider_strategy: AccountProviderStrategy::PrivateKey,
        }
    }

    pub fn get_strategy(&self) -> AccountProviderStrategy {
        debug_assert!(!(self.account_dir.is_some() && self.secret_file.is_some()));
        if self.account_dir.is_some() {
            AccountProviderStrategy::Local
        } else if self.secret_file.is_some() {
            AccountProviderStrategy::PrivateKey
        } else {
            AccountProviderStrategy::RPC
        }
    }
}

impl Default for AccountProviderConfig {
    fn default() -> Self {
        Self {
            account_dir: None,
            secret_file: None,
            account_address: None,
            provider_strategy: AccountProviderStrategy::RPC,
        }
    }
}
