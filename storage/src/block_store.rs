// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecStorage, Repository, ValueCodec};
use anyhow::{bail, Result};
use crypto::hash::CryptoHash;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::block::{Block, BlockHeader};

pub struct BlockStore {
    store: CodecStorage<HashValue, Block>,
}

impl ValueCodec for Block {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl BlockStore {
    pub fn new(block_store: Arc<dyn Repository>) -> Self {
        BlockStore {
            store: CodecStorage::new(block_store),
        }
    }

    pub fn save(&self, block: Block) -> Result<()> {
        self.store.put(block.header().id(), block)
    }

    pub fn get(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.store.get(block_id)
    }

    pub fn commit_block(&self, block: Block) -> Result<()> {
        self.save(block)
    }

    ///返回某个块到分叉块的路径上所有块的hash
    pub fn get_branch_hashes(&self, hash: HashValue) -> Result<Vec<HashValue>> {
        unimplemented!()
    }

    pub fn get_latest_block_header(&self) -> Result<BlockHeader> {
        unimplemented!()
    }

    pub fn get_latest_block(&self) -> Result<Block> {
        unimplemented!()
    }

    pub fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<BlockHeader> {
        match self.get(block_id).unwrap() {
            Some(block) => Ok(block.header().clone()),
            None => bail!("can't find block:{}", block_id),
        }
    }

    pub fn get_block_by_hash(&self, block_id: HashValue) -> Result<Block> {
        match self.get(block_id).unwrap() {
            Some(block) => Ok(block),
            None => bail!("can't find block:{}", block_id),
        }
    }

    pub fn get_block_header_by_height(&self, height: u64) -> Result<BlockHeader> {
        unimplemented!()
    }

    pub fn get_block_by_height(&self, height: u64) -> Result<Block> {
        unimplemented!()
    }
}
