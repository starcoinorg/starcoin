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
use crate::parser::parse_struct_tag;
use anyhow::{bail, Result};
use forkable_jellyfish_merkle::RawKey;
use move_core_types::language_storage::{ModuleId, ResourceKey, StructTag};
use num_enum::{IntoPrimitive, TryFromPrimitive};
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use rand::prelude::{Distribution, SliceRandom};
use rand::rngs::OsRng;
use rand::Rng;
use schemars::{self, JsonSchema};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::hash::HashValue;
use std::fmt;
use std::str::FromStr;
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd, JsonSchema)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[schemars(with = "String")]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: DataPath,
}

impl AccessPath {
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

    pub fn as_module_id(&self) -> Option<ModuleId> {
        match &self.path {
            DataPath::Code(module_name) => Some(ModuleId::new(self.address, module_name.clone())),
            _ => None,
        }
    }
}

impl Serialize for AccessPath {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.to_string().as_str())
        } else {
            serializer.serialize_newtype_struct("AccessPath", &(self.address, self.path.clone()))
        }
    }
}

impl<'de> Deserialize<'de> for AccessPath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            AccessPath::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "AccessPath")]
            struct Value(AccountAddress, DataPath);
            let value = Value::deserialize(deserializer)?;
            Ok(AccessPath::new(value.0, value.1))
        }
    }
}

//TODO move to a suitable mod
struct IdentifierSymbols;

impl Distribution<char> for IdentifierSymbols {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> char {
        //TODO add more valid identity char
        *b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .choose(rng)
            .unwrap_or(&97) as char
    }
}

fn random_identity() -> Identifier {
    let rng = OsRng;
    let id: String = rng.sample_iter(&IdentifierSymbols).take(7).collect();
    Identifier::new(id).expect("random identity should valid.")
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
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
#[allow(clippy::upper_case_acronyms)]
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

    pub fn from_index(idx: u8) -> Result<Self> {
        Ok(Self::try_from_primitive(idx)?)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for DataType {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![Just(DataType::CODE), Just(DataType::RESOURCE),].boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

pub type ModuleName = Identifier;

#[derive(
    Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, Debug, JsonSchema,
)]
pub enum DataPath {
    Code(#[schemars(with = "String")] ModuleName),
    Resource(#[schemars(with = "String")] StructTag),
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for DataPath {
    type Parameters = ();
    fn arbitrary_with(_args: ()) -> Self::Strategy {
        prop_oneof![
            (any::<Identifier>()).prop_map(DataPath::Code),
            (
                any::<AccountAddress>(),
                any::<Identifier>(),
                any::<Identifier>(),
                vec(any::<move_core_types::language_storage::TypeTag>(), 0..4),
            )
                .prop_map(|(address, module, name, type_params)| DataPath::Resource(
                    StructTag {
                        address,
                        module,
                        name,
                        type_params,
                    }
                )),
        ]
        .boxed()
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

    pub fn key_hash(&self) -> HashValue {
        match self {
            DataPath::Resource(struct_tag) => struct_tag.key_hash(),
            DataPath::Code(module_name) => module_name.key_hash(),
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

impl FromStr for AccessPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<_>>();
        if parts.len() != 3 {
            bail!("Invalid access_path string: {}", s);
        }
        let address = AccountAddress::from_str(parts[0])?;
        let data_type = DataType::from_index(parts[1].parse()?)?;
        let data_path = match data_type {
            DataType::CODE => AccessPath::code_data_path(Identifier::new(parts[2])?),
            DataType::RESOURCE => AccessPath::resource_data_path(parse_struct_tag(parts[2])?),
        };
        Ok(AccessPath::new(address, data_path))
    }
}
