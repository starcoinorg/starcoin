// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{BaseConfig, ConfigModule, StarcoinOpt};
use anyhow::Result;
use network_api::PeerStrategy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct SyncConfig {
    #[structopt(
        name = "peer-select-strategy",
        long,
        help = "peer select strategy, default random."
    )]
    peer_select_strategy: Option<PeerStrategy>,
}

impl SyncConfig {
    pub fn peer_select_strategy(&self) -> PeerStrategy {
        match &self.peer_select_strategy {
            None => PeerStrategy::default(),
            Some(strategy) => strategy.clone(),
        }
    }
}

impl ConfigModule for SyncConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        if opt.sync.peer_select_strategy.is_some() {
            self.peer_select_strategy = opt.sync.peer_select_strategy.clone();
        }

        Ok(())
    }
}
