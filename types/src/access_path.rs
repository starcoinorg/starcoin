// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
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

use crate::{
    account_address::AccountAddress,
    account_config::{association_address, ACCOUNT_RESOURCE_PATH},
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, ResourceKey, StructTag},
};
use anyhow::{Error, Result};
use mirai_annotations::*;
use num_enum::IntoPrimitive;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, HashValue};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    slice::Iter,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Field(Identifier);

impl Field {
    pub fn new(name: Identifier) -> Field {
        Field(name)
    }

    pub fn name(&self) -> &IdentStr {
        &self.0
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Eq, Hash, Serialize, Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub enum Access {
    Field(Field),
    Index(u64),
}

impl Access {
    pub fn new(name: Identifier) -> Self {
        Access::Field(Field::new(name))
    }
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Access::Field(field) => write!(f, "\"{}\"", field),
            Access::Index(i) => write!(f, "{}", i),
        }
    }
}

/// Non-empty sequence of field accesses
#[derive(Eq, Hash, Serialize, Deserialize, Debug, Clone, PartialEq, Ord, PartialOrd)]
pub struct Accesses(Vec<Access>);
// invariant self.0.len() == 1

/// SEPARATOR is used as a delimiter between fields. It should not be a legal part of any identifier
/// in the language
const SEPARATOR: char = '/';

impl Accesses {
    pub fn empty() -> Self {
        Accesses(vec![])
    }

    pub fn new(field: Field) -> Self {
        Accesses(vec![Access::Field(field)])
    }

    /// Add a field to the end of the sequence
    pub fn add_field_to_back(&mut self, field: Field) {
        self.0.push(Access::Field(field))
    }

    /// Add an index to the end of the sequence
    pub fn add_index_to_back(&mut self, idx: u64) {
        self.0.push(Access::Index(idx))
    }

    pub fn append(&mut self, accesses: &mut Accesses) {
        self.0.append(&mut accesses.0)
    }

    /// Returns the first field in the sequence and reference to the remaining fields
    pub fn split_first(&self) -> (&Access, &[Access]) {
        self.0.split_first().unwrap()
    }

    /// Return the last access in the sequence
    pub fn last(&self) -> &Access {
        assume!(self.0.last().is_some()); // follows from invariant
        self.0.last().unwrap() // guaranteed not to fail because sequence is non-empty
    }

    pub fn iter(&self) -> Iter<'_, Access> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_separated_string(&self) -> String {
        let mut path = String::new();
        for access in self.0.iter() {
            match access {
                Access::Field(s) => {
                    let access_str = s.name().as_str();
                    assert!(access_str != "");
                    path.push_str(access_str)
                }
                Access::Index(i) => path.push_str(i.to_string().as_ref()),
            };
            path.push(SEPARATOR);
        }
        path
    }

    pub fn take_nth(&self, new_len: usize) -> Accesses {
        assert!(self.0.len() >= new_len);
        Accesses(self.0.clone().into_iter().take(new_len).collect())
    }
}

impl<'a> IntoIterator for &'a Accesses {
    type Item = &'a Access;
    type IntoIter = Iter<'a, Access>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl From<Vec<Access>> for Accesses {
    fn from(accesses: Vec<Access>) -> Accesses {
        Accesses(accesses)
    }
}

#[derive(
    IntoPrimitive, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, Debug,
)]
#[repr(u8)]
pub enum DataType {
    RESOURCE,
    CODE,
}

impl DataType {
    pub fn is_code(&self) -> bool {
        match self {
            DataType::CODE => true,
            _ => false,
        }
    }
    pub fn is_resource(&self) -> bool {
        match self {
            DataType::RESOURCE => true,
            _ => false,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Ord, PartialOrd, CryptoHash)]
pub struct AccessPath {
    address: AccountAddress,
    data_type: DataType,
    data_hash: HashValue,
}

impl AccessPath {
    pub fn new(address: AccountAddress, data_type: DataType, data_hash: HashValue) -> Self {
        AccessPath {
            address,
            data_type,
            data_hash,
        }
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn data_hash(&self) -> HashValue {
        self.data_hash
    }

    /// Given an address, returns the corresponding access path that stores the Account resource.
    pub fn new_for_account(address: AccountAddress) -> Self {
        Self::new(address, DataType::RESOURCE, ACCOUNT_RESOURCE_PATH.clone())
    }

    pub fn resource_access_vec(tag: &StructTag) -> HashValue {
        tag.crypto_hash()
    }

    /// Convert Accesses into a byte offset which would be used by the storage layer to resolve
    /// where fields are stored.
    pub fn resource_access_path(key: &ResourceKey) -> AccessPath {
        let path = AccessPath::resource_access_vec(&key.type_());
        AccessPath {
            address: key.address().to_owned(),
            data_type: DataType::RESOURCE,
            data_hash: path,
        }
    }

    fn code_access_path_vec(key: &ModuleId) -> HashValue {
        key.crypto_hash()
    }

    pub fn code_access_path(key: &ModuleId) -> AccessPath {
        let path = AccessPath::code_access_path_vec(key);
        AccessPath {
            address: key.address(),
            data_type: DataType::CODE,
            data_hash: path,
        }
    }
}

impl fmt::Debug for AccessPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AccessPath {{ address: {:x}, type: {:?} path: {} }}",
            self.address,
            self.data_type,
            self.data_hash.to_hex()
        )
    }
}

impl fmt::Display for AccessPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AccessPath {{ address: {:x}, ", self.address)?;
        match &self.data_type {
            DataType::RESOURCE => write!(f, "type: Resource, ")?,
            DataType::CODE => write!(f, "type: Module, ")?,
            tag => write!(f, "type: {:?}, ", tag)?,
        };
        write!(f, "hash: {:?}, ", self.data_hash.to_hex())?;
        // write!(
        //     f,
        //     "suffix: {:?} }} ",
        //     String::from_utf8_lossy(&self.path[1 + HashValue::LENGTH..])
        // )
        Ok(())
    }
}

impl Into<(AccountAddress, DataType, HashValue)> for AccessPath {
    fn into(self) -> (AccountAddress, DataType, HashValue) {
        (self.address, self.data_type, self.data_hash)
    }
}

// libra data tag.
const CODE_TAG: u8 = 0;
const RESOURCE_TAG: u8 = 1;

impl Into<libra_types::access_path::AccessPath> for AccessPath {
    fn into(self) -> libra_types::access_path::AccessPath {
        let mut path = vec![];
        match self.data_type {
            DataType::RESOURCE => path.push(RESOURCE_TAG),
            DataType::CODE => path.push(CODE_TAG),
        }
        path.extend(self.data_hash.to_vec());
        libra_types::access_path::AccessPath::new(self.address.into(), path)
    }
}

impl From<libra_types::access_path::AccessPath> for AccessPath {
    fn from(libra_access_path: libra_types::access_path::AccessPath) -> Self {
        let path = libra_access_path.path;
        let data_type = match path[0] {
            RESOURCE_TAG => DataType::RESOURCE,
            CODE_TAG => DataType::CODE,
            _ => panic!("Unsupported access path."),
        };
        let hash = HashValue::from_slice(path[1..=HashValue::LENGTH].as_ref())
            .expect("access_path must contains HashValue");
        Self::new(libra_access_path.address.into(), data_type, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        let access_path0 = AccessPath::new(
            AccountAddress::random(),
            DataType::RESOURCE,
            HashValue::random(),
        );
        let access_path1: libra_types::access_path::AccessPath = access_path0.clone().into();
        let access_path2: AccessPath = access_path1.into();
        assert_eq!(access_path0, access_path2);
    }
}
