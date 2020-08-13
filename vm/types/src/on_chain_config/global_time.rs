// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

use move_core_types::move_resource::MoveResource;

const TIMESTAMP_MODULE_NAME: &str = "Timestamp";

/// The Timestamp module.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TimestampOnChain;

impl OnChainConfig for TimestampOnChain {
    const IDENTIFIER: &'static str = TIMESTAMP_MODULE_NAME;
}

/// The CurrentTimeSeconds on chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GlobalTimeOnChain {
    pub seconds: u64,
}

impl MoveResource for GlobalTimeOnChain {
    const MODULE_NAME: &'static str = TIMESTAMP_MODULE_NAME;
    const STRUCT_NAME: &'static str = "CurrentTimeSeconds";
}
