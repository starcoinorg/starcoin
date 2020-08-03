// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::{bail, format_err, Result};
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    sync_mode: SyncMode,
}

impl SyncConfig {
    pub fn is_state_sync(&self) -> bool {
        self.sync_mode == SyncMode::FAST
    }

    pub fn is_light(&self) -> bool {
        self.sync_mode == SyncMode::LIGHT
    }

    pub fn fast_sync_mode(&mut self) {
        self.sync_mode = SyncMode::FAST;
    }

    pub fn full_sync_mode(&mut self) {
        self.sync_mode = SyncMode::FULL;
    }
}

impl ConfigModule for SyncConfig {
    fn default_with_opt(opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        let sync_mode = opt.sync_mode.unwrap_or_else(|| SyncMode::FULL);
        Ok(SyncConfig { sync_mode })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        if let Some(sync_mode) = opt.sync_mode {
            self.sync_mode = sync_mode;
        }
        if self.sync_mode == SyncMode::LIGHT {
            bail!("{} is not supported yet.", self.sync_mode);
        }
        info!("sync mode : {:?} : {:?}", opt.sync_mode, self.sync_mode);
        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SyncMode {
    LIGHT,
    FAST,
    FULL,
}

impl FromStr for SyncMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "light" => Ok(SyncMode::LIGHT),
            "fast" => Ok(SyncMode::FAST),
            "full" => Ok(SyncMode::FULL),
            _ => Err(format_err!("")),
        }
    }
}

impl Display for SyncMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::LIGHT => write!(f, "light"),
            SyncMode::FAST => write!(f, "fast"),
            SyncMode::FULL => write!(f, "full"),
        }
    }
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::FULL
    }
}
