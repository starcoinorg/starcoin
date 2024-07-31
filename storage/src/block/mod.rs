// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    define_storage,
    storage::{CodecKVStore, StorageInstance, ValueCodec},
    BLOCK_BODY_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME, BLOCK_HEADER_PREFIX_NAME_V2,
    BLOCK_PREFIX_NAME, BLOCK_PREFIX_NAME_V2, BLOCK_TRANSACTIONS_PREFIX_NAME,
    BLOCK_TRANSACTION_INFOS_PREFIX_NAME, DAG_SYNC_BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME,
    FAILED_BLOCK_PREFIX_NAME_V2,
};
use anyhow::{bail, Result};
use bcs_ext::{BCSCodec, Sample};
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_types::block::{Block, BlockBody, BlockHeader, LegacyBlock, LegacyBlockHeader};

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
pub struct DagSyncBlock {
    pub block: Block,
    pub children: Vec<HashValue>,
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
    block: LegacyBlock,
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
define_storage!(
    OldBlockInnerStorage,
    HashValue,
    LegacyBlock,
    BLOCK_PREFIX_NAME
);
define_storage!(
    OldBlockHeaderStorage,
    HashValue,
    LegacyBlockHeader,
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
    DagSyncBlockStorage,
    HashValue,
    DagSyncBlock,
    DAG_SYNC_BLOCK_PREFIX_NAME
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
    dag_sync_block_storage: DagSyncBlockStorage,
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

impl ValueCodec for LegacyBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for LegacyBlockHeader {
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

impl ValueCodec for DagSyncBlock {
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
        Self {
            block_store: BlockInnerStorage::new(instance.clone()),
            header_store: BlockHeaderStorage::new(instance.clone()),
            body_store: BlockBodyStorage::new(instance.clone()),
            block_txns_store: BlockTransactionsStorage::new(instance.clone()),
            block_txn_infos_store: BlockTransactionInfosStorage::new(instance.clone()),
            failed_block_storage: FailedBlockStorage::new(instance.clone()),
            dag_sync_block_storage: DagSyncBlockStorage::new(instance),
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

    pub fn save_dag_sync_block(&self, block: DagSyncBlock) -> Result<()> {
        self.dag_sync_block_storage.put(block.block.id(), block)
    }

    pub fn delete_dag_sync_block(&self, block_id: HashValue) -> Result<()> {
        self.dag_sync_block_storage.remove(block_id)
    }

    pub fn delete_all_dag_sync_blocks(&self) -> Result<()> {
        self.dag_sync_block_storage.remove_all()
    }

    pub fn get_dag_sync_block(&self, block_id: HashValue) -> Result<Option<DagSyncBlock>> {
        self.dag_sync_block_storage.get(block_id)
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
}
