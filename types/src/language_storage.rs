// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;

use move_core_types::identifier::{IdentStr, Identifier};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, HashValue, LibraCryptoHash};

//pub use libra_types::language_storage::TypeTag;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub enum TypeTag {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<TypeTag>),
    Struct(StructTag),
}

impl Into<libra_types::language_storage::TypeTag> for TypeTag {
    fn into(self) -> libra_types::language_storage::TypeTag {
        match self {
            TypeTag::Bool => libra_types::language_storage::TypeTag::Bool,
            TypeTag::U8 => libra_types::language_storage::TypeTag::U8,
            TypeTag::U64 => libra_types::language_storage::TypeTag::U64,
            TypeTag::U128 => libra_types::language_storage::TypeTag::U128,
            TypeTag::Address => libra_types::language_storage::TypeTag::Address,
            TypeTag::Vector(v) => {
                libra_types::language_storage::TypeTag::Vector(Box::new(v.as_ref().clone().into()))
            }
            TypeTag::Struct(s) => libra_types::language_storage::TypeTag::Struct(s.into()),
        }
    }
}

impl From<libra_types::language_storage::TypeTag> for TypeTag {
    fn from(type_tag: libra_types::language_storage::TypeTag) -> Self {
        match type_tag {
            libra_types::language_storage::TypeTag::Bool => TypeTag::Bool,
            libra_types::language_storage::TypeTag::U8 => TypeTag::U8,
            libra_types::language_storage::TypeTag::U64 => TypeTag::U64,
            libra_types::language_storage::TypeTag::U128 => TypeTag::U128,
            libra_types::language_storage::TypeTag::Address => TypeTag::Address,
            libra_types::language_storage::TypeTag::Vector(v) => {
                TypeTag::Vector(Box::new(v.as_ref().clone().into()))
            }
            libra_types::language_storage::TypeTag::Struct(s) => TypeTag::Struct(s.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct StructTag {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub type_params: Vec<TypeTag>,
}

impl Into<libra_types::language_storage::StructTag> for StructTag {
    fn into(self) -> libra_types::language_storage::StructTag {
        libra_types::language_storage::StructTag {
            address: self.address.into(),
            module: self.module,
            name: self.name,
            type_params: self.type_params.into_iter().map(|tag| tag.into()).collect(),
        }
    }
}

impl From<libra_types::language_storage::StructTag> for StructTag {
    fn from(struct_tag: libra_types::language_storage::StructTag) -> Self {
        Self {
            address: struct_tag.address.into(),
            module: struct_tag.module,
            name: struct_tag.name,
            type_params: struct_tag
                .type_params
                .into_iter()
                .map(|tag| tag.into())
                .collect(),
        }
    }
}

impl CryptoHash for StructTag {
    fn crypto_hash(&self) -> HashValue {
        //TODO fixme.
        //Use libra CryptoHash temporarily
        let struct_tag: libra_types::language_storage::StructTag = self.clone().into();
        LibraCryptoHash::hash(&struct_tag)
    }
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
        HashValue::from_sha3_256(self.name.as_bytes())
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn into_inner(self) -> (AccountAddress, Identifier) {
        (self.address, self.name)
    }
}

impl Into<libra_types::language_storage::ModuleId> for ModuleId {
    fn into(self) -> libra_types::language_storage::ModuleId {
        libra_types::language_storage::ModuleId::new(self.address().into(), self.name)
    }
}

impl From<libra_types::language_storage::ModuleId> for ModuleId {
    fn from(module_id: libra_types::language_storage::ModuleId) -> Self {
        Self::new(
            module_id.address().clone().into(),
            Identifier::from(module_id.name()),
        )
    }
}
