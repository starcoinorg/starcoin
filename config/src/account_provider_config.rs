use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::{bail, Result};
use clap::{value_parser, Parser};
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountProviderStrategy;
use starcoin_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::sync::Arc;

pub const G_ENV_PRIVATE_KEY: &str = "STARCOIN_PRIVATE_KEY";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Parser)]
pub struct AccountProviderConfig {
    /// Path to the local account provider dir, load the accounts from local dir path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long = "local-account-dir", value_parser = value_parser!(std::ffi::OsString))]
    pub account_dir: Option<PathBuf>,

    /// Path to the secret file storing the private key.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(
        long = "secret-file",
        help = "file path of private key",
        value_parser = value_parser!(std::ffi::OsString)
    )]
    pub secret_file: Option<PathBuf>,

    /// Read private from env variable `STARCOIN_PRIVATE_KEY`.
    #[serde(default)]
    #[arg(long = "from-env")]
    pub from_env: bool,

    #[serde(skip)]
    #[arg(skip)]
    pub account_address: Option<AccountAddress>,

    #[serde(skip)]
    #[arg(skip)]
    provider_strategy: AccountProviderStrategy,
}

impl ConfigModule for AccountProviderConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        if (self.account_dir.is_some() as i32)
            + (self.secret_file.is_some() as i32)
            + (self.from_env as i32)
            > 1
        {
            bail!("Account provider conflicts")
        };
        if opt.account_provider.account_dir.is_some() {
            self.account_dir
                .clone_from(&opt.account_provider.account_dir);
        }
        if opt.account_provider.secret_file.is_some() {
            self.secret_file
                .clone_from(&opt.account_provider.secret_file);
            self.account_address = opt.account_provider.account_address;
        }
        self.from_env = opt.account_provider.from_env;
        if self.account_dir.is_some() {
            self.provider_strategy = AccountProviderStrategy::Local
        } else if self.secret_file.is_some() || self.from_env {
            self.provider_strategy = AccountProviderStrategy::PrivateKey
        } else {
            self.provider_strategy = AccountProviderStrategy::RPC
        }
        Ok(())
    }
}

impl AccountProviderConfig {
    pub fn new_local_provider_config(account_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            account_dir: Some(account_dir),
            secret_file: None,
            from_env: false,
            account_address: None,
            provider_strategy: AccountProviderStrategy::Local,
        })
    }

    pub fn new_private_key_provider_config(
        secret_file: Option<PathBuf>,
        account_address: Option<AccountAddress>,
        from_env: bool,
    ) -> Result<Self> {
        if secret_file.is_some() && from_env {
            bail!("Arg secret_file conflict with from_env.")
        }
        Ok(Self {
            account_dir: None,
            secret_file,
            from_env,
            account_address,
            provider_strategy: AccountProviderStrategy::PrivateKey,
        })
    }

    pub fn get_strategy(&self) -> AccountProviderStrategy {
        debug_assert!(
            (self.account_dir.is_some() as i32)
                + (self.secret_file.is_some() as i32)
                + (self.from_env as i32)
                <= 1
        );
        if self.account_dir.is_some() {
            AccountProviderStrategy::Local
        } else if self.secret_file.is_some() || self.from_env {
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
            from_env: false,
            provider_strategy: AccountProviderStrategy::RPC,
        }
    }
}
