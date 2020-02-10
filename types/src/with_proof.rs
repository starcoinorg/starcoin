// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::proof::{AccountProof, StateProof};
use crate::storage_root::StorageRoot;

pub struct StateWithProof {
    pub state: Option<Vec<u8>>,
    pub proof: StateProof,
}

pub struct AccountWithProof {
    pub storage_root: StorageRoot,
    pub proof: AccountProof,
}
