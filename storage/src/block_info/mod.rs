// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecStorage, ColumnFamilyName, Repository, ValueCodec};
use anyhow::Result;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::block::BlockInfo;

pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub trait BlockInfoStorage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<()>;
    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>>;
}

pub struct BlockInfoStore {
    store: CodecStorage<HashValue, BlockInfo>,
}

impl ValueCodec for BlockInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl BlockInfoStore {
    pub fn new(kv_store: Arc<dyn Repository>) -> Self {
        BlockInfoStore {
            store: CodecStorage::new(kv_store),
        }
    }

    pub fn save(&self, block_info: BlockInfo) -> Result<()> {
        self.store.put(block_info.block_id, block_info)
    }

    pub fn get(&self, hash_value: HashValue) -> Result<Option<BlockInfo>> {
        self.store.get(hash_value)
    }
}
