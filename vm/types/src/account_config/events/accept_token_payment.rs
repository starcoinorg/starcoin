// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::{constants::ACCOUNT_MODULE_NAME, resources::AccountResource};
use crate::contract_event::ContractEvent;
use crate::language_storage::TypeTag;
use anyhow::{Error, Result};
use move_core_types::move_resource::MoveResource;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The path to the accept token event counter for an Account resource.
/// It can be used to query the event DB for the given event.
pub static ACCEPT_TOKEN_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = AccountResource::resource_path();
    path.extend_from_slice(b"/accept_token_events_count/");
    path
});

/// Struct that represents a AcceptTokenEvent.
#[derive(Debug, Serialize, Deserialize)]
pub struct AcceptTokenEvent {
    token_code: Vec<u8>,
}

impl AcceptTokenEvent {
    pub fn new(token_code: Vec<u8>) -> Self {
        Self { token_code }
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        scs::from_bytes(bytes).map_err(Into::into)
    }

    /// Return the currency currency_code symbol that the payment was made in.
    pub fn currency_code(&self) -> &[u8] {
        &self.token_code
    }
}

impl MoveResource for AcceptTokenEvent {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "AcceptTokenEvent";
}

impl TryFrom<&ContractEvent> for AcceptTokenEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag() != &TypeTag::Struct(AcceptTokenEvent::struct_tag()) {
            anyhow::bail!("Expected AcceptTokenEvent")
        }
        Self::try_from_bytes(event.event_data())
    }
}
