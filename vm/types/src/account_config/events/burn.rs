// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::TOKEN_MODULE_NAME;
use anyhow::Result;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveResource,
};
use serde::{Deserialize, Serialize};

/// Struct that represents a BurnEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct BurnEvent {
    amount: u128,
    token_code: Identifier,
}

impl BurnEvent {
    /// Get the amount burned
    pub fn amount(&self) -> u128 {
        self.amount
    }

    /// Return the code for the currency that was burned
    pub fn token_code(&self) -> &IdentStr {
        &self.token_code
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        scs::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for BurnEvent {
    const MODULE_NAME: &'static str = TOKEN_MODULE_NAME;
    const STRUCT_NAME: &'static str = "BurnEvent";
}
