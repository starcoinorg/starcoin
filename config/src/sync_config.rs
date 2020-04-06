use crate::{ChainNetwork, ConfigModule};
use serde::{Deserialize, Serialize};

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

impl ConfigModule for SyncConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        SyncConfig {
            sync_mode: SyncMode::FULL,
        }
    }
}
