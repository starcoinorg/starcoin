// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataType;
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct AccountState {
    storage_roots: Vec<Option<HashValue>>,
}

impl AccountState {
    pub fn new(mut storage_roots: Vec<Option<HashValue>>) -> AccountState {
        if storage_roots.len() < DataType::LENGTH {
            storage_roots.extend(vec![None; DataType::LENGTH - storage_roots.len()]);
        }
        assert_eq!(
            storage_roots.len(),
            DataType::LENGTH,
            "Storage root length must equals DataType length"
        );
        Self { storage_roots }
    }

    pub fn resource_root(&self) -> HashValue {
        self.storage_roots[DataType::RESOURCE.storage_index()]
            .expect("Account at least must have resource storage root")
    }

    pub fn code_root(&self) -> Option<HashValue> {
        self.storage_roots[DataType::CODE.storage_index()]
    }

    pub fn storage_roots(&self) -> &[Option<HashValue>] {
        self.storage_roots.as_slice()
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            storage_roots: vec![None; DataType::LENGTH],
        }
    }
}

impl Into<Vec<Option<HashValue>>> for AccountState {
    fn into(self) -> Vec<Option<HashValue>> {
        self.storage_roots
    }
}

impl<'a> IntoIterator for &'a AccountState {
    type Item = &'a Option<HashValue>;
    type IntoIter = ::std::slice::Iter<'a, Option<HashValue>>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage_roots.iter()
    }
}

impl TryInto<Vec<u8>> for AccountState {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

impl<'a> TryFrom<&'a [u8]> for AccountState {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self> {
        Self::decode(value)
    }
}
