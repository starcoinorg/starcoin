// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
};

use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, HashValue};
use move_core_types::identifier::Identifier as LibraIdentifier;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHash)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    ByteArray,
    Address,
    Struct(StructTag),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHash)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTag>,
}

/// Represents the intitial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHash)]
pub struct ResourceKey {
    address: AccountAddress,
    type_: StructTag,
}

impl ResourceKey {
    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn type_(&self) -> &StructTag {
        &self.type_
    }
}

impl ResourceKey {
    pub fn new(address: AccountAddress, type_: StructTag) -> Self {
        ResourceKey { address, type_ }
    }
}

/// Represents the initial key into global storage where we first index by the address, and then
/// the struct tag
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, CryptoHash)]
pub struct ModuleId {
    address: AccountAddress,
    name: Identifier,
}

impl ModuleId {
    pub fn new(address: AccountAddress, name: Identifier) -> Self {
        ModuleId { address, name }
    }

    pub fn name(&self) -> &IdentStr {
        &self.name
    }

    pub fn name_hash(&self) -> HashValue {
        self.name.crypto_hash()
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn into_inner(&self) -> (AccountAddress, Identifier) {
        (self.address, self.name.clone())
    }
}

impl Into<libra_types::language_storage::ModuleId> for ModuleId {
    fn into(self) -> libra_types::language_storage::ModuleId {
        libra_types::language_storage::ModuleId::new(self.address().into(), LibraIdentifier::from_utf8(self.name.into_bytes()).unwrap())
    }
}

impl From<libra_types::language_storage::ModuleId> for ModuleId {
    fn from(module_id: libra_types::language_storage::ModuleId) -> Self {
        Self::new(AccountAddress::from(module_id.address().clone()), Identifier::new(module_id.name().as_str()).unwrap())
    }
}
