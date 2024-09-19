// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::ACCOUNT_MODULE_NAME;
use crate::token::token_code::TokenCode;
use anyhow::Result;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::MoveStructType;
use serde::{Deserialize, Serialize};

/// Struct that represents a AcceptTokenEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptTokenEvent {
    token_code: TokenCode,
}

impl AcceptTokenEvent {
    pub fn new(token_code: TokenCode) -> Self {
        Self { token_code }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }

    /// Return the token code symbol that the payment was made in.
    pub fn token_code(&self) -> &TokenCode {
        &self.token_code
    }
}

impl MoveStructType for AcceptTokenEvent {
    const STRUCT_NAME: &'static IdentStr = ident_str!("AcceptTokenEvent");
    const MODULE_NAME: &'static IdentStr = ident_str!(ACCOUNT_MODULE_NAME);
}
