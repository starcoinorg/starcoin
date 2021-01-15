// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

static DEFAULT_DIR: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("account_vaults"));

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct AccountVaultConfig {
    #[structopt(long = "vault-dir", parse(from_os_str))]
    /// Account vault dir config.
    /// Default: account_vaults in data_dir
    dir: Option<PathBuf>,

    #[serde(skip)]
    #[structopt(skip)]
    base: Option<Arc<BaseConfig>>,
}

impl AccountVaultConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn dir(&self) -> PathBuf {
        let path = self.dir.as_ref().unwrap_or(&DEFAULT_DIR);
        if path.is_absolute() {
            path.clone()
        } else {
            self.base().data_dir().join(path)
        }
    }
}

impl ConfigModule for AccountVaultConfig {
    fn merge_with_opt(&mut self, _opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);
        Ok(())
    }
}
