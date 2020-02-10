// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::HashValue;

/// Define a state merkle tree root
pub struct StorageRoot {
    root: HashValue,
}
