// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::event::EventHandle;
use crate::move_resource::MoveResource;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

/// On chain resource BlockMetadata mapping
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockMetadata {
    // number of the current block
    pub number: u64,
    // Hash of the parent block.
    pub parent_hash: HashValue,
    // Author of the current block.
    pub author: AccountAddress,
    pub uncles: u64,
    // Handle where events with the time of new blocks are emitted
    pub new_block_events: EventHandle,
}

impl MoveResource for BlockMetadata {
    const MODULE_NAME: &'static str = "Block";
    const STRUCT_NAME: &'static str = "BlockMetadata";
}

/// On chain resource BlockMetadata mapping for FlexiDag block
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockMetadataV2 {
    // number of the current block
    pub number: u64,
    // Hash of the parent block.
    pub parent_hash: HashValue,
    // Author of the current block.
    pub author: AccountAddress,
    pub uncles: u64,
    pub parents_hash: Vec<u8>,
    // Handle where events with the time of new blocks are emitted
    pub new_block_events: EventHandle,
}

impl BlockMetadataV2 {
    pub fn parents_hash(&self) -> anyhow::Result<Vec<HashValue>> {
        bcs_ext::from_bytes(self.parents_hash.as_slice())
    }
}

impl MoveResource for BlockMetadataV2 {
    const MODULE_NAME: &'static str = "Block";
    const STRUCT_NAME: &'static str = "BlockMetadataV2";
}
