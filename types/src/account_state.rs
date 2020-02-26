// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};

#[derive(Default, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, CryptoHash)]
pub struct AccountState {
    code_root: HashValue,
    storage_root: HashValue,
}

impl AccountState {
    pub fn new(code_root: HashValue, storage_root: HashValue) -> AccountState {
        Self {
            code_root,
            storage_root,
        }
    }

    pub fn storage_root(&self) -> HashValue {
        self.storage_root
    }

    pub fn code_root(&self) -> HashValue {
        self.code_root
    }
}
