// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::{CodecStorage, KeyCodec, StorageInstance, ValueCodec};
use crate::{
    BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME, BLOCK_NUM_PREFIX_NAME, BLOCK_PREFIX_NAME,
    BLOCK_SONS_PREFIX_NAME,
};
use anyhow::{bail, ensure, Error, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockNumber, BranchNumber};
use std::io::Write;
use std::mem::size_of;
use std::sync::{Arc, RwLock};
define_storage!(BlockInnerStorage, HashValue, Block, BLOCK_PREFIX_NAME);
define_storage!(
    BlockHeaderStorage,
    HashValue,
    BlockHeader,
    BLOCK_HEADER_PREFIX_NAME
);
define_storage!(
    BlockSonsStorage,
    HashValue,
    Vec<HashValue>,
    BLOCK_SONS_PREFIX_NAME
);
define_storage!(
    BlockBodyStorage,
    HashValue,
    BlockBody,
    BLOCK_BODY_PREFIX_NAME
);
define_storage!(
    BlockNumberStorage,
    BlockNumber,
    HashValue,
    BLOCK_NUM_PREFIX_NAME
);
define_storage!(
    BranchNumberStorage,
    BranchNumber,
    HashValue,
    BLOCK_NUM_PREFIX_NAME
);

pub struct BlockStorage {
    block_store: BlockInnerStorage,
    header_store: BlockHeaderStorage,
    //store parents relationship
    sons_store: RwLock<BlockSonsStorage>,
    body_store: BlockBodyStorage,
    number_store: BlockNumberStorage,
    branch_number_store: BranchNumberStorage,
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
            encoded.write_all(&hash.to_vec()).unwrap();
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

impl KeyCodec for BranchNumber {
    fn encode_key(&self) -> Result<Vec<u8>> {
        let (branch_id, number) = *self;

        let mut encoded_key = Vec::with_capacity(size_of::<BranchNumber>());
        encoded_key.write(&branch_id.to_vec()).unwrap();
        encoded_key.write_u64::<BigEndian>(number)?;
        Ok(encoded_key)
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        let branch_id = HashValue::from_slice(&data[..HashValue::LENGTH])?;
        let number = (&data[HashValue::LENGTH..]).read_u64::<BigEndian>()?;
        Ok((branch_id, number))
    }
}

impl BlockStorage {
    pub fn new(instance: StorageInstance) -> Self {
        BlockStorage {
            block_store: BlockInnerStorage::new(instance.clone()),
            header_store: BlockHeaderStorage::new(instance.clone()),
            sons_store: RwLock::new(BlockSonsStorage::new(instance.clone())),
            body_store: BlockBodyStorage::new(instance.clone()),
            number_store: BlockNumberStorage::new(instance.clone()),
            branch_number_store: BranchNumberStorage::new(instance.clone()),
        }
    }
    pub fn save(&self, block: Block) -> Result<()> {
        debug!(
            "insert block:{:?}, block:{:?}",
            block.header().id(),
            block.header().parent_hash()
        );
        self.block_store.put(block.header().id(), block)
    }

    pub fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.header_store.put(header.id(), header.clone()).unwrap();
        //save sons relationship
        self.put_sons(header.parent_hash(), header.id())
    }

    pub fn get_headers(&self) -> Result<Vec<HashValue>> {
        let mut key_hashes = vec![];
        for hash in self.header_store.keys().unwrap() {
            let hashval = HashValue::from_slice(hash.as_slice()).unwrap();
            debug!("header key:{}", hashval.to_hex());
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
    pub fn save_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
        block_id: HashValue,
    ) -> Result<()> {
        let key = (branch_id, number);
        self.branch_number_store.put(key, block_id)
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

    pub fn get_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<HashValue>> {
        let key = (branch_id, number);
        self.branch_number_store.get(key)
    }

    pub fn commit_block(&self, block: Block) -> Result<()> {
        let (header, body) = block.clone().into_inner();
        //save header
        let block_id = header.id();
        self.save_header(header.clone()).unwrap();
        //save number
        self.save_number(header.number(), block_id).unwrap();
        //save body
        self.save_body(block_id, body).unwrap();
        //save block cache
        self.save(block)
    }

    pub fn commit_branch_block(&self, branch_id: HashValue, block: Block) -> Result<()> {
        info!("commit block: {:?}, block: {:?}", branch_id, block);
        let (header, body) = block.clone().into_inner();
        //save header
        let block_id = header.id();
        self.save_header(header.clone()).unwrap();
        //save number
        self.save_branch_number(branch_id, header.number(), block_id)
            .unwrap();
        //save body
        self.save_body(block_id, body).unwrap();
        //save block cache
        self.save(block)
    }

    ///返回某个块到分叉块的路径上所有块的hash
    pub fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        let mut vev_hash = Vec::new();
        let mut temp_block_id = block_id;
        loop {
            debug!("block_id: {}", temp_block_id.to_hex());
            //get header by block_id
            match self.get_block_header_by_hash(temp_block_id)? {
                Some(header) => {
                    if header.id() != block_id {
                        vev_hash.push(header.id());
                    }
                    temp_block_id = header.parent_hash();
                    match self.get_sons(temp_block_id) {
                        Ok(sons) => {
                            if sons.len() > 1 {
                                break;
                            }
                        }
                        Err(err) => bail!("get sons Error: {:?}", err),
                    }
                }
                None => bail!("Error: can not find block {:?}", temp_block_id),
            }
        }
        Ok(vev_hash)
    }
    /// Get common ancestor
    #[allow(dead_code)]
    pub fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        let mut parent_id1 = block_id1;
        let mut parent_id2 = block_id2;
        let mut found;
        info!("common ancestor: {:?}, {:?}", block_id1, block_id2);
        match self.get_relationship(block_id1, block_id2) {
            Ok(Some(hash)) => return Ok(Some(hash)),
            _ => {}
        }
        match self.get_relationship(block_id2, block_id1) {
            Ok(Some(hash)) => return Ok(Some(hash)),
            _ => {}
        }

        loop {
            // info!("block_id: {}", parent_id1.to_hex());
            //get header by block_id
            match self.get_block_header_by_hash(parent_id1)? {
                Some(header) => {
                    parent_id1 = header.parent_hash();
                    ensure!(parent_id1 != HashValue::zero(), "invaild block id is zero.");
                    match self.get_sons(parent_id1) {
                        Ok(sons1) => {
                            info!("parent: {:?}, sons1 : {:?}", parent_id1, sons1);
                            if sons1.len() > 1 {
                                // get parent2 from block2
                                loop {
                                    info!("parent2 : {:?}", parent_id2);
                                    ensure!(
                                        parent_id2 != HashValue::zero(),
                                        "invaild block id is zero."
                                    );
                                    if sons1.contains(&parent_id2) {
                                        found = true;
                                        break;
                                    }
                                    match self.get_block_header_by_hash(parent_id2)? {
                                        Some(header2) => {
                                            parent_id2 = header2.parent_hash();
                                        }
                                        None => {
                                            bail!("Error: can not find block2 {:?}", parent_id2)
                                        }
                                    }
                                }
                                if found {
                                    break;
                                }
                            }
                        }
                        Err(err) => bail!("get sons Error: {:?}", err),
                    }
                }
                None => bail!("Error: can not find block {:?}", parent_id1),
            }
        }
        if found {
            Ok(Some(parent_id1))
        } else {
            bail!("not find common ancestor");
        }
    }

    pub fn get_latest_block_header(&self) -> Result<Option<BlockHeader>> {
        let max_number = self.number_store.get_len()?;
        if max_number == 0 {
            return Ok(None);
        }
        self.get_block_header_by_number(max_number - 1)
    }

    pub fn get_latest_block(&self) -> Result<Block> {
        //get storage current len
        let max_number = self.number_store.get_len()?;
        Ok(self.get_block_by_number(max_number - 1)?.unwrap())
    }

    pub fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.header_store.get(block_id)
    }

    pub fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.get(block_id)
    }

    pub fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        match self.number_store.get(number).unwrap() {
            Some(block_id) => self.get_block_header_by_hash(block_id),
            None => bail!("can't find block header by number:{}", number),
        }
    }

    pub fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        match self.number_store.get(number)? {
            Some(block_id) => self.block_store.get(block_id),
            None => Ok(None),
        }
    }

    pub fn get_header_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<BlockHeader>> {
        let key = (branch_id, number);
        match self.branch_number_store.get(key).unwrap() {
            Some(block_id) => self.get_block_header_by_hash(block_id),
            None => bail!(
                "can't find header by branch number:{:?}, {}",
                branch_id,
                number
            ),
        }
    }

    pub fn get_block_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<Block>> {
        let key = (branch_id, number);
        match self.branch_number_store.get(key).unwrap() {
            Some(block_id) => self.get(block_id),
            None => bail!(
                "can't find block by branch number:{:?}, {}",
                branch_id,
                number
            ),
        }
    }

    fn get_relationship(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        match self.get_sons(block_id1) {
            Ok(sons) => {
                if sons.contains(&block_id2) {
                    return Ok(Some(block_id1));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn get_sons(&self, parent_hash: HashValue) -> Result<Vec<HashValue>> {
        match self.sons_store.read().unwrap().get(parent_hash)? {
            Some(sons) => Ok(sons),
            None => bail!("cant't find sons: {}", parent_hash),
        }
    }

    fn put_sons(&self, parent_hash: HashValue, son_hash: HashValue) -> Result<()> {
        info!("put son:{}, {}", parent_hash, son_hash);
        match self.get_sons(parent_hash) {
            Ok(mut vec_hash) => {
                info!("branch block:{}, {:?}", parent_hash, vec_hash);
                vec_hash.push(son_hash);
                self.sons_store
                    .write()
                    .unwrap()
                    .put(parent_hash, vec_hash)?;
            }
            _ => {
                self.sons_store
                    .write()
                    .unwrap()
                    .put(parent_hash, vec![son_hash])?;
            }
        }
        Ok(())
    }
}
