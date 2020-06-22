// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator::AccumulatorStorage;
use crate::block::BlockStorage;
use crate::block_info::{BlockInfoStorage, BlockInfoStore};
use crate::state_node::StateStorage;
use crate::storage::{ColumnFamilyName, InnerStorage, KVStore, StorageInstance};
use crate::transaction::TransactionStorage;
use crate::transaction_info::TransactionInfoStorage;
use anyhow::{bail, ensure, format_err, Error, Result};
use crypto::HashValue;
use once_cell::sync::Lazy;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{
    AccumulatorNode, AccumulatorReader, AccumulatorTreeStore, AccumulatorWriter,
};
use starcoin_state_store_api::{StateNode, StateNodeStore};
use starcoin_types::block::BlockState;
use starcoin_types::transaction::Transaction;
use starcoin_types::{
    block::{Block, BlockBody, BlockHeader, BlockInfo},
    startup_info::StartupInfo,
    transaction::TransactionInfo,
};
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::Arc;

pub mod accumulator;
pub mod batch;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod db_storage;
mod metrics;
pub mod state_node;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod transaction;
pub mod transaction_info;

#[macro_use]
pub mod storage_macros;
pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node";
pub const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
pub const BLOCK_HEADER_PREFIX_NAME: ColumnFamilyName = "block_header";
pub const BLOCK_SONS_PREFIX_NAME: ColumnFamilyName = "block_sons";
pub const BLOCK_BODY_PREFIX_NAME: ColumnFamilyName = "block_body";
pub const BLOCK_NUM_PREFIX_NAME: ColumnFamilyName = "block_num";
pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub const BLOCK_TRANSACTIONS_PREFIX_NAME: ColumnFamilyName = "block_txns";
pub const BLOCK_TRANSACTION_INFOS_PREFIX_NAME: ColumnFamilyName = "block_txn_infos";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const STARTUP_INFO_PREFIX_NAME: ColumnFamilyName = "startup_info";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_INFO_PREFIX_NAME: ColumnFamilyName = "transaction_info";
pub const BRANCH_PREFIX_NAME: ColumnFamilyName = "branch";
///db storage use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
pub static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_SONS_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_NUM_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        STARTUP_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        BRANCH_PREFIX_NAME,
    ]
});

pub trait BlockStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_headers(&self) -> Result<Vec<HashValue>>;

    fn save_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
        block_id: HashValue,
    ) -> Result<()>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_block_state(&self, block_id: HashValue) -> Result<Option<BlockState>>;

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>>;

    fn get_branch_number(&self, branch_id: HashValue, number: u64) -> Result<Option<HashValue>>;

    fn get_number(&self, number: u64) -> Result<Option<HashValue>>;

    fn commit_block(&self, block: Block, state: BlockState) -> Result<()>;

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>>;

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>>;

    fn get_latest_block(&self) -> Result<Option<Block>>;

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

    fn save_block_transactions(
        &self,
        block_id: HashValue,
        transactions: Vec<HashValue>,
    ) -> Result<()>;

    /// get txn info id list for block `block_id`.
    /// If block_id doesn't exists, return error.
    fn get_block_txn_info_ids(&self, block_id: HashValue) -> Result<Vec<HashValue>>;

    fn save_block_txn_info_ids(
        &self,
        block_id: HashValue,
        txn_info_ids: Vec<HashValue>,
    ) -> Result<()>;
}

pub trait TransactionInfoStore {
    fn get_transaction_info(&self, txn_info_hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn save_transaction_info(&self, txn_info: TransactionInfo) -> Result<()> {
        self.save_transaction_infos(vec![txn_info])
    }
    fn save_transaction_infos(&self, vec_txn_info: Vec<TransactionInfo>) -> Result<()>;
}

pub trait TransactionStore {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>>;
    fn save_transaction(&self, txn_info: Transaction) -> Result<()>;
    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()>;
}

pub struct Storage {
    transaction_info_storage: TransactionInfoStorage,
    transaction_storage: TransactionStorage,
    block_storage: BlockStorage,
    state_node_storage: StateStorage,
    accumulator_storage: AccumulatorStorage,
    block_info_storage: BlockInfoStorage,
    startup_info_storage: Arc<dyn KVStore>,
}

impl Storage {
    pub fn new(instance: StorageInstance) -> Result<Self> {
        Ok(Self {
            transaction_info_storage: TransactionInfoStorage::new(instance.clone()),
            transaction_storage: TransactionStorage::new(instance.clone()),
            block_storage: BlockStorage::new(instance.clone()),
            state_node_storage: StateStorage::new(instance.clone()),
            accumulator_storage: AccumulatorStorage::new(instance.clone()),
            block_info_storage: BlockInfoStorage::new(instance.clone()),
            startup_info_storage: Arc::new(InnerStorage::new(
                instance.clone(),
                STARTUP_INFO_PREFIX_NAME,
            )),
        })
    }
}

impl StateNodeStore for Storage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        self.state_node_storage.get(*hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_node_storage.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<(), Error> {
        self.state_node_storage.write_nodes(nodes)
    }
}

impl BlockStore for Storage {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.startup_info_storage
            .get(STARTUP_INFO_PREFIX_NAME.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.startup_info_storage.put(
            STARTUP_INFO_PREFIX_NAME.as_bytes().to_vec(),
            startup_info.try_into()?,
        )
    }

    fn get_headers(&self) -> Result<Vec<HashValue>> {
        self.block_storage.get_headers()
    }

    fn save_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
        block_id: HashValue,
    ) -> Result<(), Error> {
        self.block_storage
            .save_branch_number(branch_id, number, block_id)
    }

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_storage.get(block_id)
    }

    fn get_block_state(&self, block_id: HashValue) -> Result<Option<BlockState>> {
        self.block_storage.get_block_state(block_id)
    }

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.block_storage.get_body(block_id)
    }

    fn get_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<HashValue>, Error> {
        self.block_storage.get_branch_number(branch_id, number)
    }

    fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.block_storage.get_number(number)
    }

    fn commit_block(&self, block: Block, state: BlockState) -> Result<()> {
        self.block_storage.commit_block(block, state)
    }

    fn get_branch_hashes(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        self.block_storage.get_branch_hashes(block_id)
    }

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>> {
        self.block_storage.get_latest_block_header()
    }

    fn get_latest_block(&self) -> Result<Option<Block>> {
        self.block_storage.get_latest_block()
    }

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.block_storage.get_block_header_by_hash(block_id)
    }

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_storage.get_block_by_hash(block_id)
    }

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.block_storage.get_block_header_by_number(number)
    }

    fn get_header_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<BlockHeader>, Error> {
        self.block_storage
            .get_header_by_branch_number(branch_id, number)
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.block_storage.get_block_by_number(number)
    }

    fn get_block_by_branch_number(
        &self,
        branch_id: HashValue,
        number: u64,
    ) -> Result<Option<Block>, Error> {
        self.block_storage
            .get_block_by_branch_number(branch_id, number)
    }

    fn get_common_ancestor(
        &self,
        block_id1: HashValue,
        block_id2: HashValue,
    ) -> Result<Option<HashValue>> {
        self.block_storage.get_common_ancestor(block_id1, block_id2)
    }

    fn save_block_transactions(
        &self,
        block_id: HashValue,
        transactions: Vec<HashValue>,
    ) -> Result<()> {
        self.block_storage.put_transactions(block_id, transactions)
    }

    fn get_block_txn_info_ids(&self, block_id: HashValue) -> Result<Vec<HashValue>> {
        self.block_storage
            .get_transaction_info_ids(block_id)
            .and_then(|d| {
                d.ok_or_else(|| format_err!("can't find txn info id list for block {}", block_id))
            })
    }

    fn save_block_txn_info_ids(
        &self,
        block_id: HashValue,
        txn_info_ids: Vec<HashValue>,
    ) -> Result<()> {
        self.block_storage
            .put_transaction_infos(block_id, txn_info_ids)
    }
}

impl AccumulatorTreeStore for Storage {}
impl AccumulatorReader for Storage {
    ///get node by node hash
    fn get_node(
        &self,
        store_type: AccumulatorStoreType,
        hash: HashValue,
    ) -> Result<Option<AccumulatorNode>> {
        self.accumulator_storage.get_node(store_type, hash)
    }

    fn multiple_get(
        &self,
        _store_type: AccumulatorStoreType,
        _hash_vec: Vec<HashValue>,
    ) -> Result<Vec<AccumulatorNode>, Error> {
        unimplemented!()
    }
}

impl AccumulatorWriter for Storage {
    /// save node
    fn save_node(&self, store_type: AccumulatorStoreType, node: AccumulatorNode) -> Result<()> {
        self.accumulator_storage.save_node(store_type, node)
    }

    fn save_nodes(
        &self,
        store_type: AccumulatorStoreType,
        nodes: Vec<AccumulatorNode>,
    ) -> Result<(), Error> {
        self.accumulator_storage.save_nodes(store_type, nodes)
    }

    ///delete node
    fn delete_nodes(
        &self,
        store_type: AccumulatorStoreType,
        node_hash_vec: Vec<HashValue>,
    ) -> Result<()> {
        self.accumulator_storage
            .delete_nodes(store_type, node_hash_vec)
    }
}

impl BlockInfoStore for Storage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<(), Error> {
        self.block_info_storage.put(block_info.block_id, block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>, Error> {
        self.block_info_storage.get(hash_value)
    }
}

impl TransactionInfoStore for Storage {
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>> {
        self.transaction_info_storage.get(txn_hash)
    }

    fn save_transaction_infos(&self, vec_txn_info: Vec<TransactionInfo>) -> Result<(), Error> {
        self.transaction_info_storage
            .save_transaction_infos(vec_txn_info)
    }
}

impl TransactionStore for Storage {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.transaction_storage.get(txn_hash)
    }

    fn save_transaction(&self, txn: Transaction) -> Result<(), Error> {
        self.transaction_storage.put(txn.id(), txn)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<(), Error> {
        self.transaction_storage.save_transaction_batch(txn_vec)
    }
}

/// Chain storage define
pub trait Store:
    StateNodeStore
    + BlockStore
    + AccumulatorTreeStore
    + BlockInfoStore
    + TransactionStore
    + TransactionInfoStore
    + IntoSuper<dyn StateNodeStore>
    + IntoSuper<dyn AccumulatorTreeStore>
{
    fn get_transaction_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>> {
        let txn_infos = self.get_block_txn_info_ids(block_id)?;
        match txn_infos.get(idx as usize) {
            None => Ok(None),
            Some(info_hash) => self.get_transaction_info(*info_hash),
        }
    }

    fn get_block_transaction_infos(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<TransactionInfo>, Error> {
        let txn_info_ids = self.get_block_txn_info_ids(block_id)?;
        let mut txn_infos = vec![];
        for hash in txn_info_ids {
            match self.get_transaction_info(hash)? {
                Some(info) => txn_infos.push(info),
                None => bail!(
                    "invalid state: txn info {} of block {} should exist",
                    hash,
                    block_id
                ),
            }
        }
        Ok(txn_infos)
    }
}

pub trait IntoSuper<Super: ?Sized> {
    fn as_super(&self) -> &Super;
    fn as_super_mut(&mut self) -> &mut Super;
    fn into_super(self: Box<Self>) -> Box<Super>;
    fn into_super_arc(self: Arc<Self>) -> Arc<Super>;
}

impl<'a, T: 'a + StateNodeStore> IntoSuper<dyn StateNodeStore + 'a> for T {
    fn as_super(&self) -> &(dyn StateNodeStore + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn StateNodeStore + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn StateNodeStore + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn StateNodeStore + 'a> {
        self
    }
}

impl<'a, T: 'a + AccumulatorTreeStore> IntoSuper<dyn AccumulatorTreeStore + 'a> for T {
    fn as_super(&self) -> &(dyn AccumulatorTreeStore + 'a) {
        self
    }
    fn as_super_mut(&mut self) -> &mut (dyn AccumulatorTreeStore + 'a) {
        self
    }
    fn into_super(self: Box<Self>) -> Box<dyn AccumulatorTreeStore + 'a> {
        self
    }
    fn into_super_arc(self: Arc<Self>) -> Arc<dyn AccumulatorTreeStore + 'a> {
        self
    }
}

impl Store for Storage {}

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
