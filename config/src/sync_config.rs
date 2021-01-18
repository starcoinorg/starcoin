// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {}

impl SyncConfig {}

impl ConfigModule for SyncConfig {
    fn merge_with_opt(&mut self, _opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        Ok(())
    }
}
