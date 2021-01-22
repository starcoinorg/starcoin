// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::TOKEN_MODULE_NAME;
use crate::move_resource::MoveResource;
use crate::token::token_code::TokenCode;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Struct that represents a MintEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct MintEvent {
    amount: u128,
    token_code: TokenCode,
}

impl MintEvent {
    /// Get the amount minted
    pub fn amount(&self) -> u128 {
        self.amount
    }

    /// Return the code for the currency that was minted
    pub fn token_code(&self) -> &TokenCode {
        &self.token_code
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }
}

impl MoveResource for MintEvent {
    const MODULE_NAME: &'static str = TOKEN_MODULE_NAME;
    const STRUCT_NAME: &'static str = "MintEvent";
}
