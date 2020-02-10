// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Default, Hash, Serialize, Deserialize, Ord, PartialOrd)]
pub struct AccessPath {
    pub address: AccountAddress,
    pub path: Vec<u8>,
}
