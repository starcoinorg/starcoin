use crate::{BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::{format_err, Result};
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SyncConfig {
    sync_mode: SyncMode,
}

impl SyncConfig {
    pub fn is_state_sync(&self) -> bool {
        self.sync_mode == SyncMode::FAST_SYNC
    }

    pub fn is_light(&self) -> bool {
        self.sync_mode == SyncMode::LIGHT
    }

    //just for test
    pub fn fast_sync_mode(&mut self) {
        self.sync_mode = SyncMode::FAST_SYNC;
    }
}

impl ConfigModule for SyncConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        SyncConfig {
            sync_mode: SyncMode::FAST_SYNC,
        }
    }

    fn load(&mut self, _base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        info!("sync_mode : {:?}", opt.sync_mode);
        self.sync_mode = opt.sync_mode.clone();
        Ok(())
    }
}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig::default_with_net(ChainNetwork::default())
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SyncMode {
    LIGHT,
    FAST_SYNC,
    FULL,
}

impl FromStr for SyncMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "light" => Ok(SyncMode::LIGHT),
            "fast" => Ok(SyncMode::FAST_SYNC),
            "full" => Ok(SyncMode::FULL),
            _ => Err(format_err!("")),
        }
    }
}

impl Display for SyncMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::LIGHT => write!(f, "light"),
            SyncMode::FAST_SYNC => write!(f, "fast"),
            SyncMode::FULL => write!(f, "full"),
        }
    }
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::FULL
    }
}
