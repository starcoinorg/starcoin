// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod vm2;

use anyhow::{bail, format_err, Result};
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_chain_api::ExcludedTxns;
use starcoin_crypto::HashValue;
use starcoin_executor::BlockExecutedData as BlockExecutedData1;
use starcoin_executor::{execute_block_transactions, execute_transactions, VMMetrics};
use starcoin_logger::prelude::*;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{Store, Store2};
use starcoin_types::block::Version;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::write_set::WriteSet;
use starcoin_types::{
    block::BlockNumber,
    block::{BlockBody, BlockHeader, BlockInfo, BlockTemplate},
    block_metadata::{self, BlockMetadataLegacy},
    error::BlockExecutorError,
    genesis_config::ChainId,
    transaction::{
        SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput, TransactionStatus,
    },
    vm_error::KeptVMStatus,
    U256,
};
use starcoin_vm2_executor::block_executor::BlockExecutedData as BlockExecutedData2;
use starcoin_vm2_state_api::ChainStateReader as ChainStateReader2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::block_metadata::BlockMetadata;
use starcoin_vm2_types::contract_event::ContractEvent as ContractEvent2;
use starcoin_vm2_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::collections::BTreeMap;
use std::{convert::TryInto, sync::Arc};

/// Result of OpenedBlock::finalize(), containing all data needed for caching.
/// This includes the StateDB (with write_sets applied) and BlockExecutedData,
/// which allows skipping re-execution entirely when the mined block is received.
pub struct FinalizedBlock {
    pub template: BlockTemplate,
    pub statedb: Arc<ChainStateDB>,
    pub statedb2: Arc<ChainStateDB2>,
    /// Executed data for VM1 (txn_infos, events, table_infos, write_sets)
    pub executed_data: BlockExecutedData1,
    /// Executed data for VM2 (txn_infos, events, table_infos)
    pub executed_data2: BlockExecutedData2,
}

pub struct OpenedBlock {
    previous_block_info: BlockInfo,
    block_meta: BlockMetadata,
    gas_limit: u64,

    state: (Arc<ChainStateDB>, Arc<ChainStateDB2>),
    txn_accumulator: MerkleAccumulator,
    vm_state_accumulator: MerkleAccumulator,

    gas_used: u64,
    total_fee2: u128,
    included_user_txns: Vec<SignedUserTransaction>,
    included_user_txns2: Vec<SignedUserTransaction2>,
    uncles: Vec<BlockHeader>,
    chain_id: ChainId,
    difficulty: U256,
    strategy: ConsensusStrategy,
    vm_metrics: Option<VMMetrics>,
    // DAG fields
    version: Version,
    pruning_point: HashValue,
    parents_hash: Vec<HashValue>,
    // Tracked execution data for VM1
    vm1_txn_infos: Vec<TransactionInfo>,
    vm1_txn_events: Vec<Vec<ContractEvent>>,
    vm1_write_sets: Vec<WriteSet>,
    vm1_table_infos: BTreeMap<TableHandle, TableInfo>,
    // Tracked execution data for VM2
    vm2_txn_infos: Vec<starcoin_vm2_types::transaction::TransactionInfo>,
    vm2_txn_events: Vec<Vec<ContractEvent2>>,
    // Note: VM2 doesn't track table_infos in its output (TransactionAuxiliaryData instead)
}

impl OpenedBlock {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        storage: Arc<dyn Store>,
        _storage2: Arc<dyn Store2>,
        previous_header: BlockHeader,
        block_gas_limit: u64,
        author: AccountAddress,
        block_timestamp: u64,
        uncles: Vec<BlockHeader>,
        difficulty: U256,
        strategy: ConsensusStrategy,
        vm_metrics: Option<VMMetrics>,
        tips_hash: Vec<HashValue>,
        version: Version,
        pruning_point: HashValue,
        red_blocks: u64,
        chain_state_dbs: (Arc<ChainStateDB>, Arc<ChainStateDB2>),
    ) -> Result<Self> {
        let previous_block_id = previous_header.id();
        let block_info = storage
            .get_block_info(previous_block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", previous_block_id))?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let vm_state_accumulator_info = block_info.get_vm_state_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new_with_info(
            txn_accumulator_info.clone(),
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let vm_state_accumulator = MerkleAccumulator::new_with_info(
            vm_state_accumulator_info.clone(),
            storage.get_accumulator_store(AccumulatorStoreType::VMState),
        );

        let chain_id = previous_header.chain_id();
        let block_meta = BlockMetadata::new(
            previous_block_id,
            block_timestamp,
            author,
            uncles.len() as u64,
            previous_header.number() + 1,
            chain_id.id().into(),
            previous_header.gas_used(),
            tips_hash.clone(),
            red_blocks,
        );

        let mut opened_block = Self {
            previous_block_info: block_info,
            block_meta,
            gas_limit: block_gas_limit,
            state: chain_state_dbs,
            txn_accumulator,
            vm_state_accumulator,
            gas_used: 0,
            total_fee2: 0,
            included_user_txns: vec![],
            included_user_txns2: vec![],
            uncles,
            chain_id,
            difficulty,
            strategy,
            vm_metrics,
            version,
            pruning_point,
            parents_hash: tips_hash.clone(),
            vm1_txn_infos: vec![],
            vm1_txn_events: vec![],
            vm1_write_sets: vec![],
            vm1_table_infos: BTreeMap::new(),
            vm2_txn_infos: vec![],
            vm2_txn_events: vec![],
        };

        opened_block.initialize()?;

        Ok(opened_block)
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    // TODO: should use check_sub or not
    pub fn gas_left(&self) -> u64 {
        debug_assert!(self.gas_limit >= self.gas_used);
        self.gas_limit - self.gas_used
    }

    pub fn accumulator_root(&self) -> HashValue {
        self.txn_accumulator.root_hash()
    }

    pub fn block_meta(&self) -> &BlockMetadata {
        &self.block_meta
    }

    /// Convert VM2 BlockMetadata to VM1 format with uncles set to 0
    pub fn convert_block_meta_to_legacy(&self) -> BlockMetadataLegacy {
        block_metadata::from(self.block_meta.clone())
    }
    pub fn block_number(&self) -> u64 {
        self.block_meta.number()
    }

    pub fn state_reader(&self) -> Arc<impl ChainStateReader> {
        self.state.0.clone()
    }

    pub fn state_reader2(&self) -> Arc<impl ChainStateReader2> {
        self.state.1.clone()
    }

    pub fn state_db(&self) -> Arc<ChainStateDB> {
        self.state.0.clone()
    }

    pub fn state_db2(&self) -> Arc<ChainStateDB2> {
        self.state.1.clone()
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    /// Try to add `user_txns` into this block.
    /// Return any txns  not included, either txn is discarded, or block gas limit is reached.
    /// If error occurs during the processing, the `open_block` should be dropped,
    /// as the internal state may be corrupted.
    /// TODO: make the function can be called again even last call returns error.  
    pub fn push_txns(&mut self, user_txns: Vec<SignedUserTransaction>) -> Result<ExcludedTxns> {
        let (state, _state2) = &self.state;
        let mut discard_txns = Vec::new();
        let mut txns: Vec<_> = user_txns
            .into_iter()
            .filter(|txn| {
                let is_blacklisted = AddressFilter::is_blacklisted(self.block_number());
                // Discard the txns send by the account in black list after a block number.
                if is_blacklisted {
                    discard_txns.push(txn.clone().into());
                }
                !is_blacklisted
            })
            .map(Transaction::UserTransaction)
            .collect();

        let txn_outputs = {
            let gas_left = self.gas_limit.checked_sub(self.gas_used).ok_or_else(|| {
                format_err!(
                    "block gas_used {} exceed block gas_limit:{}",
                    self.gas_used,
                    self.gas_limit
                )
            })?;
            execute_block_transactions(state, txns.clone(), gas_left, self.vm_metrics.clone())?
        };

        let untouched_user_txns: Vec<MultiSignedUserTransaction> =
            if txn_outputs.len() >= txns.len() {
                vec![]
            } else {
                txns.drain(txn_outputs.len()..)
                    .map(|t| t.try_into().expect("user txn"))
                    .collect()
            };
        debug_assert_eq!(txns.len(), txn_outputs.len());
        let last_index = txn_outputs.len().saturating_sub(1);
        for (index, (txn, output)) in txns.into_iter().zip(txn_outputs.into_iter()).enumerate() {
            let txn_hash = txn.id();
            match output.status() {
                TransactionStatus::Discard(status) => {
                    debug!("discard txn {}, vm status: {:?}", txn_hash, status);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Keep(status) => {
                    if status != &KeptVMStatus::Executed {
                        debug!("txn {:?} execute error: {:?}", txn_hash, status);
                    }
                    let gas_used = output.gas_used();
                    self.push_txn_and_state(txn_hash, output, index == last_index)?;
                    self.gas_used += gas_used;
                    self.included_user_txns
                        .push(txn.try_into().expect("user txn"));
                }
                TransactionStatus::Retry => {
                    debug!("impossible retry txn {}", txn_hash);
                    discard_txns.push(txn.try_into().expect("user txn"));
                }
            };
        }

        Ok(ExcludedTxns {
            discarded_txns: discard_txns,
            untouched_txns: untouched_user_txns,
        })
    }

    /// Run blockmeta first
    fn execute_block_meta_vm1(&mut self) -> Result<()> {
        let (state, _state2) = &self.state;
        let vm1_metadata = self.convert_block_meta_to_legacy();
        debug!("VM1 BlockMetadata: {:?}", vm1_metadata);
        debug!("VM2 BlockMetadata (original): {:?}", self.block_meta);
        let block_metadata_txn = Transaction::BlockMetadata(vm1_metadata.clone());
        let block_meta_txn_hash = block_metadata_txn.id();
        let mut results =
            execute_transactions(state, vec![block_metadata_txn], self.vm_metrics.clone())
                .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        let output = results.pop().expect("execute txn has output");

        match output.status() {
            TransactionStatus::Discard(status) => {
                bail!(
                    "block_metadata txn {:?} is discarded, vm status: {:?}",
                    vm1_metadata,
                    status
                );
            }
            TransactionStatus::Keep(_) => {
                let _ = self.push_txn_and_state(block_meta_txn_hash, output, true)?;
            }
            TransactionStatus::Retry => {
                bail!(
                    "block_metadata txn {:?} is retry impossible",
                    self.block_meta
                );
            }
        };
        Ok(())
    }

    pub fn process_vm1_transactions(
        &mut self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<ExcludedTxns> {
        self.execute_block_meta_vm1()?;
        match self.push_txns(txns) {
            Ok(excluded_txns) => Ok(excluded_txns),
            Err(e) => bail!("[BlockProcess] push txns error: {}", e),
        }
    }

    fn push_txn_and_state(
        &mut self,
        txn_hash: HashValue,
        output: TransactionOutput,
        root_state_calc: bool,
    ) -> Result<(Option<HashValue>, HashValue)> {
        let (state, _state2) = &mut self.state;
        // Extract table_infos and merge them into vm1_table_infos
        let (mut table_infos, write_set, events, gas_used, status) = output.into_inner();
        debug_assert!(matches!(status, TransactionStatus::Keep(_)));
        let status = status
            .status()
            .expect("TransactionStatus at here must been KeptVMStatus");

        // Track table_infos (merge into existing, keeping latest for same TableHandle)
        self.vm1_table_infos.append(&mut table_infos);

        // Track write_set for later use
        self.vm1_write_sets.push(write_set.clone());

        state
            .apply_write_set(write_set)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        let txn_state_root = if root_state_calc {
            Some(
                state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?,
            )
        } else {
            None
        };
        let txn_info = TransactionInfo::new(
            txn_hash,
            txn_state_root,
            events.as_slice(),
            gas_used,
            status,
        );

        // Track txn_info and events
        self.vm1_txn_infos.push(txn_info.clone());
        self.vm1_txn_events.push(events);

        let accumulator_root = self.txn_accumulator.append(&[txn_info.id()])?;
        Ok((txn_state_root, accumulator_root))
    }

    pub fn included_user_txns2(&self) -> &[SignedUserTransaction2] {
        &self.included_user_txns2
    }

    /// Construct a block template for mining.
    /// Returns FinalizedBlock containing the template, outputs, and StateDBs for caching.
    pub fn finalize(self) -> Result<FinalizedBlock> {
        let accumulator_root = self.txn_accumulator.root_hash();
        // update state_root accumulator, state_root order is important
        let (state_root, state_root1, state_root2) = {
            self.vm_state_accumulator
                .append(&[self.state.0.state_root(), self.state.1.state_root()])?;
            (
                self.vm_state_accumulator.root_hash(),
                self.state.0.state_root(),
                self.state.1.state_root(),
            )
        };
        let uncles = if !self.uncles.is_empty() {
            Some(self.uncles.clone())
        } else {
            None
        };
        let body = BlockBody::new_v2(self.included_user_txns, self.included_user_txns2, uncles);
        let block_template = BlockTemplate::new(
            self.previous_block_info
                .block_accumulator_info
                .accumulator_root,
            accumulator_root,
            state_root,
            state_root1,
            state_root2,
            self.gas_used,
            body,
            self.chain_id,
            self.difficulty,
            self.strategy,
            self.block_meta.clone(),
            self.version,
            self.pruning_point,
            self.parents_hash.clone(),
        );

        // Build BlockExecutedData for VM1
        let executed_data = BlockExecutedData1 {
            state_root: state_root1,
            txn_infos: self.vm1_txn_infos,
            txn_events: self.vm1_txn_events,
            txn_table_infos: self.vm1_table_infos,
            write_sets: self.vm1_write_sets,
        };

        // Build BlockExecutedData for VM2 (no write_sets field in VM2)
        // Note: VM2 doesn't track table_infos in its TransactionOutput (uses TransactionAuxiliaryData instead)
        let executed_data2 = BlockExecutedData2 {
            state_root: state_root2,
            txn_infos: self.vm2_txn_infos,
            txn_events: self.vm2_txn_events,
            txn_table_infos: BTreeMap::new(),
        };

        Ok(FinalizedBlock {
            template: block_template,
            statedb: self.state.0,
            statedb2: self.state.1,
            executed_data,
            executed_data2,
        })
    }
}

pub struct AddressFilter;
//static BLACKLIST: [&str; 0] = [];
impl AddressFilter {
    const FROZEN_BEGIN_BLOCK_NUMBER: BlockNumber = 16801958;
    const FROZEN_END_BLOCK_NUMBER: BlockNumber = 23026635;
    pub fn is_blacklisted(block_number: BlockNumber) -> bool {
        block_number > Self::FROZEN_BEGIN_BLOCK_NUMBER
            && block_number < Self::FROZEN_END_BLOCK_NUMBER
        /*&& BLACKLIST
            .iter()
            .map(|&s| AccountAddress::from_str(s).expect("account address decode must success"))
            .any(|x| x == raw_txn.sender())
        */
    }
}
