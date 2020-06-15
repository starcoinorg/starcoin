// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::Result;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::HashValue;

pub use libra_types::access_path::AccessPath;
use std::convert::TryFrom;

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
        match self {
            DataType::CODE => true,
            _ => false,
        }
    }
    pub fn is_resource(self) -> bool {
        match self {
            DataType::RESOURCE => true,
            _ => false,
        }
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

pub fn into_inner(access_path: AccessPath) -> Result<(AccountAddress, DataType, HashValue)> {
    let address = access_path.address;
    let path = access_path.path;
    let data_type = DataType::try_from(path[0])?;
    let hash = HashValue::from_slice(&path[0..HashValue::LENGTH])?;
    Ok((address, data_type, hash))
}

pub fn new(address: AccountAddress, data_type: DataType, hash: HashValue) -> AccessPath {
    let mut path = vec![data_type.into()];
    path.extend(hash.to_vec());
    AccessPath::new(address, path)
}

pub fn random_code() -> AccessPath {
    new(
        AccountAddress::random(),
        DataType::CODE,
        HashValue::random(),
    )
}

pub fn random_resource() -> AccessPath {
    new(
        AccountAddress::random(),
        DataType::RESOURCE,
        HashValue::random(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_config::AccountResource;
    use crate::language_storage::ModuleId;
    use crate::move_resource::MoveResource;
    use libra_types::access_path::AccessPath;
    use move_core_types::identifier::Identifier;

    #[test]
    fn test_data_type() {
        let (_address, data_type, _hash) = into_inner(AccessPath::new(
            AccountAddress::random(),
            AccountResource::resource_path(),
        ))
        .unwrap();
        assert_eq!(data_type, DataType::RESOURCE);

        let (_address, data_type, _hash) = into_inner(AccessPath::code_access_path(
            &ModuleId::new(AccountAddress::random(), Identifier::new("Test").unwrap()),
        ))
        .unwrap();
        assert_eq!(data_type, DataType::CODE);
    }
}
