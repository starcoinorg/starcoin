// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

static G_DEFAULT_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("account_vaults"));

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, Parser)]
#[serde(deny_unknown_fields)]
pub struct AccountVaultConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(long = "vault-dir", parse(from_os_str))]
    /// Account vault dir config.
    /// Default: account_vaults in data_dir
    dir: Option<PathBuf>,

    #[serde(skip)]
    #[clap(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl AccountVaultConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn dir(&self) -> PathBuf {
        let path = self.dir.as_ref().unwrap_or(&G_DEFAULT_DIR);
        if path.is_absolute() {
            path.clone()
        } else {
            self.base().data_dir().join(path)
        }
    }

    // "/path/to/account_vaults -> /path/to/account_vaults2"
    // "/path/to/account_vaults/ -> /path/to/account_vaults2"
    pub fn dir2(&self) -> PathBuf {
        let mut dir = self.dir();
        let last = dir
            .file_name()
            .expect("account dir should be set properly")
            .to_string_lossy();
        dir.set_file_name(format!("{last}2"));
        dir
    }
}

impl ConfigModule for AccountVaultConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        if opt.vault.dir.is_some() {
            self.dir = opt.vault.dir.clone();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::account_vault_config::AccountVaultConfig;
    use crate::{BaseConfig, ChainNetwork};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_dir2() {
        use super::*;
        let config = AccountVaultConfig {
            dir: Some(PathBuf::from("/path/to/account_vaults")),
            base: None,
        };
        assert_eq!(config.dir2(), PathBuf::from("/path/to/account_vaults2"));

        let config = AccountVaultConfig {
            dir: Some(PathBuf::from("/path/to/account_vaults/")),
            base: None,
        };
        assert_eq!(config.dir2(), PathBuf::from("/path/to/account_vaults2"));
    }
    #[test]
    #[should_panic(expected = "account dir should be set properly")]
    fn test_dir2_panic1() {
        let config = AccountVaultConfig {
            dir: Some(PathBuf::new()),
            base: Some(Arc::new(BaseConfig {
                net: ChainNetwork::new_test(),
                base_data_dir: Default::default(),
                data_dir: PathBuf::from("/"),
            })),
        };
        let _ = config.dir2();
    }

    #[test]
    #[should_panic(expected = "account dir should be set properly")]
    fn test_dir2_panic2() {
        let config = AccountVaultConfig {
            dir: Some(PathBuf::from("/")),
            base: None,
        };
        let _ = config.dir2();
    }
}
