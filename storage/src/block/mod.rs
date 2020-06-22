// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::batch::WriteBatch;
use crate::define_storage;
use crate::storage::{CodecStorage, KeyCodec, StorageInstance, ValueCodec};
use crate::{
    BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME, BLOCK_NUM_PREFIX_NAME, BLOCK_PREFIX_NAME,
    BLOCK_SONS_PREFIX_NAME, BLOCK_TRANSACTIONS_PREFIX_NAME, BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
};
use anyhow::{bail, ensure, Error, Result};
use byteorder::{BigEndian, ReadBytesExt};
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_types::block::{Block, BlockBody, BlockHeader, BlockNumber, BlockState};
use std::io::Write;
use std::mem::size_of;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageBlock {
    block: Block,
    state: BlockState,
}

impl StorageBlock {
    fn new(block: Block, state: BlockState) -> Self {
        Self { block, state }
    }
}

impl Into<(Block, BlockState)> for StorageBlock {
    fn into(self) -> (Block, BlockState) {
        (self.block, self.state)
    }
}

define_storage!(
    BlockInnerStorage,
    HashValue,
    StorageBlock,
    BLOCK_PREFIX_NAME
);
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
    BlockTransactionsStorage,
    HashValue,
    Vec<HashValue>,
    BLOCK_TRANSACTIONS_PREFIX_NAME
);

define_storage!(
    BlockTransactionInfosStorage,
    HashValue,
    Vec<HashValue>,
    BLOCK_TRANSACTION_INFOS_PREFIX_NAME
);

pub struct BlockStorage {
    block_store: BlockInnerStorage,
    header_store: BlockHeaderStorage,
    //store parents relationship
    sons_store: RwLock<BlockSonsStorage>,
    body_store: BlockBodyStorage,
    number_store: BlockNumberStorage,
    block_txns_store: BlockTransactionsStorage,
    block_txn_infos_store: BlockTransactionInfosStorage,
}

impl ValueCodec for StorageBlock {
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
            encoded.write_all(&hash.to_vec())?
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
            ends += hash_size;
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

impl BlockStorage {
    pub fn new(instance: StorageInstance) -> Self {
        BlockStorage {
            block_store: BlockInnerStorage::new(instance.clone()),
            header_store: BlockHeaderStorage::new(instance.clone()),
            sons_store: RwLock::new(BlockSonsStorage::new(instance.clone())),
            body_store: BlockBodyStorage::new(instance.clone()),
            number_store: BlockNumberStorage::new(instance.clone()),
            block_txns_store: BlockTransactionsStorage::new(instance.clone()),
            block_txn_infos_store: BlockTransactionInfosStorage::new(instance),
        }
    }
    pub fn save(&self, block: Block, state: BlockState) -> Result<()> {
        debug!(
            "insert block:{:?}, block:{:?}",
            block.header().id(),
            block.header().parent_hash()
        );
        let block_id = block.header().id();
        let storage_block = StorageBlock::new(block, state);
        self.block_store.put(block_id, storage_block)
    }

    pub fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.header_store.put(header.id(), header.clone())?;
        //save sons relationship
        self.put_sons(header.parent_hash(), header.id())
    }

    pub fn get_headers(&self) -> Result<Vec<HashValue>> {
        let mut key_hashes = vec![];
        for hash in self.header_store.keys()? {
            let hashval = HashValue::from_slice(hash.as_slice())?;
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
        Ok(
            if let Some(storage_block) = self.block_store.get(block_id)? {
                let (block, _) = storage_block.into();
                Some(block)
            } else {
                None
            },
        )
    }

    pub fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.body_store.get(block_id)
    }

    pub fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.number_store.get(number)
    }

    pub fn commit_block(&self, block: Block, state: BlockState) -> Result<()> {
        let (header, body) = block.clone().into_inner();
        //save header
        let block_id = header.id();
        self.save_header(header.clone())?;
        //save number
        self.save_number(header.number(), block_id)?;
        //save body
        self.save_body(block_id, body)?;
        //save block cache
        self.save(block, state)
    }

    ///返回某个块到分叉块的路径上所有块的hash
    pub fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        let mut vev_hash = Vec::new();
        let mut temp_block_id = block_id;
        loop {
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
    pub fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        let mut parent_id1 = block_id1;
        let mut parent_id2 = block_id2;
        let mut found;
        if let Ok(Some(hash)) = self.get_relationship(block_id1, block_id2) {
            return Ok(Some(hash));
        }
        if let Ok(Some(hash)) = self.get_relationship(block_id2, block_id1) {
            return Ok(Some(hash));
        }

        loop {
            //get header by block_id
            match self.get_block_header_by_hash(parent_id1)? {
                Some(header) => {
                    parent_id1 = header.parent_hash();
                    ensure!(parent_id1 != HashValue::zero(), "invalid block id is zero.");
                    match self.get_sons(parent_id1) {
                        Ok(sons1) => {
                            debug!("parent: {:?}, sons1 : {:?}", parent_id1, sons1);
                            if sons1.len() > 1 {
                                // get parent2 from block2
                                loop {
                                    ensure!(
                                        parent_id2 != HashValue::zero(),
                                        "invalid block id is zero."
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

    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        //get storage current len
        let max_number = self.number_store.get_len()?;
        self.get_block_by_number(max_number - 1)
    }

    pub fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.header_store.get(block_id)
    }

    pub fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.get(block_id)
    }

    pub fn get_block_state(&self, block_id: HashValue) -> Result<Option<BlockState>> {
        Ok(
            if let Some(storage_block) = self.block_store.get(block_id)? {
                let (_, block_state) = storage_block.into();
                Some(block_state)
            } else {
                None
            },
        )
    }

    pub fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        match self.number_store.get(number)? {
            Some(block_id) => self.get_block_header_by_hash(block_id),
            None => bail!("can't find block header by number:{}", number),
        }
    }

    pub fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        match self.number_store.get(number)? {
            Some(block_id) => self.get(block_id),
            None => Ok(None),
        }
    }

    pub fn get_transactions(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        match self.block_txns_store.get(block_id) {
            Ok(Some(transactions)) => Ok(transactions),
            _ => bail!("can't find block's transaction: {:?}", block_id),
        }
    }

    /// get txn info ids for `block_id`.
    /// return None, if block_id not exists.
    pub fn get_transaction_info_ids(&self, block_id: HashValue) -> Result<Option<Vec<HashValue>>> {
        self.block_txn_infos_store.get(block_id)
    }

    pub fn put_transactions(
        &self,
        block_id: HashValue,
        transactions: Vec<HashValue>,
    ) -> Result<()> {
        self.block_txns_store.put(block_id, transactions)
    }

    pub fn put_transaction_infos(
        &self,
        block_id: HashValue,
        txn_info_ids: Vec<HashValue>,
    ) -> Result<()> {
        self.block_txn_infos_store.put(block_id, txn_info_ids)
    }

    fn get_relationship(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        if let Ok(sons) = self.get_sons(block_id1) {
            if sons.contains(&block_id2) {
                return Ok(Some(block_id1));
            }
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
        trace!("put son:{}, {}", parent_hash, son_hash);
        match self.get_sons(parent_hash) {
            Ok(mut vec_hash) => {
                debug!("branch block:{}, {:?}", parent_hash, vec_hash);
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
