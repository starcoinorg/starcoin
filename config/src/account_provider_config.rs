use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::{bail, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountProviderStrategy;
use starcoin_types::account_address::AccountAddress;
use std::path::PathBuf;
use std::sync::Arc;

pub const G_ENV_PRIVATE_KEY: &str = "STARCOIN_PRIVATE_KEY";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Parser)]
pub struct AccountProviderConfig {
    /// Path to the local account provider dir, load the accounts from local dir path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "local-account-dir", parse(from_os_str))]
    pub account_dir: Option<PathBuf>,

    /// Path to the secret file storing the private key.
    #[clap(
        long = "secret-file",
        help = "file path of private key",
        parse(from_os_str)
    )]
    pub secret_file: Option<PathBuf>,

    /// Read private from env variable `STARCOIN_PRIVATE_KEY`.
    #[clap(long = "from-env")]
    pub from_env: bool,

    #[serde(skip)]
    #[clap(skip)]
    pub account_address: Option<AccountAddress>,

    #[serde(skip)]
    #[clap(skip)]
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
            self.account_dir = opt.account_provider.account_dir.clone()
        }
        if opt.account_provider.secret_file.is_some() {
            self.secret_file = opt.account_provider.secret_file.clone();
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
