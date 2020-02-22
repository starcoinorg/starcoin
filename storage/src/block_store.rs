// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::memory_storage::MemoryStorage;
use crate::storage::{CodecStorage, KeyCodec, Repository, ValueCodec};
use anyhow::{bail, Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use crypto::hash::CryptoHash;
use crypto::HashValue;
use scs::SCSCodec;
use std::io::Write;
use std::mem::size_of;
use std::sync::Arc;
use types::block::{Block, BlockBody, BlockHeader, BlockNumber};

pub struct BlockStore {
    block_store: CodecStorage<HashValue, Block>,
    header_store: CodecStorage<HashValue, BlockHeader>,
    //store parents relationship
    sons_store: CodecStorage<HashValue, Vec<HashValue>>,
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

impl ValueCodec for Vec<HashValue> {
    fn encode_value(&self) -> Result<Vec<u8>> {
        let mut encoded = vec![];
        for hash in self {
            encoded.write_all(&hash.to_vec());
        }
        Ok(encoded)
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        let hash_size = size_of::<HashValue>();
        let mut decoded = vec![];
        let mut ends = hash_size;
        let len = data.len();
        let mut begin: usize = 0;
        loop {
            if ends <= len {
                let hash = HashValue::from_slice(&data[begin..ends])?;
                decoded.push(hash);
            } else {
                break;
            }
            begin = ends;
            ends = ends + hash_size;
        }
        Ok(decoded)
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
        sons_store: Arc<dyn Repository>,
        body_store: Arc<dyn Repository>,
        number_store: Arc<dyn Repository>,
    ) -> Self {
        BlockStore {
            block_store: CodecStorage::new(block_store),
            header_store: CodecStorage::new(header_store),
            sons_store: CodecStorage::new(sons_store),
            body_store: CodecStorage::new(body_store),
            number_store: CodecStorage::new(number_store),
        }
    }

    pub fn save(&self, block: Block) -> Result<()> {
        self.block_store.put(block.header().id(), block)
    }

    pub fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.header_store.put(header.id(), header.clone());
        //save sons relationship
        self.put_sons(header.parent_hash(), header.id())
    }

    pub fn get_headers(&self) -> Result<Vec<HashValue>> {
        let mut key_hashes = vec![];
        for hash in self.header_store.keys().unwrap() {
            let hashval = HashValue::from_slice(hash.as_slice()).unwrap();
            println!("header key:{}", hashval.to_hex());
            key_hashes.push(hashval)
        }
        Ok(key_hashes)
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
        let (header, body) = block.clone().into_inner();
        //save header
        let block_id = header.id();
        self.save_header(header.clone());
        //save number
        self.save_number(header.number(), block_id);
        //save body
        self.save_body(block_id, body);
        //save block cache
        self.save(block)
    }

    ///返回某个块到分叉块的路径上所有块的hash
    pub fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        let mut vev_hash = Vec::new();
        let mut temp_block_id = block_id;
        loop {
            println!("block_id: {}", temp_block_id.to_hex());
            //get header by block_id
            match self.get_block_header_by_hash(temp_block_id) {
                Ok(header) => {
                    if header.id() != block_id {
                        vev_hash.push(header.id());
                    }
                    temp_block_id = header.parent_hash();
                    match self.get_sons(temp_block_id) {
                        Ok(sons) => {
                            println!("  sons:{:?}", sons);
                            if sons.len() > 1 {
                                break;
                            }
                        }
                        Err(err) => bail!("get sons Error: {:?}", err),
                    }
                }
                Err(err) => bail!("Error: {:?}", err),
            }
        }
        Ok(vev_hash)
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
        let result = self.header_store.get(block_id);
        match result {
            Ok(option_header) => match option_header {
                Some(header) => Ok(header),
                None => bail!("can't find block header:{}", block_id.to_hex()),
            },
            Err(err) => bail!("Error: {:?}", err),
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
            Some(block_id) => self.get_block_header_by_hash(block_id),
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

    fn get_sons(&self, parent_hash: HashValue) -> Result<Vec<HashValue>> {
        match self.sons_store.get(parent_hash).unwrap() {
            Some(sons) => Ok(sons),
            None => bail!("cant't find sons: {}", parent_hash),
        }
    }

    fn put_sons(&self, parent_hash: HashValue, son_hash: HashValue) -> Result<()> {
        println!("put son:{}, {}", parent_hash, son_hash);
        match self.get_sons(parent_hash) {
            Ok(mut vec_hash) => {
                println!("branch block:{}, {:?}", parent_hash, vec_hash);
                vec_hash.push(son_hash);
                self.sons_store.put(parent_hash, vec_hash);
            }
            _ => {
                self.sons_store.put(parent_hash, vec![son_hash]);
            }
        }
        Ok(())
    }
}
