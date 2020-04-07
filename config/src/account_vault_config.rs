use crate::{BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AccountVaultConfig {
    dir: PathBuf,
    #[serde(skip)]
    absolute_dir: Option<PathBuf>,
}

impl Default for AccountVaultConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
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
    fn default_with_net(_net: ChainNetwork) -> Self {
        Self {
            dir: PathBuf::from("account_vaults"),
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
