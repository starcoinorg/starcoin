use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_account_api::AccountProviderStrategy;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct AccountProviderConfig {
    /// Path to the local account provider dir, load the accounts from local dir path
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "local-account-dir", parse(from_os_str))]
    pub account_dir: Option<PathBuf>,
    #[serde(skip)]
    #[structopt(skip)]
    provider_strategy: AccountProviderStrategy,
}

impl ConfigModule for AccountProviderConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        if opt.account_provider.account_dir.is_some() {
            self.account_dir = opt.account_provider.account_dir.clone()
        }
        if self.account_dir.is_some() {
            self.provider_strategy = AccountProviderStrategy::Local
        } else {
            self.provider_strategy = AccountProviderStrategy::RPC
        }
        Ok(())
    }
}

impl AccountProviderConfig {
    pub fn get_strategy(&self) -> AccountProviderStrategy {
        if self.account_dir.is_some() {
            AccountProviderStrategy::Local
        } else {
            AccountProviderStrategy::RPC
        }
    }
}

impl Default for AccountProviderConfig {
    fn default() -> Self {
        Self {
            account_dir: None,
            provider_strategy: AccountProviderStrategy::RPC,
        }
    }
}
