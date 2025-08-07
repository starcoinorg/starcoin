// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::accumulator::{
    AccumulatorStorage, BlockAccumulatorStorage, TransactionAccumulatorStorage,
    VMStateAccumulatorStorage,
};
use crate::block::BlockStorage;
use crate::block_info::{BlockInfoStore, StcBlockInfoStorage};
use crate::chain_info::ChainInfoStorage;
use crate::contract_event::StcContractEventStorage;
use crate::state_node::StateStorage;
use crate::storage::{CodecKVStore, CodecWriteBatch, ColumnFamilyName, StorageInstance};
use crate::table_info::{StcTableInfoStorage, TableInfoStore};
use crate::transaction::StcTransactionStorage;
use crate::transaction_info::{StcTransactionInfoStorage, TransactionInfoHashStorage};
use anyhow::{bail, ensure, format_err, Error, Result};
use network_p2p_types::peer_id::PeerId;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, AccumulatorTreeStore, MerkleAccumulator};
use starcoin_crypto::HashValue;
use starcoin_state_store_api::{StateNode, StateNodeStore};
//use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::contract_event::StcContractEvent;
use starcoin_types::multi_state::MultiState;
use starcoin_types::startup_info::{ChainInfo, ChainStatus, SnapshotRange};
use starcoin_types::table::{StcTableHandle, StcTableInfo};
use starcoin_types::transaction::{StcRichTransactionInfo, StcTransaction};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo},
    startup_info::StartupInfo,
};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
pub use upgrade::BARNARD_HARD_FORK_HASH;
pub use upgrade::BARNARD_HARD_FORK_HEIGHT;
pub use version::StorageVersion;

pub mod accumulator;
pub mod batch;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod chain_info;
pub mod contract_event;
pub mod db_storage;
pub mod errors;
pub mod metrics;
pub mod state_node;
pub mod storage;
pub mod table_info;
#[cfg(test)]
mod tests;
pub mod transaction;
pub mod transaction_info;
mod upgrade;

#[macro_use]
pub mod storage_macros;
mod version;

pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const BLOCK_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_block";
pub const TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_transaction";
pub const VM_STATE_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_vm_state";
pub const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
pub const BLOCK_PREFIX_NAME_V2: ColumnFamilyName = "block_v2";
pub const BLOCK_HEADER_PREFIX_NAME: ColumnFamilyName = "block_header";
pub const BLOCK_HEADER_PREFIX_NAME_V2: ColumnFamilyName = "block_header_v2";
pub const BLOCK_BODY_PREFIX_NAME: ColumnFamilyName = "block_body";
pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub const BLOCK_INFO_PREFIX_NAME_V2: ColumnFamilyName = "block_info_v2";
pub const BLOCK_TRANSACTIONS_PREFIX_NAME: ColumnFamilyName = "block_txns";
pub const BLOCK_TRANSACTION_INFOS_PREFIX_NAME: ColumnFamilyName = "block_txn_infos";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const STATE_NODE_PREFIX_NAME_PREV: ColumnFamilyName = "state_node_prev";
pub const CHAIN_INFO_PREFIX_NAME: ColumnFamilyName = "chain_info";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_PREFIX_NAME_V2: ColumnFamilyName = "transaction_v2";
pub const TRANSACTION_INFO_PREFIX_NAME: ColumnFamilyName = "transaction_info";
pub const TRANSACTION_INFO_PREFIX_NAME_V2: ColumnFamilyName = "transaction_info_v2";
pub const TRANSACTION_INFO_PREFIX_NAME_V3: ColumnFamilyName = "transaction_info_v3";
pub const TRANSACTION_INFO_HASH_PREFIX_NAME: ColumnFamilyName = "transaction_info_hash";
pub const CONTRACT_EVENT_PREFIX_NAME: ColumnFamilyName = "contract_event";
pub const CONTRACT_EVENT_PREFIX_NAME_V2: ColumnFamilyName = "contract_event_v2";
pub const FAILED_BLOCK_PREFIX_NAME: ColumnFamilyName = "failed_block";
pub const FAILED_BLOCK_PREFIX_NAME_V2: ColumnFamilyName = "failed_block_v2";
pub const TABLE_INFO_PREFIX_NAME: ColumnFamilyName = "table_info";
pub const TABLE_INFO_PREFIX_NAME_V2: ColumnFamilyName = "table_info_v2";
pub const DAG_SYNC_BLOCK_PREFIX_NAME: ColumnFamilyName = "dag_sync_block";

pub trait BlockStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_genesis(&self) -> Result<Option<HashValue>>;

    fn save_genesis(&self, genesis_hash: HashValue) -> Result<()>;

    fn get_chain_info(&self) -> Result<Option<ChainInfo>>;

    fn get_block(&self, block_id: HashValue) -> Result<Option<Block>>;

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;

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
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<StcRichTransactionInfo>>;
    fn get_transaction_info_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>>;
    /// Get transaction info ids by transaction hash, one transaction may be in different chain branch, so produce multiply transaction info.
    /// if not transaction info match with the `txn_hash`, return empty Vec.
    fn get_transaction_info_ids_by_txn_hash(&self, txn_hash: HashValue) -> Result<Vec<HashValue>>;
    fn save_transaction_infos(&self, vec_txn_info: Vec<StcRichTransactionInfo>) -> Result<()>;
    fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<StcRichTransactionInfo>>>;
}
pub trait ContractEventStore {
    /// Save events by key `txn_info_id`.
    /// As txn_info has accumulator root of events, so there is a one-to-one mapping.
    fn save_contract_events_v2(
        &self,
        txn_info_id: HashValue,
        events: Vec<StcContractEvent>,
    ) -> Result<()>;

    /// Get events by `txn_info_id`.
    /// If the txn_info_id does not exists in the store, return `None`.
    /// NOTICE: *don't exists* is different with *no events produced*.
    fn get_contract_events_v2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>>;

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
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>>;
    fn save_transaction(&self, txn_info: StcTransaction) -> Result<()>;
    fn save_transaction_batch(&self, txn_vec: Vec<StcTransaction>) -> Result<()>;
    fn get_transactions(&self, txn_hash_vec: Vec<HashValue>)
        -> Result<Vec<Option<StcTransaction>>>;
}

// TODO: remove Arc<dyn Store>, we can clone Storage directly.
#[derive(Clone)]
pub struct Storage {
    transaction_info_storage: StcTransactionInfoStorage,
    transaction_info_hash_storage: TransactionInfoHashStorage,
    transaction_storage: StcTransactionStorage,
    block_storage: BlockStorage,
    state_node_storage: StateStorage,
    block_accumulator_storage: AccumulatorStorage<BlockAccumulatorStorage>,
    transaction_accumulator_storage: AccumulatorStorage<TransactionAccumulatorStorage>,
    vm_state_accumulator_storage: AccumulatorStorage<VMStateAccumulatorStorage>,
    block_info_storage: StcBlockInfoStorage,
    event_storage: StcContractEventStorage,
    chain_info_storage: ChainInfoStorage,
    table_info_storage: StcTableInfoStorage,
    // instance: StorageInstance,
}

impl Storage {
    pub fn new(instance: StorageInstance) -> Result<Self> {
        let storage = Self {
            transaction_info_storage: StcTransactionInfoStorage::new(instance.clone()),
            transaction_info_hash_storage: TransactionInfoHashStorage::new(instance.clone()),
            transaction_storage: StcTransactionStorage::new(instance.clone()),
            block_storage: BlockStorage::new(instance.clone()),
            state_node_storage: StateStorage::new(instance.clone()),
            block_accumulator_storage: AccumulatorStorage::new_block_accumulator_storage(
                instance.clone(),
            ),
            transaction_accumulator_storage:
                AccumulatorStorage::new_transaction_accumulator_storage(instance.clone()),
            vm_state_accumulator_storage: AccumulatorStorage::new_vm_state_accumulator_storage(
                instance.clone(),
            ),
            block_info_storage: StcBlockInfoStorage::new(instance.clone()),
            event_storage: StcContractEventStorage::new(instance.clone()),
            chain_info_storage: ChainInfoStorage::new(instance.clone()),
            table_info_storage: StcTableInfoStorage::new(instance),
            // instance,
        };
        Ok(storage)
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

    fn get_table_info(&self, address: AccountAddress) -> Result<Option<TableInfo>> {
        let handle = TableHandle(address).into();
        Ok(self.table_info_storage.get(handle)?.and_then(|i| i.to_v1()))
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
    fn get_transaction_info(&self, id: HashValue) -> Result<Option<StcRichTransactionInfo>> {
        self.transaction_info_storage.get_transaction_info(id)
    }

    fn get_transaction_info_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>, Error> {
        let mut transaction_info_vec = vec![];
        if let Some(transaction_info_ids) = self.transaction_info_hash_storage.get(txn_hash)? {
            let txn_infos = self.get_transaction_infos(transaction_info_ids)?;
            for transaction_info in txn_infos.into_iter().flatten() {
                transaction_info_vec.push(transaction_info);
            }
        }
        Ok(transaction_info_vec)
    }

    fn get_transaction_info_ids_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<HashValue>, Error> {
        self.transaction_info_hash_storage
            .get_transaction_info_ids_by_hash(txn_hash)
    }

    fn save_transaction_infos(
        &self,
        vec_txn_info: Vec<StcRichTransactionInfo>,
    ) -> Result<(), Error> {
        self.transaction_info_hash_storage
            .save_transaction_infos(vec_txn_info.as_slice())?;
        self.transaction_info_storage
            .save_transaction_infos(vec_txn_info)
    }

    fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<StcRichTransactionInfo>>> {
        self.transaction_info_storage.get_transaction_infos(ids)
    }
}

impl ContractEventStore for Storage {
    fn save_contract_events_v2(
        &self,
        txn_info_id: HashValue,
        events: Vec<StcContractEvent>,
    ) -> Result<()> {
        self.event_storage
            .save_contract_events_v2(txn_info_id, events)
    }

    fn get_contract_events_v2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>> {
        self.event_storage.get_contract_events_v2(txn_info_id)
    }

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
        self.event_storage.get_contract_events(txn_info_id)
    }
}

impl TransactionStore for Storage {
    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>, Error> {
        self.transaction_storage.get(txn_hash)
    }

    fn save_transaction(&self, txn: StcTransaction) -> Result<(), Error> {
        self.transaction_storage.put(txn.id(), txn)
    }

    fn save_transaction_batch(&self, txn_vec: Vec<StcTransaction>) -> Result<(), Error> {
        self.transaction_storage.save_transaction_batch(txn_vec)
    }

    fn get_transactions(
        &self,
        txn_hash_vec: Vec<HashValue>,
    ) -> Result<Vec<Option<StcTransaction>>, Error> {
        self.transaction_storage.multiple_get(txn_hash_vec)
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
    fn get_transaction_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<StcRichTransactionInfo>> {
        let txn_infos = self.get_block_txn_info_ids(block_id)?;
        match txn_infos.get(idx as usize) {
            None => Ok(None),
            Some(info_hash) => self.get_transaction_info(*info_hash),
        }
    }

    fn get_block_transaction_infos(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>, Error> {
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

    fn get_vm_multi_state(&self, block_id: HashValue) -> Result<MultiState> {
        let block_info = self
            .get_block_info(block_id)?
            .ok_or_else(|| format_err!("Can not find block info {}", block_id))?;
        let acc_info = block_info.vm_state_accumulator_info;
        let num_leaves = acc_info.num_leaves;
        ensure!(
            acc_info.num_leaves > 1,
            "vm state accumulator should have at least 2 leaves"
        );
        let acc = MerkleAccumulator::new_with_info(
            acc_info,
            self.get_accumulator_store(AccumulatorStoreType::VMState),
        );
        Ok(MultiState::new(
            acc.get_leaf(num_leaves - 2)?.ok_or_else(|| {
                format_err!("failed to get leaf for state1 at {}", num_leaves - 2)
            })?,
            acc.get_leaf(num_leaves - 1)?.ok_or_else(|| {
                format_err!("failed to get leaf for state2 at {}", num_leaves - 1)
            })?,
        ))
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
            AccumulatorStoreType::VMState => Arc::new(self.vm_state_accumulator_storage.clone()),
        }
    }
}

impl TableInfoStore for Storage {
    fn get_table_info(&self, key: StcTableHandle) -> Result<Option<StcTableInfo>> {
        self.table_info_storage.get(key)
    }

    fn save_table_info(&self, key: StcTableHandle, table_info: StcTableInfo) -> Result<()> {
        self.table_info_storage.put(key, table_info)
    }

    fn get_table_infos(&self, keys: Vec<StcTableHandle>) -> Result<Vec<Option<StcTableInfo>>> {
        self.table_info_storage.multiple_get(keys)
    }

    fn save_table_infos(&self, table_infos: Vec<(StcTableHandle, StcTableInfo)>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(table_infos);
        self.table_info_storage.write_batch(batch)
    }
}
