// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::DataType;
use anyhow::Result;
use bcs_ext::BCSCodec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use std::convert::{TryFrom, TryInto};
#[derive(
    Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct AccountState {
    storage_roots: Vec<Option<HashValue>>,
}

impl AccountState {
    pub fn new(code_root: Option<HashValue>, resource_root: HashValue) -> AccountState {
        let mut storage_roots = vec![None; DataType::LENGTH];
        storage_roots[DataType::CODE.storage_index()] = code_root;
        storage_roots[DataType::RESOURCE.storage_index()] = Some(resource_root);
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

#[allow(clippy::from_over_into)]
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
