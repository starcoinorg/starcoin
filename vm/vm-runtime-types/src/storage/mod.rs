// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::storage::change_set_configs::ChangeSetConfigs;
use move_core_types::gas_algebra::NumBytes;
use starcoin_gas_meter::StarcoinGasParameters;
use starcoin_gas_schedule::LATEST_GAS_FEATURE_VERSION;
use starcoin_vm_types::on_chain_config::ConfigStorage;
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
        _config_storage: &impl ConfigStorage,
    ) -> Self {
        let change_set_configs = ChangeSetConfigs::new(feature_version, gas_params);

        Self { change_set_configs }
    }

    pub fn unlimited(_free_write_bytes_quota: NumBytes) -> Self {
        Self {
            change_set_configs: ChangeSetConfigs::unlimited_at_gas_feature_version(
                LATEST_GAS_FEATURE_VERSION,
            ),
        }
    }
}
