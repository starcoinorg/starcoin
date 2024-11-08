// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_config::constants::ACCOUNT_MODULE_NAME, event::EventHandle};
use move_core_types::account_address::AccountAddress;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResource {
    authentication_key: Vec<u8>,
    sequence_number: u64,
    guid_creation_num: u64,
    coin_register_events: EventHandle,
    key_rotation_events: EventHandle,
    rotation_capability_offer: Option<AccountAddress>,
    signer_capability_offer: Option<AccountAddress>,
}

impl AccountResource {
    pub const DUMMY_AUTH_KEY: [u8; 32] = [0; 32];
    pub const CONTRACT_AUTH_KEY: [u8; 32] = {
        let mut k = [0u8; 32];
        k[31] = 1u8;
        k
    };
    pub fn new(
        sequence_number: u64,
        authentication_key: Vec<u8>,
        coin_register_events: EventHandle,
        key_rotation_events: EventHandle,
    ) -> Self {
        AccountResource {
            authentication_key,
            sequence_number,
            guid_creation_num: 0,
            coin_register_events,
            key_rotation_events,
            rotation_capability_offer: None,
            signer_capability_offer: None,
        }
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
        &self.authentication_key
    }

    pub fn coin_register_events(&self) -> &EventHandle {
        &self.coin_register_events
    }

    pub fn key_rotation_events(&self) -> &EventHandle {
        &self.key_rotation_events
    }

    pub fn guid_creation_num(&self) -> u64 {
        self.guid_creation_num
    }

    pub fn rotation_capability_offer(&self) -> Option<AccountAddress> {
        self.rotation_capability_offer
    }

    pub fn signer_capability_offer(&self) -> Option<AccountAddress> {
        self.signer_capability_offer
    }

    /// Returns if this account has delegated its withdrawal capability
    pub fn has_delegated_withdrawal_capability(&self) -> bool {
        // TODO(BobOng): [framework-upgrade] to remove this function, this function for compatible with old code
        false
    }

    /// Returns if this account has delegated its key rotation capability
    pub fn has_delegated_key_rotation_capability(&self) -> bool {
        // TODO(BobOng): [framework-upgrade] to remove this function, this function for compatible with old code
        self.rotation_capability_offer.is_none()
    }

    /// Return the deposit_events handle for the given AccountResource
    pub fn deposit_events(&self) -> &EventHandle {
        // TODO(BobOng): [framework-upgrade] to remove this function, this function for compatible with old code
        &self.coin_register_events
    }

    /// Return the withdraw_events handle for the given AccountResource
    pub fn withdraw_events(&self) -> &EventHandle {
        // TODO(BobOng): [framework-upgrade] to remove this function, this function for compatible with old code
        &self.coin_register_events
    }

    /// Return the accept_token_events handle for the given AccountResource
    pub fn accept_token_events(&self) -> &EventHandle {
        // TODO(BobOng): [framework-upgrade] to remove this function, this function for compatible with old code
        &self.coin_register_events
    }
}

impl MoveStructType for AccountResource {
    const MODULE_NAME: &'static IdentStr = ident_str!(ACCOUNT_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!("Account");
}

impl MoveResource for AccountResource {}
