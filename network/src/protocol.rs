// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::{ Roles};

/// Configuration for the Substrate-specific part of the networking layer.
#[derive(Clone)]
pub struct ProtocolConfig {
    /// Assigned roles.
    pub roles: Roles,
    /// Maximum number of peers to ask the same blocks in parallel.
    pub max_parallel_downloads: u32,
}

impl Default for ProtocolConfig {
    fn default() -> ProtocolConfig {
        ProtocolConfig {
            roles: Roles::FULL,
            max_parallel_downloads: 5,
        }
    }
}