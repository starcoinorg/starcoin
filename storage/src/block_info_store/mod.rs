// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::KeyPrefixName;
use crate::storage::{CodecStorage, Repository, ValueCodec};
use anyhow::Result;
use crypto::hash::CryptoHash;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::block::BlockInfo;

const BLOCK_INFO_KEY_NAME: KeyPrefixName = "block_info";
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
            store: CodecStorage::new(kv_store, BLOCK_INFO_KEY_NAME),
        }
    }

    pub fn save(&self, block_info: BlockInfo) -> Result<()> {
        self.store.put(block_info.id(), block_info)
    }

    pub fn get(&self, hash_value: HashValue) -> Result<Option<BlockInfo>> {
        self.store.get(hash_value)
    }
}
