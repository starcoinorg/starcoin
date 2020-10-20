// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::{constants::ACCOUNT_MODULE_NAME, resources::AccountResource};
use anyhow::Result;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveResource,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Returns the path to the deposit for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCOUNT_DEPOSIT_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = AccountResource::resource_path();
    path.extend_from_slice(b"/deposit_events_count/");
    path
});

/// Struct that represents a ReceivedPaymentEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct DepositEvent {
    amount: u128,
    token_code: Identifier,
    metadata: Vec<u8>,
}

impl DepositEvent {
    pub fn new(amount: u128, token_code: Identifier, metadata: Vec<u8>) -> Self {
        Self {
            amount,
            token_code,

            metadata,
        }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        scs::from_bytes(bytes).map_err(Into::into)
    }

    /// Get the amount sent or received
    pub fn amount(&self) -> u128 {
        self.amount
    }

    /// Get the metadata associated with this event
    pub fn metadata(&self) -> &Vec<u8> {
        &self.metadata
    }

    /// Return the currency code that the payment was made in.
    pub fn currency_code(&self) -> &IdentStr {
        &self.token_code
    }
}

impl MoveResource for DepositEvent {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "DepositEvent";
}
