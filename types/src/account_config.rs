// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath, account_address::AccountAddress, byte_array::ByteArray,
    language_storage::StructTag,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::convert::{TryFrom, TryInto};
use logger::prelude::*;


//TODO rename account and coin name.
// Starcoin
static COIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("LibraCoin").unwrap());
static COIN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

// Account
static ACCOUNT_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraAccount").unwrap());
static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

// Payment Events
static SENT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("SentPaymentEvent").unwrap());
static RECEIVED_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ReceivedPaymentEvent").unwrap());

/// Path to the Account resource.
/// It can be used to create an AccessPath for an Account resource.
pub static ACCOUNT_RESOURCE_PATH: Lazy<HashValue> =
    Lazy::new(|| AccessPath::resource_access_vec(&account_struct_tag()));

pub fn coin_module_name() -> &'static IdentStr {
    &*COIN_MODULE_NAME
}

pub fn coin_struct_name() -> &'static IdentStr {
    &*COIN_STRUCT_NAME
}

pub fn account_module_name() -> &'static IdentStr {
    &*ACCOUNT_MODULE_NAME
}

pub fn account_struct_name() -> &'static IdentStr {
    &*ACCOUNT_STRUCT_NAME
}

pub fn sent_event_name() -> &'static IdentStr {
    &*SENT_EVENT_NAME
}

pub fn received_event_name() -> &'static IdentStr {
    &*RECEIVED_EVENT_NAME
}

pub fn core_code_address() -> AccountAddress {
    AccountAddress::default()
}

pub fn association_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xA550C18")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn transaction_fee_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0xFEE")
        .expect("Parsing valid hex literal should always succeed")
}

pub fn account_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: account_struct_name().to_owned(),
        type_params: vec![],
    }
}

pub fn sent_payment_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: sent_event_name().to_owned(),
        type_params: vec![],
    }
}

pub fn received_payment_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: received_event_name().to_owned(),
        type_params: vec![],
    }
}

/// A Rust representation of an Account resource.
/// This is not how the Account is represented in the VM but it's a convenient representation.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountResource {
    authentication_key: ByteArray,
    balance: u64,
    sequence_number: u64,
}

impl AccountResource {
    /// Constructs an Account resource.
    pub fn new(balance: u64, sequence_number: u64, authentication_key: ByteArray) -> Self {
        AccountResource {
            authentication_key,
            balance,
            sequence_number,
        }
    }

    pub fn new_by_address(balance: u64, sequence_number: u64, address: AccountAddress) -> Self {
        AccountResource {
            authentication_key: ByteArray::new(address.to_vec()),
            balance,
            sequence_number,
        }
    }

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    pub fn make_from(bytes: &[u8]) -> Result<Self> {
        // make from libra data blob
        let libra_account_res = libra_types::account_config::AccountResource::decode(bytes)?;
        Ok(AccountResource::from(libra_account_res))
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    /// Return the balance field for the given AccountResource
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &ByteArray {
        &self.authentication_key
    }
}

impl TryInto<Vec<u8>> for AccountResource {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

impl TryFrom<Vec<u8>> for AccountResource {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        AccountResource::make_from(value.as_slice())
    }
}

impl Into<libra_types::account_config::AccountResource> for AccountResource {
    fn into(self) -> libra_types::account_config::AccountResource {
        unimplemented!()
    }
}

impl From<libra_types::account_config::AccountResource> for AccountResource {
    fn from(libra_account_res: libra_types::account_config::AccountResource) -> Self {
        AccountResource::new(
            libra_account_res.balance(),
            libra_account_res.sequence_number(),
            ByteArray::new(libra_account_res.authentication_key().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_account_res() {
        let address = libra_types::account_address::AccountAddress::random();
        let send_event_handle = libra_types::event::EventHandle::new(
            libra_types::event::EventKey::new_from_address(&address, 0),
            0
        );
        let receive_event_handle = libra_types::event::EventHandle::new(
            libra_types::event::EventKey::new_from_address(&address, 1),
            0
        );
        let account_res = libra_types::account_config::AccountResource::new(
            0,
            1,
            address.to_vec(),
            false,
            false,
            send_event_handle,
            receive_event_handle,
            0
        );
        let account_res1: AccountResource = AccountResource::from(account_res);
        assert_eq!(account_res1.balance(), 0);
        assert_eq!(account_res1.authentication_key().len(), address.to_vec().len());
    }
}
