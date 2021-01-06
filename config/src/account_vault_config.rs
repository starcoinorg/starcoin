// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct AccountVaultConfig {
    #[structopt(long = "vault-dir", parse(from_os_str), conflicts_with("vault-dir"))]
    /// Account vault dir config.
    dir: PathBuf,
    #[serde(skip)]
    absolute_dir: Option<PathBuf>,
}

impl AccountVaultConfig {
    pub fn dir(&self) -> PathBuf {
        self.absolute_dir
            .as_ref()
            .cloned()
            .expect("config should init first.")
    }
}

impl ConfigModule for AccountVaultConfig {
    fn default_with_opt(_opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        Ok(Self {
            dir: PathBuf::from("account_vaults"),
            absolute_dir: None,
        })
    }

    fn after_load(&mut self, _opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.absolute_dir = Some(if self.dir.is_relative() {
            base.data_dir().join(&self.dir)
        } else {
            self.dir.clone()
        });
        Ok(())
    }
}
