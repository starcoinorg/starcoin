// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Suppose we have the following data structure in a smart contract:
//!
//! struct B {
//!   Map<String, String> mymap;
//! }
//!
//! struct A {
//!   B b;
//!   int my_int;
//! }
//!
//! struct C {
//!   List<int> mylist;
//! }
//!
//! A a;
//! C c;
//!
//! and the data belongs to Alice. Then an access to `a.b.mymap` would be translated to an access
//! to an entry in key-value store whose key is `<Alice>/a/b/mymap`. In the same way, the access to
//! `c.mylist` would need to query `<Alice>/c/mylist`.
//!
//! So an account stores its data in a directory structure, for example:
//!   <Alice>/balance:   10
//!   <Alice>/a/b/mymap: {"Bob" => "abcd", "Carol" => "efgh"}
//!   <Alice>/a/myint:   20
//!   <Alice>/c/mylist:  [3, 5, 7, 9]
//!
//! If someone needs to query the map above and find out what value associated with "Bob" is,
//! `address` will be set to Alice and `path` will be set to "/a/b/mymap/Bob".
//!
//! On the other hand, if you want to query only <Alice>/a/*, `address` will be set to Alice and
//! `path` will be set to "/a" and use the `get_prefix()` method from statedb

use crate::account_address::AccountAddress;
use crate::identifier::Identifier;
use anyhow::Result;
use move_core_types::language_storage::{ModuleId, ResourceKey, StructTag, CODE_TAG, RESOURCE_TAG};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::{Arbitrary, BoxedStrategy};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::distributions::Alphanumeric;
use rand::prelude::{Distribution, SliceRandom};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_helpers::{deserialize_binary, serialize_binary};
use starcoin_crypto::hash::{CryptoHash, HashValue, PlainCryptoHash};
use std::convert::TryFrom;
use std::fmt;
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: DataPath,
}

impl AccessPath {
    pub const CODE_TAG: u8 = 0;
    pub const RESOURCE_TAG: u8 = 1;

    pub fn new(address: AccountAddress, path: DataPath) -> Self {
        AccessPath { address, path }
    }

    pub fn resource_access_path(address: AccountAddress, struct_tag: StructTag) -> Self {
        Self::new(address, Self::resource_data_path(struct_tag))
    }

    pub fn code_access_path(address: AccountAddress, module_name: Identifier) -> AccessPath {
        AccessPath::new(address, Self::code_data_path(module_name))
    }

    pub fn resource_data_path(tag: StructTag) -> DataPath {
        DataPath::Resource(tag)
    }

    pub fn code_data_path(module_name: ModuleName) -> DataPath {
        DataPath::Code(module_name)
    }

    pub fn into_inner(self) -> (AccountAddress, DataPath) {
        let address = self.address;
        let path = self.path;
        (address, path)
    }

    pub fn random_code() -> AccessPath {
        AccessPath::new(AccountAddress::random(), DataPath::Code(random_identity()))
    }

    pub fn random_resource() -> AccessPath {
        let struct_tag = StructTag {
            address: AccountAddress::random(),
            module: random_identity(),
            name: random_identity(),
            type_params: vec![],
        };
        AccessPath::new(AccountAddress::random(), DataPath::Resource(struct_tag))
    }
}

//TODO move to a suitable mod
struct IdentifierSymbols;

impl Distribution<char> for IdentifierSymbols {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        //TODO add more valid identity char
        *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .choose(rng)
            .unwrap() as char
    }
}

fn random_identity() -> Identifier {
    let mut rng = OsRng;
    let id: String = rng.sample_iter(&IdentifierSymbols).take(7).collect();
    Identifier::new(id).unwrap()
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccessPath {{ address: {:x}, path: {} }}",
            self.address, self.path
        )
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.path)
    }
}

impl From<&ModuleId> for AccessPath {
    fn from(id: &ModuleId) -> AccessPath {
        AccessPath::code_access_path(*id.address(), id.name().to_owned())
    }
}

impl From<&ResourceKey> for AccessPath {
    fn from(key: &ResourceKey) -> AccessPath {
        AccessPath::resource_access_path(key.address(), key.type_().clone())
    }
}

#[derive(
    IntoPrimitive,
    TryFromPrimitive,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
    Ord,
    PartialOrd,
    Debug,
)]
#[repr(u8)]
pub enum DataType {
    CODE,
    RESOURCE,
}

impl DataType {
    pub const LENGTH: usize = 2;

    pub fn is_code(self) -> bool {
        matches!(self, DataType::CODE)
    }
    pub fn is_resource(self) -> bool {
        matches!(self, DataType::RESOURCE)
    }

    #[inline]
    pub fn type_index(self) -> u8 {
        self.into()
    }

    /// Every DataType has a storage root in AccountState
    #[inline]
    pub fn storage_index(self) -> usize {
        self.type_index() as usize
    }
}

pub type ModuleName = Identifier;

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, Debug)]
pub enum DataPath {
    Code(ModuleName),
    Resource(StructTag),
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for DataPath {
    type Parameters = ();
    fn arbitrary_with((): ()) -> Self::Strategy {
        //TODO
        unimplemented!()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl DataPath {
    pub fn is_code(&self) -> bool {
        matches!(self, DataPath::Code(_))
    }
    pub fn is_resource(&self) -> bool {
        matches!(self, DataPath::Resource(_))
    }
    pub fn as_struct_tag(&self) -> Option<&StructTag> {
        match self {
            DataPath::Resource(struct_tag) => Some(struct_tag),
            _ => None,
        }
    }
    pub fn data_type(&self) -> DataType {
        match self {
            DataPath::Code(_) => DataType::CODE,
            DataPath::Resource(_) => DataType::RESOURCE,
        }
    }
    //TODO implement RawKey for DataPath?
    pub fn key_hash(&self) -> HashValue {
        match self {
            DataPath::Resource(struct_tag) => struct_tag.crypto_hash(),
            DataPath::Code(module_name) => module_name.to_owned().crypto_hash(),
        }
    }
}

impl fmt::Display for DataPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let storage_index = self.data_type().storage_index();
        match self {
            DataPath::Resource(struct_tag) => {
                write!(f, "{}/{}", storage_index, struct_tag)
            }
            DataPath::Code(module_name) => {
                write!(f, "{}/{}", storage_index, module_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type() {
        let (_address, data_path) = AccessPath::random_resource().into_inner();
        assert_eq!(data_path.data_type(), DataType::RESOURCE);

        let (_address, data_path) = AccessPath::random_code().into_inner();
        assert_eq!(data_path.data_type(), DataType::CODE);
    }
}
