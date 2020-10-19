// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator::{
    AccumulatorStorage, BlockAccumulatorStorage, TransactionAccumulatorStorage,
};
use crate::block::BlockStorage;
use crate::block_info::{BlockInfoStorage, BlockInfoStore};
use crate::chain_info::ChainInfoStorage;
use crate::contract_event::ContractEventStorage;
use crate::state_node::StateStorage;
use crate::storage::{CodecKVStore, CodecWriteBatch, ColumnFamilyName, StorageInstance};
use crate::transaction::TransactionStorage;
use crate::transaction_info::{TransactionInfoHashStorage, TransactionInfoStorage};
use anyhow::{bail, format_err, Error, Result};
use crypto::HashValue;
use once_cell::sync::Lazy;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorTreeStore;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use starcoin_types::block::BlockState;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::transaction::Transaction;
use starcoin_types::{
    block::{Block, BlockBody, BlockHeader, BlockInfo},
    startup_info::StartupInfo,
    transaction::TransactionInfo,
};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub mod accumulator;
pub mod batch;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod chain_info;
pub mod contract_event;
pub mod db_storage;
pub mod errors;
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
pub const BLOCK_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_block";
pub const TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_transaction";
pub const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
pub const BLOCK_HEADER_PREFIX_NAME: ColumnFamilyName = "block_header";
pub const BLOCK_BODY_PREFIX_NAME: ColumnFamilyName = "block_body";
pub const BLOCK_NUM_PREFIX_NAME: ColumnFamilyName = "block_num";
pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub const BLOCK_TRANSACTIONS_PREFIX_NAME: ColumnFamilyName = "block_txns";
pub const BLOCK_TRANSACTION_INFOS_PREFIX_NAME: ColumnFamilyName = "block_txn_infos";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const CHAIN_INFO_PREFIX_NAME: ColumnFamilyName = "chain_info";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_INFO_PREFIX_NAME: ColumnFamilyName = "transaction_info";
pub const TRANSACTION_INFO_HASH_PREFIX_NAME: ColumnFamilyName = "transaction_info_hash";
pub const CONTRACT_EVENT_PREFIX_NAME: ColumnFamilyName = "contract_event";

///db storage use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
pub static VEC_PREFIX_NAME: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_NUM_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
    ]
});

pub trait BlockStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_headers(&self) -> Result<Vec<HashValue>>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;

    fn get_block_state(&self, block_id: HashValue) -> Result<Option<BlockState>>;

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>>;

    fn get_number(&self, number: u64) -> Result<Option<HashValue>>;

    fn commit_block(&self, block: Block, state: BlockState) -> Result<()>;

    fn get_latest_block_header(&self) -> Result<Option<BlockHeader>>;

    fn get_latest_block(&self) -> Result<Option<Block>>;

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>>;

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_block_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>>;

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>>;

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
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_transaction_info_by_hash(&self, txn_hash: HashValue) -> Result<Vec<TransactionInfo>>;
    /// Get transaction info ids by transaction hash, one transaction may be in different chain branch, so produce multiply transaction info.
    /// if not transaction info match with the `txn_hash`, return empty Vec.
    fn get_transaction_info_ids_by_hash(&self, txn_hash: HashValue) -> Result<Vec<HashValue>>;
    fn save_transaction_info(&self, txn_info: TransactionInfo) -> Result<()> {
        self.save_transaction_infos(vec![txn_info])
    }
    fn save_transaction_infos(&self, vec_txn_info: Vec<TransactionInfo>) -> Result<()>;
}
pub trait ContractEventStore {
    /// Save events by key `txn_info_id`.
    /// As txn_info has accumulator root of events, so there is a one-to-one mapping.
    fn save_contract_events(
        &self,
        txn_info_id: HashValue,
        events: Vec<ContractEvent>,
    ) -> Result<()>;

    /// Get events by `txn_info_id`.
    /// If the txn_info_id does not exists in the store, return `None`.
    /// NOTICE: *don't exists* is different with *no events produced*.
    fn get_contract_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>>;
}

pub trait TransactionStore {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>>;
    fn save_transaction(&self, txn_info: Transaction) -> Result<()>;
    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()>;
}

// TODO: remove Arc<dyn Store>, we can clone Storage directly.
#[derive(Clone)]
pub struct Storage {
    transaction_info_storage: TransactionInfoStorage,
    transaction_info_hash_storage: TransactionInfoHashStorage,
    transaction_storage: TransactionStorage,
    block_storage: BlockStorage,
    state_node_storage: StateStorage,
    block_accumulator_storage: AccumulatorStorage<BlockAccumulatorStorage>,
    transaction_accumulator_storage: AccumulatorStorage<TransactionAccumulatorStorage>,
    block_info_storage: BlockInfoStorage,
    event_storage: ContractEventStorage,
    chain_info_storage: ChainInfoStorage,
}

impl Storage {
    pub fn new(instance: StorageInstance) -> Result<Self> {
        Ok(Self {
            transaction_info_storage: TransactionInfoStorage::new(instance.clone()),
            transaction_info_hash_storage: TransactionInfoHashStorage::new(instance.clone()),
            transaction_storage: TransactionStorage::new(instance.clone()),
            block_storage: BlockStorage::new(instance.clone()),
            state_node_storage: StateStorage::new(instance.clone()),
            block_accumulator_storage: AccumulatorStorage::new_block_accumulator_storage(
                instance.clone(),
            ),
            transaction_accumulator_storage:
                AccumulatorStorage::new_transaction_accumulator_storage(instance.clone()),
            block_info_storage: BlockInfoStorage::new(instance.clone()),
            event_storage: ContractEventStorage::new(instance.clone()),
            chain_info_storage: ChainInfoStorage::new(instance),
        })
    }

    pub fn get_block_accumulator_storage(&self) -> AccumulatorStorage<BlockAccumulatorStorage> {
        self.block_accumulator_storage.clone()
    }

    pub fn get_transaction_accumulator_storage(
        &self,
    ) -> AccumulatorStorage<TransactionAccumulatorStorage> {
        self.transaction_accumulator_storage.clone()
    }
}

impl StateNodeStore for Storage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        self.state_node_storage.get(*hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_node_storage.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.state_node_storage.write_batch(batch)
    }
}

impl Display for Storage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.clone())
    }
}
impl Debug for Storage {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl BlockStore for Storage {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.chain_info_storage.get_startup_info()
    }

    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.chain_info_storage.save_startup_info(startup_info)
    }

    fn get_headers(&self) -> Result<Vec<HashValue>> {
        self.block_storage.get_headers()
    }

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_storage.get(block_id)
    }

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        self.block_storage.get_blocks(ids)
    }

    fn get_block_state(&self, block_id: HashValue) -> Result<Option<BlockState>> {
        self.block_storage.get_block_state(block_id)
    }

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.block_storage.get_body(block_id)
    }

    fn get_number(&self, number: u64) -> Result<Option<HashValue>> {
        self.block_storage.get_number(number)
    }

    fn commit_block(&self, block: Block, state: BlockState) -> Result<()> {
        self.block_storage.commit_block(block, state)
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

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.block_storage.get_block_by_number(number)
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

impl BlockInfoStore for Storage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<(), Error> {
        self.block_info_storage.put(block_info.block_id, block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>, Error> {
        self.block_info_storage.get(hash_value)
    }
}

impl TransactionInfoStore for Storage {
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<TransactionInfo>> {
        self.transaction_info_storage.get(id)
    }

    fn get_transaction_info_by_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<TransactionInfo>, Error> {
        let mut transaction_info_vec = vec![];
        if let Ok(Some(transaction_info_ids)) = self.transaction_info_hash_storage.get(txn_hash) {
            for id in transaction_info_ids {
                if let Ok(Some(transaction_info)) = self.get_transaction_info(id) {
                    transaction_info_vec.push(transaction_info);
                }
            }
        }
        Ok(transaction_info_vec)
    }

    fn get_transaction_info_ids_by_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<HashValue>, Error> {
        self.transaction_info_hash_storage
            .get_transaction_info_ids_by_hash(txn_hash)
    }

    fn save_transaction_infos(&self, vec_txn_info: Vec<TransactionInfo>) -> Result<(), Error> {
        self.transaction_info_hash_storage
            .save_transaction_infos(vec_txn_info.clone())?;
        self.transaction_info_storage
            .save_transaction_infos(vec_txn_info)
    }
}

impl ContractEventStore for Storage {
    fn save_contract_events(
        &self,
        txn_info_id: HashValue,
        events: Vec<ContractEvent>,
    ) -> Result<(), Error> {
        self.event_storage.save_contract_events(txn_info_id, events)
    }

    fn get_contract_events(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        self.event_storage.get(txn_info_id)
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
    + BlockInfoStore
    + TransactionStore
    + TransactionInfoStore
    + ContractEventStore
    + IntoSuper<dyn StateNodeStore>
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

    fn get_accumulator_store(
        &self,
        accumulator_type: AccumulatorStoreType,
    ) -> Arc<dyn AccumulatorTreeStore>;
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

impl Store for Storage {
    fn get_accumulator_store(
        &self,
        accumulator_type: AccumulatorStoreType,
    ) -> Arc<dyn AccumulatorTreeStore> {
        match accumulator_type {
            AccumulatorStoreType::Block => Arc::new(self.block_accumulator_storage.clone()),
            AccumulatorStoreType::Transaction => {
                Arc::new(self.transaction_accumulator_storage.clone())
            }
        }
    }
}
