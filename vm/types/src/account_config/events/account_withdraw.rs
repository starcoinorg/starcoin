// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::ACCOUNT_MODULE_NAME;
use crate::token::token_code::TokenCode;
use anyhow::Result;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

/// Struct that represents a SentPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawEvent {
    amount: u128,
    token_code: TokenCode,
    metadata: Vec<u8>,
}

impl WithdrawEvent {
    pub fn new(amount: u128, token_code: TokenCode, metadata: Vec<u8>) -> Self {
        Self {
            amount,
            token_code,
            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u128 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }

    /// Return the token_code symbol that the payment was made in.
    pub fn token_code(&self) -> &TokenCode {
        &self.token_code
    }
}

impl MoveStructType for WithdrawEvent {
    const MODULE_NAME: &'static IdentStr = ident_str!(ACCOUNT_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!("WithdrawEvent");
}

impl MoveResource for WithdrawEvent {}
