// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecStorage, KeyCodec, Repository, ValueCodec};
use anyhow::{bail, Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use crypto::hash::CryptoHash;
use crypto::HashValue;
use scs::SCSCodec;
use std::sync::Arc;
use types::block::{Block, BlockBody, BlockHeader, BlockNumber};

pub struct BlockStore {
    block_store: CodecStorage<HashValue, Block>,
    header_store: CodecStorage<HashValue, BlockHeader>,
    body_store: CodecStorage<HashValue, BlockBody>,
    number_store: CodecStorage<BlockNumber, HashValue>,
}

impl ValueCodec for Block {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for BlockHeader {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for BlockBody {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl KeyCodec for BlockNumber {
    fn encode_key(&self) -> Result<Vec<u8>> {
        Ok(self.to_be_bytes().to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        Ok((&data[..]).read_u64::<BigEndian>()?)
    }
}

impl BlockStore {
    pub fn new(
        block_store: Arc<dyn Repository>,
        header_store: Arc<dyn Repository>,
        body_store: Arc<dyn Repository>,
        number_store: Arc<dyn Repository>,
    ) -> Self {
        BlockStore {
            block_store: CodecStorage::new(block_store),
            header_store: CodecStorage::new(header_store),
            body_store: CodecStorage::new(body_store),
            number_store: CodecStorage::new(number_store),
        }
    }

    pub fn save(&self, block: Block) -> Result<()> {
        self.block_store.put(block.header().id(), block)
    }

    pub fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.header_store.put(header.id(), header)
    }

    pub fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()> {
        self.body_store.put(block_id, body)
    }
    pub fn save_number(&self, number: BlockNumber, block_id: HashValue) -> Result<()> {
        self.number_store.put(number, block_id)
    }

    pub fn get(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get(block_id)
    }

    pub fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.body_store.get(block_id)
    }

    pub fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.number_store.get(number)
    }

    pub fn commit_block(&self, block: Block) -> Result<()> {
        self.save(block)
    }

    ///返回某个块到分叉块的路径上所有块的hash
    pub fn get_branch_hashes(&self, hash: HashValue) -> Result<Vec<HashValue>> {
        unimplemented!()
    }

    pub fn get_latest_block_header(&self) -> Result<BlockHeader> {
        let max_number = self.number_store.get_len()?;
        self.get_block_header_by_number(max_number - 1)
    }

    pub fn get_latest_block(&self) -> Result<Block> {
        //get storage current len
        let max_number = self.number_store.get_len()?;
        self.get_block_by_number(max_number - 1)
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

    pub fn get_block_header_by_number(&self, number: u64) -> Result<BlockHeader> {
        match self.number_store.get(number).unwrap() {
            Some(block_id) => match self.header_store.get(block_id)? {
                Some(header) => Ok(header),
                None => bail!("can't find block header:{}", number),
            },
            None => bail!("can't find block header by number:{}", number),
        }
    }

    pub fn get_block_by_number(&self, number: u64) -> Result<Block> {
        match self.number_store.get(number).unwrap() {
            Some(block_id) => match self.block_store.get(block_id).unwrap() {
                Some(block) => Ok(block),
                None => bail!("can't find block:{}", number),
            },
            None => bail!("can't find block  by number:{}", number),
        }
    }
}
