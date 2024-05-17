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

pub enum BlockMetadataWrapper {
    V1(BlockMetadata),
    V2(BlockMetadataV2),
}

impl BlockMetadataWrapper {
    pub fn number(&self) -> u64 {
        match self {
            BlockMetadataWrapper::V1(block) => block.number,
            BlockMetadataWrapper::V2(block) => block.number,
        }
    }

    pub fn parent_hash(&self) -> &HashValue {
        match self {
            BlockMetadataWrapper::V1(block) => &block.parent_hash,
            BlockMetadataWrapper::V2(block) => &block.parent_hash,
        }
    }

    pub fn author(&self) -> &AccountAddress {
        match self {
            BlockMetadataWrapper::V1(block) => &block.author,
            BlockMetadataWrapper::V2(block) => &block.author,
        }
    }

    pub fn uncles(&self) -> u64 {
        match self {
            BlockMetadataWrapper::V1(block) => block.uncles,
            BlockMetadataWrapper::V2(block) => block.uncles,
        }
    }

    pub fn new_block_events(&self) -> &EventHandle {
        match self {
            BlockMetadataWrapper::V1(block) => &block.new_block_events,
            BlockMetadataWrapper::V2(block) => &block.new_block_events,
        }
    }

    pub fn parents_hash(&self) -> Option<Vec<HashValue>> {
        match self {
            BlockMetadataWrapper::V1(_) => None,
            BlockMetadataWrapper::V2(block) => {
                Some(block.parents_hash().expect("Get parent hash failed"))
            }
        }
    }
}
