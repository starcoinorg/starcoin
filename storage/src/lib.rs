// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::schema::{
    state_node::State as StateSchema, transaction::Transaction as TxnSchema,
    transaction_info::TransactionInfo as TxnInfoSchema,
    transaction_info::TransactionInfoHash as TxnInfoHashSchema,
};
use crate::transaction_store::{
    TransactionInfoHashStorage, TransactionInfoStorage, TransactionStorage,
};
use crate::{
    accumulator::{BlockAccumulatorStorage, TransactionAccumulatorStorage},
    block::BlockStorage,
    block_info::{BlockInfoStorage, BlockInfoStore},
    chain_info::ChainInfoStorage,
    contract_event::ContractEventStorage,
    state_node::StateStorage,
    storage::{CodecKVStore, CodecWriteBatch, StorageInstance},
    table_info::{TableInfoStorage, TableInfoStore},
};
use anyhow::{bail, format_err, Error, Result};
use network_p2p_types::peer_id::PeerId;
use starcoin_accumulator::{node::AccumulatorStoreType, AccumulatorTreeStore};
use starcoin_crypto::HashValue;
pub use starcoin_schemadb::db::{
    StorageVersion, BLOCK_ACCUMULATOR_NODE_PREFIX_NAME, BLOCK_BODY_PREFIX_NAME,
    BLOCK_HEADER_PREFIX_NAME, BLOCK_INFO_PREFIX_NAME, BLOCK_PREFIX_NAME,
    BLOCK_TRANSACTIONS_PREFIX_NAME, BLOCK_TRANSACTION_INFOS_PREFIX_NAME, CHAIN_INFO_PREFIX_NAME,
    CONTRACT_EVENT_PREFIX_NAME, DEFAULT_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME,
    STATE_NODE_PREFIX_NAME, STATE_NODE_PREFIX_NAME_PREV, TABLE_INFO_PREFIX_NAME,
    TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME, TRANSACTION_INFO_HASH_PREFIX_NAME,
    TRANSACTION_INFO_PREFIX_NAME, TRANSACTION_INFO_PREFIX_NAME_V2, TRANSACTION_PREFIX_NAME,
};
use starcoin_schemadb::{db::DBStorage as DB, SchemaBatch};
use starcoin_state_store_api::{StateNode, StateNodeStore};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockBody, BlockHeader, BlockInfo},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, ChainStatus, SnapshotRange, StartupInfo},
    transaction::{RichTransactionInfo, Transaction},
};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};
pub use upgrade::{BARNARD_HARD_FORK_HASH, BARNARD_HARD_FORK_HEIGHT};

pub mod accumulator;
pub mod batch;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod chain_info;
pub mod contract_event;
pub mod db_storage;
pub mod errors {
    pub use starcoin_schemadb::error::StorageInitError;
}
pub mod metrics {
    pub use starcoin_schemadb::metrics::*;
}
pub mod state_node;
pub mod storage;
pub mod table_info;
#[cfg(test)]
mod tests;
mod upgrade;

#[macro_use]
pub mod storage_macros;
pub mod schema;
mod transaction_store;

pub trait BlockStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_genesis(&self) -> Result<Option<HashValue>>;

    fn save_genesis(&self, genesis_hash: HashValue) -> Result<()>;

    fn get_chain_info(&self) -> Result<Option<ChainInfo>>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;

    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>>;

    fn commit_block(&self, block: Block) -> Result<()>;

    /// delete_block will delete block data, txns and txn infos.
    fn delete_block(&self, block_id: HashValue) -> Result<()>;

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>>;

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn save_block_transaction_ids(
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

    fn save_failed_block(
        &self,
        block_id: HashValue,
        block: Block,
        peer_id: Option<PeerId>,
        failed: String,
        version: String,
    ) -> Result<()>;

    fn delete_failed_block(&self, block_id: HashValue) -> Result<()>;

    fn get_failed_block_by_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<(Block, Option<PeerId>, String, String)>>;

    fn get_snapshot_range(&self) -> Result<Option<SnapshotRange>>;
    fn save_snapshot_range(&self, snapshot_height: SnapshotRange) -> Result<()>;
}

pub trait BlockTransactionInfoStore {
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<RichTransactionInfo>>;
    fn get_transaction_info_by_txn_hash(
        &self,
        txn_hash: &HashValue,
    ) -> Result<Vec<RichTransactionInfo>>;
    /// Get transaction info ids by transaction hash, one transaction may be in different chain branch, so produce multiply transaction info.
    /// if not transaction info match with the `txn_hash`, return empty Vec.
    fn get_transaction_info_ids_by_txn_hash(
        &self,
        txn_hash: &HashValue,
    ) -> Result<Option<Vec<HashValue>>>;
    fn merge_transaction_info_ids(
        &self,
        txn_infos: &[(HashValue, HashValue)],
    ) -> Result<BTreeMap<HashValue, Vec<HashValue>>>;
    fn save_transaction_infos_batch(
        &self,
        vec_txn_info: &[RichTransactionInfo],
        txn_info_ids: &[(HashValue, Vec<HashValue>)],
        batch: &SchemaBatch,
    ) -> Result<(), Error>;
    fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<RichTransactionInfo>>>;
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
    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>, batch: &SchemaBatch) -> Result<()>;
    fn get_transactions(&self, txn_hash_vec: Vec<HashValue>) -> Result<Vec<Option<Transaction>>>;
}

// TODO: remove Arc<dyn Store>, we can clone Storage directly.
#[derive(Clone)]
pub struct Storage {
    ledger_db: Arc<DB>,
    transaction_info_storage: TransactionInfoStorage,
    transaction_info_hash_storage: TransactionInfoHashStorage,
    transaction_storage: TransactionStorage,
    block_storage: BlockStorage,
    state_node_storage: StateStorage,
    block_accumulator_storage: BlockAccumulatorStorage,
    transaction_accumulator_storage: TransactionAccumulatorStorage,
    block_info_storage: BlockInfoStorage,
    event_storage: ContractEventStorage,
    chain_info_storage: ChainInfoStorage,
    table_info_storage: TableInfoStorage,
    // instance: StorageInstance,
}

impl Storage {
    pub fn new(instance: StorageInstance) -> Result<Self> {
        let ledger_db = instance.db().unwrap();
        let storage = Self {
            ledger_db: Arc::clone(ledger_db),
            transaction_info_storage: TransactionInfoStorage::new(ledger_db),
            transaction_info_hash_storage: TransactionInfoHashStorage::new(ledger_db),
            transaction_storage: TransactionStorage::new(ledger_db),
            block_storage: BlockStorage::new(instance.clone()),
            state_node_storage: StateStorage::new(ledger_db),
            block_accumulator_storage: BlockAccumulatorStorage::new_accumulator_storage(
                instance.db().unwrap(),
            ),
            transaction_accumulator_storage: TransactionAccumulatorStorage::new_accumulator_storage(
                instance.db().unwrap(),
            ),
            block_info_storage: BlockInfoStorage::new(instance.clone()),
            event_storage: ContractEventStorage::new(instance.clone()),
            chain_info_storage: ChainInfoStorage::new(instance.clone()),
            table_info_storage: TableInfoStorage::new(instance),
            // instance,
        };
        Ok(storage)
    }

    pub fn get_block_accumulator_storage(&self) -> Arc<dyn AccumulatorTreeStore> {
        Arc::new(self.block_accumulator_storage.clone())
    }

    pub fn get_transaction_accumulator_storage(&self) -> Arc<dyn AccumulatorTreeStore> {
        Arc::new(self.transaction_accumulator_storage.clone())
    }

    pub fn ledger_db(&self) -> &DB {
        self.ledger_db.as_ref()
    }
}

impl StateNodeStore for Storage {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        self.state_node_storage.get(hash)
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.state_node_storage.put(&key, &node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        let batch = SchemaBatch::new();
        for (key, value) in nodes.iter() {
            batch.put::<StateSchema>(key, value)?;
        }
        self.state_node_storage.write_schemas(batch)
    }

    fn get_table_info(&self, address: AccountAddress) -> Result<Option<TableInfo>> {
        let handle = TableHandle(address);
        self.table_info_storage.get(handle)
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

    fn get_genesis(&self) -> Result<Option<HashValue>> {
        self.chain_info_storage.get_genesis()
    }

    fn save_genesis(&self, genesis_hash: HashValue) -> Result<()> {
        self.chain_info_storage.save_genesis(genesis_hash)
    }

    fn get_chain_info(&self) -> Result<Option<ChainInfo>> {
        let genesis_hash = match self.get_genesis()? {
            Some(genesis_hash) => genesis_hash,
            None => return Ok(None),
        };
        let startup_info = match self.get_startup_info()? {
            Some(startup_info) => startup_info,
            None => return Ok(None),
        };
        let head_block = self
            .get_block_header_by_hash(startup_info.main)?
            .ok_or_else(|| format_err!("Startup block {:?} should exist", startup_info.main))?;
        let head_block_info = self.get_block_info(head_block.id())?.ok_or_else(|| {
            format_err!("Startup block info {:?} should exist", startup_info.main)
        })?;
        Ok(Some(ChainInfo::new(
            head_block.chain_id(),
            genesis_hash,
            ChainStatus::new(head_block, head_block_info),
        )))
    }

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_storage.get(block_id)
    }

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        self.block_storage.get_blocks(ids)
    }

    #[allow(deprecated)]
    fn get_body(&self, block_id: HashValue) -> Result<Option<BlockBody>> {
        self.block_storage.get_body(block_id)
    }

    fn commit_block(&self, block: Block) -> Result<()> {
        self.block_storage.commit_block(block)
    }

    fn delete_block(&self, block_id: HashValue) -> Result<()> {
        self.block_storage.delete_block(block_id)
    }

    fn get_block_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.block_storage.get_block_header_by_hash(block_id)
    }

    fn get_block_by_hash(&self, block_id: HashValue) -> Result<Option<Block>> {
        self.block_storage.get_block_by_hash(block_id)
    }

    fn save_block_transaction_ids(
        &self,
        block_id: HashValue,
        transactions: Vec<HashValue>,
    ) -> Result<()> {
        self.block_storage
            .put_transaction_ids(block_id, transactions)
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

    fn save_failed_block(
        &self,
        block_id: HashValue,
        block: Block,
        peer_id: Option<PeerId>,
        failed: String,
        version: String,
    ) -> Result<()> {
        self.block_storage
            .save_failed_block(block_id, block, peer_id, failed, version)
    }

    fn delete_failed_block(&self, block_id: HashValue) -> Result<()> {
        self.block_storage.delete_failed_block(block_id)
    }

    fn get_failed_block_by_id(
        &self,
        block_id: HashValue,
    ) -> Result<Option<(Block, Option<PeerId>, String, String)>> {
        self.block_storage.get_failed_block_by_id(block_id)
    }

    fn get_snapshot_range(&self) -> Result<Option<SnapshotRange>> {
        self.chain_info_storage.get_snapshot_range()
    }

    fn save_snapshot_range(&self, snapshot_range: SnapshotRange) -> Result<()> {
        self.chain_info_storage.save_snapshot_range(snapshot_range)
    }
}

impl BlockInfoStore for Storage {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<(), Error> {
        self.block_info_storage.put(block_info.block_id, block_info)
    }

    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>, Error> {
        self.block_info_storage.get(hash_value)
    }
    fn delete_block_info(&self, block_hash: HashValue) -> Result<(), Error> {
        self.block_info_storage.remove(block_hash)
    }

    fn get_block_infos(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        Ok(self
            .block_info_storage
            .multiple_get(ids)?
            .into_iter()
            .collect())
    }
}

impl BlockTransactionInfoStore for Storage {
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<RichTransactionInfo>> {
        self.transaction_info_storage.get(&id)
    }

    fn get_transaction_info_by_txn_hash(
        &self,
        txn_hash: &HashValue,
    ) -> Result<Vec<RichTransactionInfo>, Error> {
        let mut transaction_info_vec = vec![];
        if let Some(transaction_info_ids) = self.get_transaction_info_ids_by_txn_hash(txn_hash)? {
            let txn_infos = self.get_transaction_infos(transaction_info_ids)?;
            for transaction_info in txn_infos.into_iter().flatten() {
                transaction_info_vec.push(transaction_info);
            }
        }
        Ok(transaction_info_vec)
    }

    fn get_transaction_info_ids_by_txn_hash(
        &self,
        txn_hash: &HashValue,
    ) -> Result<Option<Vec<HashValue>>, Error> {
        self.transaction_info_hash_storage.get(txn_hash)
    }

    fn merge_transaction_info_ids(
        &self,
        txn_infos: &[(HashValue, HashValue)],
    ) -> Result<BTreeMap<HashValue, Vec<HashValue>>> {
        let mut local = BTreeMap::new();
        // todo: change to multi_get
        for (hash, id) in txn_infos {
            let id_vec = local.entry(*hash).or_insert(
                self.get_transaction_info_ids_by_txn_hash(hash)?
                    .unwrap_or_else(|| vec![*id]),
            );
            if !id_vec.contains(id) {
                id_vec.push(*id);
            }
        }
        Ok(local)
    }

    fn save_transaction_infos_batch(
        &self,
        vec_txn_info: &[RichTransactionInfo],
        txn_info_ids: &[(HashValue, Vec<HashValue>)],
        batch: &SchemaBatch,
    ) -> Result<(), Error> {
        for (key, val) in txn_info_ids.iter() {
            batch.put::<TxnInfoHashSchema>(key, val)?;
        }
        for info in vec_txn_info.iter() {
            batch.put::<TxnInfoSchema>(&info.transaction_hash(), info)?;
        }

        Ok(())
    }

    fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<RichTransactionInfo>>> {
        let mut cached = self.transaction_info_storage.multi_get(ids.as_ref())?;
        for (info, id) in cached.iter_mut().zip(&ids) {
            if info.is_none() {
                let mut db_item = self.ledger_db.get::<TxnInfoSchema>(id)?;
                std::mem::swap(info, &mut db_item);
            }
        }
        Ok(cached)
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
        self.transaction_storage.get(&txn_hash)
    }

    fn save_transaction(&self, txn: Transaction) -> Result<(), Error> {
        let id = txn.id();
        self.transaction_storage.put(&id, &txn)
    }

    fn save_transaction_batch(
        &self,
        txn_vec: Vec<Transaction>,
        batch: &SchemaBatch,
    ) -> Result<(), Error> {
        for txn in txn_vec {
            let id = txn.id();
            batch.put::<TxnSchema>(&id, &txn)?;
        }
        Ok(())
    }

    fn get_transactions(
        &self,
        txn_hash_vec: Vec<HashValue>,
    ) -> Result<Vec<Option<Transaction>>, Error> {
        self.transaction_storage.multi_get(txn_hash_vec.as_ref())
    }
}

/// Chain storage define
pub trait Store:
    StateNodeStore
    + BlockStore
    + BlockInfoStore
    + TransactionStore
    + BlockTransactionInfoStore
    + ContractEventStore
    + IntoSuper<dyn StateNodeStore>
    + TableInfoStore
{
    fn db(&self) -> Arc<DB>;

    fn get_transaction_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>> {
        let txn_infos = self.get_block_txn_info_ids(block_id)?;
        match txn_infos.get(idx as usize) {
            None => Ok(None),
            Some(info_hash) => self.get_transaction_info(*info_hash),
        }
    }

    fn get_block_transaction_infos(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<RichTransactionInfo>, Error> {
        let txn_info_ids = self.get_block_txn_info_ids(block_id)?;
        let mut txn_infos = vec![];
        let txn_opt_infos = self.get_transaction_infos(txn_info_ids.clone())?;

        for (i, info) in txn_opt_infos.into_iter().enumerate() {
            match info {
                Some(info) => txn_infos.push(info),
                None => bail!(
                    "invalid state: txn info {:?} of block {} should exist",
                    txn_info_ids.get(i),
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
    fn db(&self) -> Arc<DB> {
        self.ledger_db.clone()
    }

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

impl TableInfoStore for Storage {
    fn get_table_info(&self, key: TableHandle) -> Result<Option<TableInfo>> {
        self.table_info_storage.get(key)
    }

    fn save_table_info(&self, key: TableHandle, table_info: TableInfo) -> Result<()> {
        self.table_info_storage.put(key, table_info)
    }

    fn get_table_infos(&self, keys: Vec<TableHandle>) -> Result<Vec<Option<TableInfo>>> {
        self.table_info_storage.multiple_get(keys)
    }

    fn save_table_infos(&self, table_infos: Vec<(TableHandle, TableInfo)>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(table_infos);
        self.table_info_storage.write_batch(batch)
    }
}
