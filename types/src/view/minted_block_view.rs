// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct MintedBlockView {
    pub block_hash: HashValue,
}
