// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{
    change_set_configs::ChangeSetConfigs,

};
use starcoin_gas_meter::{StarcoinGasParameters, LATEST_GAS_FEATURE_VERSION};
use starcoin_vm_types::on_chain_config::ConfigStorage;
use move_core_types::gas_algebra::NumBytes;
use std::fmt::Debug;

pub mod change_set_configs;

#[derive(Clone, Debug)]
pub struct StorageGasParameters {
    pub change_set_configs: ChangeSetConfigs,
}

impl StorageGasParameters {
    pub fn new(
        feature_version: u64,
        gas_params: &StarcoinGasParameters,
        config_storage: &impl ConfigStorage,
    ) -> Self {
        let change_set_configs = ChangeSetConfigs::new(feature_version, gas_params);

        Self {
            change_set_configs,
        }
    }

    pub fn unlimited(free_write_bytes_quota: NumBytes) -> Self {
        Self {
            change_set_configs: ChangeSetConfigs::unlimited_at_gas_feature_version(
                LATEST_GAS_FEATURE_VERSION,
            ),
        }
    }
}
