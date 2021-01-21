// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::constants::ACCOUNT_MODULE_NAME;
use crate::contract_event::ContractEvent;
use crate::language_storage::TypeTag;
use crate::move_resource::MoveResource;
use crate::token::token_code::TokenCode;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
