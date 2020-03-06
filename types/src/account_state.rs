// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::convert::{TryFrom, TryInto};

#[derive(Default, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, CryptoHash)]
pub struct AccountState {
    code_root: Option<HashValue>,
    resource_root: HashValue,
}

impl AccountState {
    pub fn new(code_root: Option<HashValue>, resource_root: HashValue) -> AccountState {
        Self {
            code_root,
            resource_root,
        }
    }

    pub fn resource_root(&self) -> HashValue {
        self.resource_root
    }

    pub fn code_root(&self) -> Option<HashValue> {
        self.code_root
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
