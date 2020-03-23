// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator::AccumulatorStore;
use crate::block::BlockStore;
use crate::block_info::{BlockInfoStorage, BlockInfoStore};
use crate::state_node::StateNodeStorage;
use crate::storage::{ColumnFamilyName, InnerRepository, Repository, Storage};
use crate::transaction_info::TransactionInfoStore;
use anyhow::{ensure, Error, Result};
use crypto::HashValue;
use starcoin_accumulator::{
    node_index::NodeIndex, AccumulatorNode, AccumulatorNodeReader, AccumulatorNodeStore,
    AccumulatorNodeWriter,
};
use state_tree::{StateNode, StateNodeStore};
use std::convert::TryInto;
use std::sync::Arc;
use types::{
    block::{Block, BlockBody, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};

pub mod accumulator;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod db_storage;
pub mod state_node;
pub mod storage;
mod tests;
pub mod transaction_info;

pub const ACCUMULATOR_INDEX_PREFIX_NAME: ColumnFamilyName = "acc_index";
pub const ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node";
pub const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
pub const BLOCK_HEADER_PREFIX_NAME: ColumnFamilyName = "block_header";
pub const BLOCK_SONS_PREFIX_NAME: ColumnFamilyName = "block_sons";
pub const BLOCK_BODY_PREFIX_NAME: ColumnFamilyName = "block_body";
pub const BLOCK_NUM_PREFIX_NAME: ColumnFamilyName = "block_num";
pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const STARTUP_INFO_PREFIX_NAME: ColumnFamilyName = "startup_info";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction_info";

pub trait BlockStorageOp {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn save(&self, block: Block) -> Result<()>;

    fn save_header(&self, header: BlockHeader) -> Result<()>;

    fn get_headers(&self) -> Result<Vec<HashValue>>;

    fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()>;

    fn save_number(&self, number: BlockNumber, block_id: HashValue) -> Result<()>;

    fn save_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
        block_id: HashValue,
    ) -> Result<()>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>>;

    fn get_branch_number(&self, branch_id: HashValue, number: u64) -> Result<Option<HashValue>>;

    fn get_number(&self, number: u64) -> Result<Option<HashValue>>;

    fn commit_block(&self, block: Block) -> Result<()>;

    fn commit_branch_block(&self, branch_id: HashValue, block: Block) -> Result<()>;

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>>;

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>>;

    fn get_latest_block(&self) -> Result<Block>;

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>>;

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>>;

    fn get_header_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<BlockHeader>>;

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>>;

    fn get_block_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<Block>>;

    fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>>;
}

pub struct StarcoinStorage {
    transaction_info_store: TransactionInfoStore,
    pub block_store: BlockStore,
    state_node_store: StateNodeStorage,
    accumulator_store: AccumulatorStore,
    block_info_store: BlockInfoStore,
    startup_info_store: Arc<dyn Repository>,
}

impl StarcoinStorage {
    pub fn new(
        cache_storage: Arc<dyn InnerRepository>,
        db_storage: Arc<dyn InnerRepository>,
    ) -> Result<Self> {
        Ok(Self {
            transaction_info_store: TransactionInfoStore::new(Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                TRANSACTION_PREFIX_NAME,
            ))),
            block_store: BlockStore::two_new(cache_storage.clone(), db_storage.clone()),
            state_node_store: StateNodeStorage::new(Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                STATE_NODE_PREFIX_NAME,
            ))),
            accumulator_store: AccumulatorStore::two_new(cache_storage.clone(), db_storage.clone()),
            block_info_store: BlockInfoStore::new(Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                BLOCK_INFO_PREFIX_NAME,
            ))),
            startup_info_store: Arc::new(Storage::new(
                cache_storage.clone(),
                db_storage.clone(),
                STARTUP_INFO_PREFIX_NAME,
            )),
        })
    }
}

impl StateNodeStore for StarcoinStorage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        self.state_node_store.get(hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_node_store.put(key, node)
    }
}

impl BlockStorageOp for StarcoinStorage {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.startup_info_store
            .get(STARTUP_INFO_PREFIX_NAME.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.startup_info_store.put(
            STARTUP_INFO_PREFIX_NAME.as_bytes().to_vec(),
            startup_info.try_into()?,
        )
    }

    fn save(&self, block: Block) -> Result<()> {
        self.block_store.save(block)
    }

    fn save_header(&self, header: BlockHeader) -> Result<()> {
        self.block_store.save_header(header)
    }

    fn get_headers(&self) -> Result<Vec<HashValue>> {
        self.block_store.get_headers()
    }

    fn save_body(&self, block_id: HashValue, body: BlockBody) -> Result<()> {
        self.block_store.save_body(block_id, body)
    }

    fn save_number(&self, number: BlockNumber, block_id: HashValue) -> Result<()> {
        self.block_store.save_number(number, block_id)
    }

    fn save_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
        block_id: HashValue,
    ) -> Result<(), Error> {
        self.block_store
            .save_branch_number(branch_id, number, block_id)
    }

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get(block_id)
    }

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.block_store.get_body(block_id)
    }

    fn get_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<HashValue>, Error> {
        self.block_store.get_branch_number(branch_id, number)
    }

    fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.block_store.get_number(number)
    }

    fn commit_block(&self, block: Block) -> Result<()> {
        self.block_store.commit_block(block)
    }

    fn commit_branch_block(&self, branch_id: HashValue, block: Block) -> Result<()> {
        self.block_store.commit_branch_block(branch_id, block)
    }

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        self.block_store.get_branch_hashes(block_id)
    }

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>> {
        self.block_store.get_latest_block_header()
    }

    fn get_latest_block(&self) -> Result<Block> {
        self.block_store.get_latest_block()
    }

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.block_store.get_block_header_by_hash(block_id)
    }

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_store.get_block_by_hash(block_id)
    }

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.block_store.get_block_header_by_number(number)
    }

    fn get_header_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<BlockHeader>, Error> {
        self.block_store
            .get_header_by_branch_number(branch_id, number)
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.block_store.get_block_by_number(number)
    }

    fn get_block_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<Block>, Error> {
        self.block_store
            .get_block_by_branch_number(branch_id, number)
    }

    fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        self.block_store.get_common_ancestor(block_id1, block_id2)
    }
}

impl AccumulatorNodeStore for StarcoinStorage {}
impl AccumulatorNodeReader for StarcoinStorage {
    ///get node by node_index
    fn get(&self, index: NodeIndex) -> Result<Option<AccumulatorNode>> {
        self.accumulator_store.get(index)
    }
    ///get node by node hash
    fn get_node(&self, hash: HashValue) -> Result<Option<AccumulatorNode>> {
        self.accumulator_store.get_node(hash)
    }
}

impl AccumulatorNodeWriter for StarcoinStorage {
    /// save node index
    fn save(&self, index: NodeIndex, hash: HashValue) -> Result<()> {
        self.accumulator_store.save(index, hash)
    }
    /// save node
    fn save_node(&self, node: AccumulatorNode) -> Result<()> {
        self.accumulator_store.save_node(node)
    }
    ///delete node
    fn delete_nodes(&self, node_hash_vec: Vec<HashValue>) -> Result<()> {
        self.accumulator_store.delete_nodes(node_hash_vec)
    }
    ///delete larger index than one
    fn delete_nodes_index(&self, vec_index: Vec<NodeIndex>) -> Result<()> {
        self.accumulator_store.delete_nodes_index(vec_index)
    }
}

impl BlockInfoStorage for StarcoinStorage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<(), Error> {
        self.block_info_store.save(block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>, Error> {
        self.block_info_store.get(hash_value)
    }
}

//TODO should move this traits to traits crate?
/// Chain storage define
pub trait BlockChainStore:
    StateNodeStore + BlockStorageOp + AccumulatorNodeStore + BlockInfoStorage
{
}

impl BlockChainStore for StarcoinStorage {}

///ensure slice length
fn ensure_slice_len_eq(data: &[u8], len: usize) -> Result<()> {
    ensure!(
        data.len() == len,
        "Unexpected data len {}, expected {}.",
        data.len(),
        len,
    );
    Ok(())
}
