// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
}

impl OnChainConfig for Version {
    const IDENTIFIER: &'static str = "Version";
}
