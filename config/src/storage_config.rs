// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StorageConfig {
    dir: PathBuf,
    absolute_dir: Option<PathBuf>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

impl StorageConfig {
    pub fn dir(&self) -> PathBuf {
        self.absolute_dir
            .as_ref()
            .cloned()
            .expect("config should init first.")
    }
}

impl ConfigModule for StorageConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        Self {
            dir: PathBuf::from("starcoindb/db"),
            absolute_dir: None,
        }
    }

    fn random(&mut self, base: &BaseConfig) {
        self.absolute_dir = Some(base.data_dir().join(self.dir.as_path()));
    }

    fn load(&mut self, base: &BaseConfig, _opt: &StarcoinOpt) -> Result<()> {
        self.absolute_dir = Some(if self.dir.is_relative() {
            base.data_dir().join(&self.dir)
        } else {
            self.dir.clone()
        });
        Ok(())
    }
}
