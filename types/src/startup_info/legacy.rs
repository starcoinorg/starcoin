// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{legacy::BlockInfo, BlockHeader};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::genesis_config::ChainId;
/// The info of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    chain_id: ChainId,
    genesis_hash: HashValue,
    status: ChainStatus,
}

impl ChainInfo {
    pub fn new(chain_id: ChainId, genesis_hash: HashValue, status: ChainStatus) -> Self {
        Self {
            chain_id,
            genesis_hash,
            status,
        }
    }

    pub fn into_inner(self) -> (ChainId, HashValue, ChainStatus) {
        (self.chain_id, self.genesis_hash, self.status)
    }
}

/// The latest status of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ChainStatus {
    /// Chain head block's header.
    pub head: BlockHeader,
    /// Chain block info
    pub info: BlockInfo,
}

impl ChainStatus {
    pub fn new(head: BlockHeader, info: BlockInfo) -> Self {
        Self { head, info }
    }

    pub fn into_inner(self) -> (BlockHeader, BlockInfo) {
        (self.head, self.info)
    }
}
