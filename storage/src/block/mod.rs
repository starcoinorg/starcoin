// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::define_storage;
use crate::storage::{CodecKVStore, CodecWriteBatch, StorageInstance, ValueCodec};
use crate::{
    BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME_V2,
    BLOCK_PREFIX_NAME, BLOCK_PREFIX_NAME_V2, BLOCK_TRANSACTIONS_PREFIX_NAME,
    BLOCK_TRANSACTION_INFOS_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME_V2,
};
use anyhow::{bail, Result};
use bcs_ext::{BCSCodec, Sample};
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::{Block, BlockBody, BlockHeader, OldBlock, OldBlockHeader};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct OldFailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
}

impl From<(Block, Option<PeerId>, String)> for OldFailedBlock {
    fn from(block: (Block, Option<PeerId>, String)) -> Self {
        Self {
            block: block.0,
            peer_id: block.1,
            failed: block.2,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<(Block, Option<PeerId>, String, String)> for OldFailedBlock {
    fn into(self) -> (Block, Option<PeerId>, String, String) {
        (self.block, self.peer_id, self.failed, "".to_string())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
    version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename(deserialize = "FailedBlock"))]
pub struct OldFailedBlockV2 {
    block: OldBlock,
    peer_id: Option<PeerId>,
    failed: String,
    version: String,
}

impl From<OldFailedBlockV2> for FailedBlock {
    fn from(value: OldFailedBlockV2) -> Self {
        Self {
            block: value.block.into(),
            peer_id: value.peer_id,
            failed: value.failed,
            version: value.version,
        }
    }
}

impl From<FailedBlock> for OldFailedBlockV2 {
    fn from(value: FailedBlock) -> Self {
        Self {
            block: value.block.into(),
            peer_id: value.peer_id,
            failed: value.failed,
            version: value.version,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<(Block, Option<PeerId>, String, String)> for FailedBlock {
    fn into(self) -> (Block, Option<PeerId>, String, String) {
        (self.block, self.peer_id, self.failed, self.version)
    }
}

impl From<(Block, Option<PeerId>, String, String)> for FailedBlock {
    fn from(block: (Block, Option<PeerId>, String, String)) -> Self {
        Self {
            block: block.0,
            peer_id: block.1,
            failed: block.2,
            version: block.3,
        }
    }
}

impl Sample for FailedBlock {
    fn sample() -> Self {
        Self {
            block: Block::sample(),
            peer_id: Some(PeerId::random()),
            failed: "Unknown reason".to_string(),
            version: "Unknown version".to_string(),
        }
    }
}

impl FailedBlock {
    pub fn random() -> Self {
        Self {
            block: Block::random(),
            peer_id: Some(PeerId::random()),
            failed: "Unknown reason".to_string(),
            version: "Unknown version".to_string(),
        }
    }
}

define_storage!(BlockInnerStorage, HashValue, Block, BLOCK_PREFIX_NAME_V2);
define_storage!(
    BlockHeaderStorage,
    HashValue,
    BlockHeader,
    BLOCK_HEADER_PREFIX_NAME_V2
);
define_storage!(OldBlockInnerStorage, HashValue, OldBlock, BLOCK_PREFIX_NAME);
define_storage!(
    OldBlockHeaderStorage,
    HashValue,
    OldBlockHeader,
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
    FailedBlockStorage,
    HashValue,
    FailedBlock,
    FAILED_BLOCK_PREFIX_NAME_V2
);

define_storage!(
    OldFailedBlockStorage,
    HashValue,
    OldFailedBlockV2,
    FAILED_BLOCK_PREFIX_NAME
);

#[derive(Clone)]
pub struct BlockStorage {
    block_store: BlockInnerStorage,
    pub(crate) header_store: BlockHeaderStorage,
    body_store: BlockBodyStorage,
    block_txns_store: BlockTransactionsStorage,
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

impl ValueCodec for OldBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for OldBlockHeader {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for Vec<BlockHeader> {
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

impl ValueCodec for OldFailedBlock {
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

impl ValueCodec for OldFailedBlockV2 {
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
        let (header, _) = block.clone().into_inner();
        //save header
        self.save_header(header)?;
        // save block , no need body
        // self.save_body(block_id, body)?;
        //save block
        self.save(block)
    }
    pub fn delete_block(&self, block_id: HashValue) -> Result<()> {
        self.header_store.remove(block_id)?;
        self.body_store.remove(block_id)?;
        self.block_store.remove(block_id)?;
        self.block_txns_store.remove(block_id)?;
        self.block_txn_infos_store.remove(block_id)
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

    /// get txn info ids for `block_id`.
    /// return None, if block_id not exists.
    pub fn get_transaction_info_ids(&self, block_id: HashValue) -> Result<Option<Vec<HashValue>>> {
        self.block_txn_infos_store.get(block_id)
    }

    pub fn put_transaction_ids(
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

    pub fn save_failed_block(
        &self,
        block_id: HashValue,
        block: Block,
        peer_id: Option<PeerId>,
        failed: String,
        version: String,
    ) -> Result<()> {
        self.failed_block_storage
            .put(block_id, (block, peer_id, failed, version).into())
    }

    pub fn delete_failed_block(&self, block_id: HashValue) -> Result<()> {
        self.failed_block_storage.remove(block_id)
    }

    pub fn get_failed_block_by_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<(Block, Option<PeerId>, String, String)>> {
        let res = self.failed_block_storage.get_raw(block_id)?;
        match res {
            Some(res) => {
                let result = OldFailedBlock::decode_value(res.as_slice());
                if result.is_ok() {
                    return Ok(Some(result?.into()));
                }
                let result = FailedBlock::decode_value(res.as_slice())?;
                Ok(Some(result.into()))
            }
            None => Ok(None),
        }
    }

    pub fn save_old_failed_block(
        &self,
        block_id: HashValue,
        block: Block,
        peer_id: Option<PeerId>,
        failed: String,
    ) -> Result<()> {
        let old_block: OldFailedBlock = (block, peer_id, failed).into();
        self.failed_block_storage
            .put_raw(block_id, old_block.encode_value()?)
    }

    fn upgrade_header_store(
        old_header_store: OldBlockHeaderStorage,
        header_store: BlockHeaderStorage,
        batch_size: usize,
    ) -> Result<usize> {
        let mut total_size: usize = 0;
        let mut old_header_iter = old_header_store.iter()?;
        old_header_iter.seek_to_first();
        let mut to_deleted = Some(CodecWriteBatch::<HashValue, OldBlockHeader>::new());
        let mut to_put = Some(CodecWriteBatch::<HashValue, BlockHeader>::new());
        let mut item_count = 0usize;
        for item in old_header_iter {
            let (id, old_header) = item?;
            let header: BlockHeader = old_header.into();
            to_deleted
                .as_mut()
                .unwrap()
                .delete(id)
                .expect("should never fail");
            to_put
                .as_mut()
                .unwrap()
                .put(id, header)
                .expect("should never fail");
            item_count += 1;
            if item_count == batch_size {
                total_size = total_size.saturating_add(item_count);
                item_count = 0;
                old_header_store.write_batch(to_deleted.take().unwrap())?;
                header_store.write_batch(to_put.take().unwrap())?;

                to_deleted = Some(CodecWriteBatch::<HashValue, OldBlockHeader>::new());
                to_put = Some(CodecWriteBatch::<HashValue, BlockHeader>::new());
            }
        }
        if item_count != 0 {
            total_size = total_size.saturating_add(item_count);
            old_header_store.write_batch(to_deleted.take().unwrap())?;
            header_store.write_batch(to_put.take().unwrap())?;
        }

        Ok(total_size)
    }

    fn upgrade_block_store(
        old_block_store: OldBlockInnerStorage,
        block_store: BlockInnerStorage,
        batch_size: usize,
    ) -> Result<usize> {
        let mut total_size: usize = 0;
        let mut old_block_iter = old_block_store.iter()?;
        old_block_iter.seek_to_first();

        let mut to_delete = Some(CodecWriteBatch::new());
        let mut to_put = Some(CodecWriteBatch::new());
        let mut item_count = 0;

        for item in old_block_iter {
            let (id, old_block) = item?;
            let block: Block = old_block.into();
            to_delete
                .as_mut()
                .unwrap()
                .delete(id)
                .expect("should never fail");
            to_put
                .as_mut()
                .unwrap()
                .put(id, block)
                .expect("should never fail");

            item_count += 1;
            if item_count == batch_size {
                total_size = total_size.saturating_add(item_count);
                item_count = 0;
                old_block_store
                    .write_batch(to_delete.take().unwrap())
                    .expect("should never fail");
                block_store
                    .write_batch(to_put.take().unwrap())
                    .expect("should never fail");

                to_delete = Some(CodecWriteBatch::new());
                to_put = Some(CodecWriteBatch::new());
            }
        }
        if item_count != 0 {
            total_size = total_size.saturating_add(item_count);
            old_block_store
                .write_batch(to_delete.take().unwrap())
                .expect("should never fail");
            block_store
                .write_batch(to_put.take().unwrap())
                .expect("should never fail");
        }

        Ok(total_size)
    }

    fn upgrade_failed_block_store(
        old_failed_block_store: OldFailedBlockStorage,
        failed_block_store: FailedBlockStorage,
        batch_size: usize,
    ) -> Result<usize> {
        let mut total_size: usize = 0;
        let mut old_failed_block_iter = old_failed_block_store.iter()?;
        old_failed_block_iter.seek_to_first();

        let mut to_delete = Some(CodecWriteBatch::new());
        let mut to_put = Some(CodecWriteBatch::new());
        let mut item_count = 0;

        for item in old_failed_block_iter {
            let (id, old_block) = item?;
            let block: FailedBlock = old_block.into();
            to_delete
                .as_mut()
                .unwrap()
                .delete(id)
                .expect("should never fail");
            to_put
                .as_mut()
                .unwrap()
                .put(id, block)
                .expect("should never fail");

            item_count += 1;
            if item_count == batch_size {
                total_size = total_size.saturating_add(item_count);
                item_count = 0;
                old_failed_block_store
                    .write_batch(to_delete.take().unwrap())
                    .expect("should never fail");
                failed_block_store
                    .write_batch(to_put.take().unwrap())
                    .expect("should never fail");

                to_delete = Some(CodecWriteBatch::new());
                to_put = Some(CodecWriteBatch::new());
            }
        }
        if item_count != 0 {
            total_size = total_size.saturating_add(item_count);
            old_failed_block_store
                .write_batch(to_delete.take().unwrap())
                .expect("should never fail");
            failed_block_store
                .write_batch(to_put.take().unwrap())
                .expect("should never fail");
        }

        Ok(total_size)
    }

    pub fn upgrade_block_header(instance: StorageInstance) -> Result<()> {
        const BATCH_SIZE: usize = 1000usize;
        let old_header_store = OldBlockHeaderStorage::new(instance.clone());
        let header_store = BlockHeaderStorage::new(instance.clone());

        let total_size = Self::upgrade_header_store(old_header_store, header_store, BATCH_SIZE)?;
        info!("upgraded {total_size} blocks in block_header_store");

        let old_block_store = OldBlockInnerStorage::new(instance.clone());
        let block_store = BlockInnerStorage::new(instance.clone());

        let total_blocks = Self::upgrade_block_store(old_block_store, block_store, BATCH_SIZE)?;
        info!("upgraded {total_blocks} blocks in block_store");

        let old_failed_block_store = OldFailedBlockStorage::new(instance.clone());
        let failed_block_store = FailedBlockStorage::new(instance);

        let total_failed_blocks = Self::upgrade_failed_block_store(
            old_failed_block_store,
            failed_block_store,
            BATCH_SIZE,
        )?;
        info!("upgraded {total_failed_blocks} blocks in failed_block_store");

        Ok(())
    }
}
