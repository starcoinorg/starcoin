use crate::{get_available_port, BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
}

impl Default for SyncConfig {
    fn default() -> Self {
        SyncConfig::default_with_net(ChainNetwork::default())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum SyncMode {
    LIGHT,
    FAST_SYNC,
    FULL,
}

impl ConfigModule for SyncConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        SyncConfig {
            sync_mode: SyncMode::FAST_SYNC,
        }
    }
}
