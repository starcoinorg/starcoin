// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crypto::HashValue;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone)]
pub struct AccountState {
    code_root: HashValue,
    storage_root: HashValue,
}
