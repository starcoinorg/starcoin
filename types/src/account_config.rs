// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath, account_address::AccountAddress, language_storage::StructTag,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use scs::SCSCodec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::HashValue;
use std::convert::{TryFrom, TryInto};

//TODO rename account and coin name.
// Starcoin
static COIN_MODULE_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("LibraCoin").unwrap());
static COIN_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

// Account
static ACCOUNT_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraAccount").unwrap());
static ACCOUNT_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());
static ACCOUNT_BALANCE_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("Balance").unwrap());
// Payment Events
static SENT_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("SentPaymentEvent").unwrap());
static RECEIVED_EVENT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ReceivedPaymentEvent").unwrap());

/// Path to the Account resource.
/// It can be used to create an AccessPath for an Account resource.
pub static ACCOUNT_RESOURCE_PATH: Lazy<HashValue> =
    Lazy::new(|| AccessPath::resource_access_vec(&account_struct_tag()));

/// Path to the Balance resource
pub static BALANCE_RESOURCE_PATH: Lazy<HashValue> = Lazy::new(||
    AccessPath::resource_access_vec(&account_balance_struct_tag()));

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

pub fn account_balance_struct_name() -> &'static IdentStr {
    &*ACCOUNT_BALANCE_STRUCT_NAME
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

pub fn mint_address() -> AccountAddress {
    AccountAddress::from_hex_literal("0x6d696e74")
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

pub fn account_balance_struct_tag() -> StructTag {
    StructTag {
        address: core_code_address(),
        module: account_module_name().to_owned(),
        name: account_balance_struct_name().to_owned(),
        type_params: vec![lbr_type_tag()],
    }
}

pub fn lbr_type_tag() -> libra_types::language_storage::TypeTag {
    libra_types::account_config::lbr_type_tag()
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
#[derive(Debug)]
pub struct AccountResource(libra_types::account_config::AccountResource);

impl Serialize for AccountResource {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AccountResource {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(AccountResource(
            libra_types::account_config::AccountResource::deserialize(deserializer)?,
        ))
    }
}

impl AccountResource {
    /// Constructs an Account resource.
    pub fn new(balance: u64, sequence_number: u64, authentication_key: Vec<u8>) -> Self {
        AccountResource(libra_types::account_config::AccountResource::new(
            sequence_number,
            authentication_key,
            false,
            false,
            //TODO eventKey as arguemnt.
            libra_types::event::EventHandle::new(
                libra_types::event::EventKey::new_from_address(
                    &libra_types::account_address::AccountAddress::DEFAULT,
                    0,
                ),
                0,
            ),
            libra_types::event::EventHandle::new(
                libra_types::event::EventKey::new_from_address(
                    &libra_types::account_address::AccountAddress::DEFAULT,
                    1,
                ),
                0,
            ),
            0,
        ))
    }

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    //TODO remove
    pub fn make_from_starcoin_blob(bytes: &[u8]) -> Result<Self> {
        Self::decode(bytes)
    }

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    pub fn make_from(bytes: &[u8]) -> Result<Self> {
        Self::decode(bytes)
    }

    /// Return the sequence_number field for the given AccountResource
    pub fn sequence_number(&self) -> u64 {
        self.0.sequence_number()
    }

//    /// Return the balance field for the given AccountResource
//    pub fn balance(&self) -> u64 {
//        self.0.balance()
//    }

    /// Return the authentication_key field for the given AccountResource
    pub fn authentication_key(&self) -> &[u8] {
        self.0.authentication_key()
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
        AccountResource::make_from_starcoin_blob(value.as_slice())
    }
}

impl Into<libra_types::account_config::AccountResource> for AccountResource {
    fn into(self) -> libra_types::account_config::AccountResource {
        self.0
    }
}

impl From<libra_types::account_config::AccountResource> for AccountResource {
    fn from(libra_account_res: libra_types::account_config::AccountResource) -> Self {
        AccountResource(libra_account_res)
    }
}

/// The balance resource held under an account.
#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResource {
    coin: u64,
}

impl BalanceResource {
    pub fn new(coin: u64) -> Self {
        Self { coin }
    }

    pub fn coin(&self) -> u64 {
        self.coin
    }

    /// Given an account map (typically from storage) retrieves the Account resource associated.
    pub fn make_from(bytes: &[u8]) -> Result<Self> {
        Self::decode(bytes)
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
            0,
        );
        let receive_event_handle = libra_types::event::EventHandle::new(
            libra_types::event::EventKey::new_from_address(&address, 1),
            0,
        );
        let account_res = libra_types::account_config::AccountResource::new(
            1,
            address.to_vec(),
            false,
            false,
            send_event_handle,
            receive_event_handle,
            0,
        );
        let account_res1: AccountResource = AccountResource::from(account_res);
        assert_eq!(account_res1.balance(), 0);
        assert_eq!(
            account_res1.authentication_key().len(),
            address.to_vec().len()
        );
    }
}
