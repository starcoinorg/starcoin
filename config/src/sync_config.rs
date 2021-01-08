// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::{bail, Result};
use clap::arg_enum;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use structopt::StructOpt;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    #[structopt(long, possible_values = &SyncMode::variants(), case_insensitive = false)]
    /// Sync mode:  Light, Fast, Full, eg.
    sync_mode: SyncMode,
}

impl SyncConfig {
    pub fn new(sync_mode: SyncMode) -> Self {
        Self { sync_mode }
    }

    pub fn set_mode(&mut self, sync_mode: SyncMode) {
        self.sync_mode = sync_mode;
    }

    pub fn is_state_sync(&self) -> bool {
        self.sync_mode == SyncMode::Fast
    }

    pub fn is_light(&self) -> bool {
        self.sync_mode == SyncMode::Lignt
    }
}

impl ConfigModule for SyncConfig {
    fn default_with_opt(opt: &StarcoinOpt, _base: &BaseConfig) -> Result<Self> {
        let sync_mode = opt.sync_mode.clone().unwrap_or(SyncMode::Full);
        Ok(SyncConfig { sync_mode })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        if let Some(sync_mode) = opt.sync_mode.clone() {
            self.sync_mode = sync_mode;
        }
        if self.sync_mode == SyncMode::Lignt || self.sync_mode == SyncMode::Fast {
            bail!("{} is not supported yet.", self.sync_mode);
        }
        info!("Sync mode : {:?} : {:?}", opt.sync_mode, self.sync_mode);
        Ok(())
    }
}
//TODO remove SyncMode.
arg_enum! {
#[derive(Debug,Clone, Deserialize, PartialEq, Serialize)]
pub enum SyncMode {
    Lignt,
    Fast,
    Full,
    }
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::Full
    }
}
