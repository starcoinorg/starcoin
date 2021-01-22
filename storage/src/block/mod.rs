// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::define_storage;
use crate::storage::{CodecKVStore, StorageInstance, ValueCodec};
use crate::{
    BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME, BLOCK_PREFIX_NAME,
    BLOCK_TRANSACTIONS_PREFIX_NAME, BLOCK_TRANSACTION_INFOS_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME,
    TRANSACTION_BLOCK_PREFIX_NAME,
};
use anyhow::{bail, Result};
use bcs_ext::BCSCodec;
use crypto::HashValue;
use logger::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_types::block::{Block, BlockBody, BlockHeader};
use starcoin_types::peer_info::PeerId;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
}

impl Into<(Block, Option<PeerId>, String)> for FailedBlock {
    fn into(self) -> (Block, Option<PeerId>, String) {
        (self.block, self.peer_id, self.failed)
    }
}

impl From<(Block, Option<PeerId>, String)> for FailedBlock {
    fn from(block: (Block, Option<PeerId>, String)) -> Self {
        Self {
            block: block.0,
            peer_id: block.1,
            failed: block.2,
        }
    }
}

define_storage!(BlockInnerStorage, HashValue, Block, BLOCK_PREFIX_NAME);
define_storage!(
    BlockHeaderStorage,
    HashValue,
    BlockHeader,
    BLOCK_HEADER_PREFIX_NAME
);
define_storage!(
    BlockBodyStorage,
    HashValue,
    BlockBody,
    BLOCK_BODY_PREFIX_NAME
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
define_storage!(
    TransactionBlockStorage,
    HashValue,
    HashValue,
    TRANSACTION_BLOCK_PREFIX_NAME
);
define_storage!(
    FailedBlockStorage,
    HashValue,
    FailedBlock,
    FAILED_BLOCK_PREFIX_NAME
);

#[derive(Clone)]
pub struct BlockStorage {
    block_store: BlockInnerStorage,
    header_store: BlockHeaderStorage,
    body_store: BlockBodyStorage,
    block_txns_store: BlockTransactionsStorage,
    txn_block_store: TransactionBlockStorage,
    block_txn_infos_store: BlockTransactionInfosStorage,
    failed_block_storage: FailedBlockStorage,
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

impl ValueCodec for FailedBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl BlockStorage {
    pub fn new(instance: StorageInstance) -> Self {
        BlockStorage {
            block_store: BlockInnerStorage::new(instance.clone()),
            header_store: BlockHeaderStorage::new(instance.clone()),
            body_store: BlockBodyStorage::new(instance.clone()),
            block_txns_store: BlockTransactionsStorage::new(instance.clone()),
            txn_block_store: TransactionBlockStorage::new(instance.clone()),
            block_txn_infos_store: BlockTransactionInfosStorage::new(instance.clone()),
            failed_block_storage: FailedBlockStorage::new(instance),
        }
    }
    pub fn save(&self, block: Block) -> Result<()> {
        debug!(
            "insert block:{}, parent:{}",
            block.header().id(),
            block.header().parent_hash()
        );
        let block_id = block.header().id();
        self.block_store.put(block_id, block)
    }

    pub fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.header_store.put(header.id(), header)
    }

    pub fn get_headers(&self) -> Result<Vec<HashValue>> {
        let mut key_hashes = vec![];
        for hash in self.header_store.keys()? {
            key_hashes.push(hash)
        }
        Ok(key_hashes)
    }

    pub fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()> {
        self.body_store.put(block_id, body)
    }

    pub fn get(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get(block_id)
    }

    pub fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        Ok(self.block_store.multiple_get(ids)?.into_iter().collect())
    }

    pub fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.body_store.get(block_id)
    }

    pub fn commit_block(&self, block: Block) -> Result<()> {
        let (header, body) = block.clone().into_inner();
        //save header
        let block_id = header.id();
        self.save_header(header)?;
        //save body
        self.save_body(block_id, body)?;
        //save block cache
        self.save(block)
    }

    pub fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.header_store.get(block_id)
    }

    pub fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.get(block_id)
    }

    pub fn get_transactions(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        match self.block_txns_store.get(block_id) {
            Ok(Some(transactions)) => Ok(transactions),
            _ => bail!("can't find block's transaction: {:?}", block_id),
        }
    }

    /// get the block id which contains the given `tnx_id`
    pub fn get_transaction_block(&self, txn_id: HashValue) -> Result<Option<HashValue>> {
        self.txn_block_store.get(txn_id)
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
        self.block_txns_store.put(block_id, transactions.clone())?;
        // add a reverse mapping
        let write_batch = transactions.into_iter().map(|t| (t, block_id)).collect();
        self.txn_block_store.put_all(write_batch)
    }

    pub fn put_transaction_infos(
        &self,
        block_id: HashValue,
        txn_info_ids: Vec<HashValue>,
    ) -> Result<()> {
        self.block_txn_infos_store.put(block_id, txn_info_ids)
    }

    pub fn save_failed_block(
        &self,
        block_id: HashValue,
        block: Block,
        peer_id: Option<PeerId>,
        failed: String,
    ) -> Result<()> {
        self.failed_block_storage
            .put(block_id, (block, peer_id, failed).into())
    }

    pub fn get_failed_block_by_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<(Block, Option<PeerId>, String)>> {
        match self.failed_block_storage.get(block_id)? {
            Some(failed_block) => Ok(Some(failed_block.into())),
            None => Ok(None),
        }
    }
}
