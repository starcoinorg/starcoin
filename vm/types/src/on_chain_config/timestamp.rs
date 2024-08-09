// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct CurrentTimeMicroseconds {
    pub microseconds: u64,
}

impl OnChainConfig for CurrentTimeMicroseconds {
    const MODULE_IDENTIFIER: &'static str = "timestamp";
    const CONF_IDENTIFIER: &'static str = "CurrentTimeMicroseconds";
}
