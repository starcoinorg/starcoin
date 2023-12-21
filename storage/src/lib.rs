// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    accumulator::{AccumulatorStorage, BlockAccumulatorStorage, TransactionAccumulatorStorage},
    block::BlockStorage,
    block_info::{BlockInfoStorage, BlockInfoStore},
    chain_info::ChainInfoStorage,
    contract_event::ContractEventStorage,
    state_node::StateStorage,
    storage::{CodecKVStore, CodecWriteBatch, ColumnFamilyName, StorageInstance},
};
use crate::{
    transaction::TransactionStorage,
    transaction_info::{TransactionInfoHashStorage, TransactionInfoStorage},
};
use anyhow::{bail, format_err, Error, Ok, Result};
use flexi_dag::{DagTips, SyncFlexiDagSnapshot, SyncFlexiDagSnapshotStorage, SyncFlexiDagStorage};
use network_p2p_types::peer_id::PeerId;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator,
    AccumulatorTreeStore, MerkleAccumulator,
};
use starcoin_crypto::HashValue;
use starcoin_state_store_api::{StateNode, StateNodeStore};
use starcoin_types::startup_info::DagStartupInfo;
use starcoin_types::{
    block::{Block, BlockBody, BlockHeader, BlockInfo},
    contract_event::ContractEvent,
    dag_block::KTotalDifficulty,
    startup_info::{ChainInfo, ChainStatus, SnapshotRange, StartupInfo},
    transaction::{RichTransactionInfo, Transaction},
};
use starcoin_vm_types::{
    account_address::AccountAddress,
    state_store::table::{TableHandle, TableInfo},
};
use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};
use table_info::{TableInfoStorage, TableInfoStore};
pub use upgrade::{BARNARD_HARD_FORK_HASH, BARNARD_HARD_FORK_HEIGHT};

pub mod accumulator;
pub mod batch;
pub mod block;
pub mod block_info;
pub mod cache_storage;
pub mod chain_info;
pub mod contract_event;
pub mod db_storage;
pub mod errors;
pub mod flexi_dag;
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

pub const DEFAULT_PREFIX_NAME: ColumnFamilyName = "default";
pub const BLOCK_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_block";
pub const TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME: ColumnFamilyName = "acc_node_transaction";
pub const BLOCK_PREFIX_NAME: ColumnFamilyName = "block";
pub const BLOCK_HEADER_PREFIX_NAME: ColumnFamilyName = "block_header";
pub const BLOCK_PREFIX_NAME_V2: ColumnFamilyName = "block_v2";
pub const BLOCK_HEADER_PREFIX_NAME_V2: ColumnFamilyName = "block_header_v2";
pub const BLOCK_BODY_PREFIX_NAME: ColumnFamilyName = "block_body";
pub const BLOCK_INFO_PREFIX_NAME: ColumnFamilyName = "block_info";
pub const BLOCK_TRANSACTIONS_PREFIX_NAME: ColumnFamilyName = "block_txns";
pub const BLOCK_TRANSACTION_INFOS_PREFIX_NAME: ColumnFamilyName = "block_txn_infos";
pub const STATE_NODE_PREFIX_NAME: ColumnFamilyName = "state_node";
pub const STATE_NODE_PREFIX_NAME_PREV: ColumnFamilyName = "state_node_prev";
pub const CHAIN_INFO_PREFIX_NAME: ColumnFamilyName = "chain_info";
pub const TRANSACTION_PREFIX_NAME: ColumnFamilyName = "transaction";
pub const TRANSACTION_INFO_PREFIX_NAME: ColumnFamilyName = "transaction_info";
pub const TRANSACTION_INFO_PREFIX_NAME_V2: ColumnFamilyName = "transaction_info_v2";
pub const TRANSACTION_INFO_HASH_PREFIX_NAME: ColumnFamilyName = "transaction_info_hash";
pub const CONTRACT_EVENT_PREFIX_NAME: ColumnFamilyName = "contract_event";
pub const FAILED_BLOCK_PREFIX_NAME: ColumnFamilyName = "failed_block";
pub const FAILED_BLOCK_PREFIX_NAME_V2: ColumnFamilyName = "failed_block_v2";
pub const TABLE_INFO_PREFIX_NAME: ColumnFamilyName = "table_info";
pub const SYNC_FLEXI_DAG_ACCUMULATOR_PREFIX_NAME: ColumnFamilyName = "sync_flexi_dag_accumulator";
pub const SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME: ColumnFamilyName = "sync_flexi_dag_snapshot";
pub const DAG_TIPS_PREFIX_NAME: ColumnFamilyName = "dag_tips";

///db storage use prefix_name vec to init
/// Please note that adding a prefix needs to be added in vec simultaneously, remember！！
static VEC_PREFIX_NAME_V1: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V2: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME,
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME_V2,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V3: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_BODY_PREFIX_NAME, // unused column
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME, // unused column
        TRANSACTION_INFO_PREFIX_NAME_V2,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
        TABLE_INFO_PREFIX_NAME,
    ]
});

static VEC_PREFIX_NAME_V4: Lazy<Vec<ColumnFamilyName>> = Lazy::new(|| {
    vec![
        BLOCK_ACCUMULATOR_NODE_PREFIX_NAME,
        TRANSACTION_ACCUMULATOR_NODE_PREFIX_NAME,
        BLOCK_PREFIX_NAME,
        BLOCK_HEADER_PREFIX_NAME,
        BLOCK_PREFIX_NAME_V2,
        BLOCK_HEADER_PREFIX_NAME_V2,
        BLOCK_BODY_PREFIX_NAME, // unused column
        BLOCK_INFO_PREFIX_NAME,
        BLOCK_TRANSACTIONS_PREFIX_NAME,
        BLOCK_TRANSACTION_INFOS_PREFIX_NAME,
        STATE_NODE_PREFIX_NAME,
        CHAIN_INFO_PREFIX_NAME,
        TRANSACTION_PREFIX_NAME,
        TRANSACTION_INFO_PREFIX_NAME, // unused column
        TRANSACTION_INFO_PREFIX_NAME_V2,
        TRANSACTION_INFO_HASH_PREFIX_NAME,
        CONTRACT_EVENT_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME,
        FAILED_BLOCK_PREFIX_NAME_V2,
        SYNC_FLEXI_DAG_ACCUMULATOR_PREFIX_NAME,
        SYNC_FLEXI_DAG_SNAPSHOT_PREFIX_NAME,
        DAG_TIPS_PREFIX_NAME,
        TABLE_INFO_PREFIX_NAME,
    ]
});

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum StorageVersion {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
}

impl StorageVersion {
    pub fn current_version() -> StorageVersion {
        StorageVersion::V4
    }

    pub fn get_column_family_names(&self) -> &'static [ColumnFamilyName] {
        match self {
            StorageVersion::V1 => &VEC_PREFIX_NAME_V1,
            StorageVersion::V2 => &VEC_PREFIX_NAME_V2,
            StorageVersion::V3 => &VEC_PREFIX_NAME_V3,
            StorageVersion::V4 => &VEC_PREFIX_NAME_V4,
        }
    }
}

// pub trait DagBlockStore {
//     fn get_flexi_dag_startup_info(&self) -> Result<Option<StartupInfo>>;
//     fn save_flexi_dag_startup_info(&self, startup_info: StartupInfo) -> Result<()>;
//     fn get_dag_accumulator_info(&self) -> Result<AccumulatorInfo>;
//     fn get_last_tips(&self) -> Result<Option<Vec<HashValue>>>;
// }

pub trait BlockStore {
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;
    fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()>;

    fn get_dag_startup_info(&self) -> Result<Option<DagStartupInfo>>;
    fn save_dag_startup_info(&self, dag_startup_info: DagStartupInfo) -> Result<()>;

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
        txn_hash: HashValue,
    ) -> Result<Vec<RichTransactionInfo>>;
    /// Get transaction info ids by transaction hash, one transaction may be in different chain branch, so produce multiply transaction info.
    /// if not transaction info match with the `txn_hash`, return empty Vec.
    fn get_transaction_info_ids_by_txn_hash(&self, txn_hash: HashValue) -> Result<Vec<HashValue>>;
    fn save_transaction_infos(&self, vec_txn_info: Vec<RichTransactionInfo>) -> Result<()>;
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
    fn save_transaction_batch(&self, txn_vec: Vec<Transaction>) -> Result<()>;
    fn get_transactions(&self, txn_hash_vec: Vec<HashValue>) -> Result<Vec<Option<Transaction>>>;
}

pub trait SyncFlexiDagStore {
    fn put_hashes(&self, key: HashValue, accumulator_snapshot: SyncFlexiDagSnapshot) -> Result<()>;
    fn query_by_hash(&self, key: HashValue) -> Result<Option<SyncFlexiDagSnapshot>>;
    fn get_accumulator_snapshot_storage(&self) -> Arc<SyncFlexiDagSnapshotStorage>;
    fn get_dag_accumulator_info(&self) -> Result<Option<AccumulatorInfo>>;
    fn get_tips_by_block_id(&self, block_id: HashValue) -> Result<Vec<HashValue>>;
    fn get_latest_snapshot(&self) -> Result<Option<SyncFlexiDagSnapshot>>;
    fn save_dag_tips(&self, tips: Vec<HashValue>) -> Result<()>;
    fn get_dag_tips(&self) -> Result<Option<DagTips>>;
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
    flexi_dag_storage: SyncFlexiDagStorage,
    table_info_storage: TableInfoStorage,
    // instance: StorageInstance,
}

impl Storage {
    pub fn new(instance: StorageInstance) -> Result<Self> {
        let storage = Self {
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
            chain_info_storage: ChainInfoStorage::new(instance.clone()),
            flexi_dag_storage: SyncFlexiDagStorage::new(instance.clone()),
            table_info_storage: TableInfoStorage::new(instance),
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

    fn get_dag_startup_info(&self) -> Result<Option<DagStartupInfo>> {
        self.chain_info_storage.get_flexi_dag_startup_info()
    }

    fn save_dag_startup_info(&self, dag_startup_info: DagStartupInfo) -> Result<()> {
        self.chain_info_storage
            .save_flexi_dag_startup_info(dag_startup_info)
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
        let tips = self.get_dag_tips()?;

        let block_accumulator = MerkleAccumulator::new_with_info(
            head_block_info.block_accumulator_info.clone(),
            self.get_accumulator_store(AccumulatorStoreType::Block),
        );
        // todo: k should be read from node config
        let k_key = block_accumulator
            .get_leaf(std::cmp::max(head_block.number() - 8, 0))?
            .ok_or_else(|| format_err!("failed to find the k-smallest total difficulty"))?;
        let k_block_info = self
            .get_block_info(k_key)?
            .ok_or_else(|| format_err!("kth block info {:?} should exist", k_key))?;
        let flexidag_dag_accumulator_info = Some(head_block_info.dag_accumulator_info.clone());
        let chain_info = ChainInfo::new(
            head_block.chain_id(),
            genesis_hash,
            ChainStatus::new(head_block, head_block_info, tips.map(|t| t.tips)),
            flexidag_dag_accumulator_info,
            KTotalDifficulty {
                total_difficulty: k_block_info.total_difficulty,
            },
        );
        Ok(Some(chain_info))
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
        self.transaction_info_storage.get_transaction_info(id)
    }

    fn get_transaction_info_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<RichTransactionInfo>, Error> {
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

    fn save_transaction_infos(&self, vec_txn_info: Vec<RichTransactionInfo>) -> Result<(), Error> {
        self.transaction_info_hash_storage
            .save_transaction_infos(vec_txn_info.as_slice())?;
        self.transaction_info_storage
            .save_transaction_infos(vec_txn_info)
    }

    fn get_transaction_infos(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<RichTransactionInfo>>> {
        self.transaction_info_storage.get_transaction_infos(ids)
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

    fn get_transactions(
        &self,
        txn_hash_vec: Vec<HashValue>,
    ) -> Result<Vec<Option<Transaction>>, Error> {
        self.transaction_storage.multiple_get(txn_hash_vec)
    }
}

impl SyncFlexiDagStore for Storage {
    fn put_hashes(&self, key: HashValue, accumulator_snapshot: SyncFlexiDagSnapshot) -> Result<()> {
        self.flexi_dag_storage.put_hashes(key, accumulator_snapshot)
    }

    fn query_by_hash(&self, key: HashValue) -> Result<Option<SyncFlexiDagSnapshot>> {
        self.flexi_dag_storage.get_hashes_by_hash(key)
    }

    fn get_accumulator_snapshot_storage(&self) -> Arc<SyncFlexiDagSnapshotStorage> {
        self.flexi_dag_storage.get_snapshot_storage()
    }

    fn get_dag_accumulator_info(&self) -> Result<Option<AccumulatorInfo>> {
        Ok(
            if let Some(dag_main) = self.get_dag_startup_info()?.map(|i| *i.get_dag_main()) {
                Some(
                    self.query_by_hash(dag_main)?
                        .expect("snapshot should not be none")
                        .accumulator_info,
                )
            } else {
                None
            },
        )
    }

    fn get_tips_by_block_id(&self, key: HashValue) -> Result<Vec<HashValue>> {
        match self.query_by_hash(key)? {
            Some(snapshot) => Ok(snapshot.child_hashes),
            None => {
                bail!("failed to get snapshot by hash: {}", key);
            }
        }
    }

    fn get_latest_snapshot(&self) -> Result<Option<SyncFlexiDagSnapshot>> {
        Ok(
            if let Some(dag_main) = self
                .get_dag_startup_info()?
                .map(|info| *info.get_dag_main())
            {
                self.query_by_hash(dag_main)?
            } else {
                None
            },
        )
    }

    fn save_dag_tips(&self, tips: Vec<HashValue>) -> Result<()> {
        self.flexi_dag_storage.save_dag_tips(tips)
    }

    fn get_dag_tips(&self) -> Result<Option<DagTips>> {
        self.flexi_dag_storage.get_dag_tips()
    }
}

/// Chain storage define
pub trait Store:
    StateNodeStore
    + SyncFlexiDagStore
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
    fn get_accumulator_store(
        &self,
        accumulator_type: AccumulatorStoreType,
    ) -> Arc<dyn AccumulatorTreeStore> {
        match accumulator_type {
            AccumulatorStoreType::Block => Arc::new(self.block_accumulator_storage.clone()),
            AccumulatorStoreType::Transaction => {
                Arc::new(self.transaction_accumulator_storage.clone())
            }
            AccumulatorStoreType::SyncDag => {
                Arc::new(self.flexi_dag_storage.get_accumulator_storage())
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
