// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

use crate::{
    access_path::AccessPath,
    account_config::constants::{stc_type_tag, CORE_CODE_ADDRESS},
};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};

const CONSENSUS_MODULE_NAME: &str = "Consensus";

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Consensus {
    pub uncle_rate_target: u64,
    pub epoch_time_target: u64,
    pub reward_half_time_target: u64,
}

impl OnChainConfig for Consensus {
    const IDENTIFIER: &'static str = CONSENSUS_MODULE_NAME;
}

/// The Epoch resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct EpochResource {
    epoch_start_time: u64,
    uncles: u64,
    start_number: u64,
    end_number: u64,
    time_target: u64,
    reward: u64,
}

impl EpochResource {
    pub fn new(
        epoch_start_time: u64,
        uncles: u64,
        start_number: u64,
        end_number: u64,
        time_target: u64,
        reward: u64,
    ) -> Self {
        Self {
            epoch_start_time,
            uncles,
            start_number,
            end_number,
            time_target,
            reward,
        }
    }

    pub fn start_number(&self) -> u64 {
        self.start_number
    }
    // TODO/XXX: remove this once the MoveResource trait allows type arguments to `struct_tag`.
    pub fn struct_tag_for_epoch(currency_typetag: TypeTag) -> StructTag {
        StructTag {
            address: CORE_CODE_ADDRESS,
            name: EpochResource::struct_identifier(),
            module: EpochResource::module_identifier(),
            type_params: vec![currency_typetag],
        }
    }

    // TODO: remove this once the MoveResource trait allows type arguments to `resource_path`.
    pub fn access_path_for(epoch_typetag: TypeTag) -> Vec<u8> {
        AccessPath::resource_access_vec(&EpochResource::struct_tag_for_epoch(epoch_typetag))
    }
}

impl MoveResource for EpochResource {
    const MODULE_NAME: &'static str = CONSENSUS_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Epoch";

    fn type_params() -> Vec<TypeTag> {
        vec![stc_type_tag()]
    }
}
