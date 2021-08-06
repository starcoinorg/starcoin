// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use crate::{
    account_config::{
        constants::ACCOUNT_MODULE_NAME, KeyRotationCapabilityResource, WithdrawCapabilityResource,
    },
    event::EventHandle,
};
use serde::{Deserialize, Serialize};

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResource {
    authentication_key: Vec<u8>,
    withdrawal_capability: Option<WithdrawCapabilityResource>,
    key_rotation_capability: Option<KeyRotationCapabilityResource>,
    withdraw_events: EventHandle,
    deposit_events: EventHandle,
    accept_token_events: EventHandle,
    sequence_number: u64,
}

impl AccountResource {
    pub const DUMMY_AUTH_KEY: [u8; 32] = [0; 32];
    pub const CONTRACT_AUTH_KEY: [u8; 32] = {
        let mut k = [0u8; 32];
        k[31] = 1u8;
        k
    };
    /// Constructs an Account resource.
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        withdrawal_capability: Option<WithdrawCapabilityResource>,
        key_rotation_capability: Option<KeyRotationCapabilityResource>,
        deposit_events: EventHandle,
        withdraw_events: EventHandle,
        accept_token_events: EventHandle,
    ) -> Self {
        AccountResource {
            sequence_number,
            withdrawal_capability,
            key_rotation_capability,
            authentication_key,
            deposit_events,
            withdraw_events,
            accept_token_events,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Returns if this account has delegated its withdrawal capability
    pub fn has_delegated_withdrawal_capability(&self) -> bool {
        self.withdrawal_capability.is_none()
    }

    /// Returns if this account has delegated its key rotation capability
    pub fn has_delegated_key_rotation_capability(&self) -> bool {
        self.key_rotation_capability.is_none()
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
        &self.authentication_key
    }

    /// Return the deposit_events handle for the given AccountResource
    pub fn deposit_events(&self) -> &EventHandle {
        &self.deposit_events
    }

    /// Return the withdraw_events handle for the given AccountResource
    pub fn withdraw_events(&self) -> &EventHandle {
        &self.withdraw_events
    }

    /// Return the accept_token_events handle for the given AccountResource
    pub fn accept_token_events(&self) -> &EventHandle {
        &self.accept_token_events
    }
}

impl MoveResource for AccountResource {
    const MODULE_NAME: &'static str = ACCOUNT_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Account";
}
