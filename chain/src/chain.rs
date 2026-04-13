// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    fixed_blocks::MAIN_DIRECT_SAVE_BLOCK_HASH_MAP,
    get_merge_bound_hash,
    txn_output_cache::global_block_state_cache,
    verifier::{BlockVerifier, FullVerifier},
};
use anyhow::{bail, ensure, format_err, Result};
use sp_utils::stop_watch::{watch, CHAIN_WATCH_NAME};
use starcoin_accumulator::inmemory::InMemoryAccumulator;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_chain_api::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, EventWithProof, EventWithProof2,
    ExcludedTxns, ExecutedBlock, MultiEventWithProof, MultiStateProof, TransactionInfoWithProof,
    VerifiedBlock, VerifyBlockField,
};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_consensus::Consensus;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::consensus_state::DagState;
use starcoin_dag::consensusdb::prelude::StoreError;
use starcoin_dag::consensusdb::schemadb::GhostdagStoreReader;
use starcoin_dag::consensusdb::schemadb::ReachabilityStore;
use starcoin_dag::reachability::inquirer;
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_executor::{BlockExecutedData, VMMetrics};
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_pipeline_timing::global_collector;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{Store, Store2};
use starcoin_time_service::TimeService;
use starcoin_types::filter::Filter;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::{StcRichTransactionInfo, StcTransaction, Transaction2};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber, BlockTemplate},
    block_metadata::{self},
    contract_event::ContractEvent,
    error::BlockExecutorError,
    transaction::Transaction,
    U256,
};
use starcoin_types::{contract_event::StcContractEventInfo, transaction::StcTransactionInfo};
use starcoin_types::{multi_access_path::MultiAccessPath, multi_state::MultiState};
use starcoin_vm2_chain::{build_block_transactions, get_epoch_from_statedb};
use starcoin_vm2_state_api::{
    ChainStateReader as ChainStateReader2, ChainStateWriter as ChainStateWriter2,
};
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_types::transaction::Transaction as Vm2Transaction;
use starcoin_vm2_types::transaction::TransactionInfo as TransactionInfo2;
use starcoin_vm2_vm_types::on_chain_resource::Epoch;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::cmp::min;
use std::iter::Extend;
use std::ops::DerefMut;
use std::option::Option::{None, Some};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

static OUTPUT_BLOCK: AtomicBool = AtomicBool::new(false);

fn expected_vm2_txn_count(transactions2: &[Vm2Transaction]) -> usize {
    let has_block_meta = transactions2
        .iter()
        .any(|txn| matches!(txn, Vm2Transaction::BlockMetadata(_)));
    let has_user_txn = transactions2
        .iter()
        .any(|txn| matches!(txn, Vm2Transaction::UserTransaction(_)));
    transactions2.len() + if has_block_meta && has_user_txn { 1 } else { 0 }
}

pub struct ChainStatusWithBlock {
    pub status: ChainStatus,
    pub head: Block,
    pub multi_state: MultiState,
}

pub struct BlockChain {
    genesis_hash: HashValue,
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    vm_state_accumulator: MerkleAccumulator,
    status: ChainStatusWithBlock,
    statedb: (ChainStateDB, ChainStateDB2),
    storage: (Arc<dyn Store>, Arc<dyn Store2>),
    time_service: Arc<dyn TimeService>,
    epoch: Epoch,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
}

impl BlockChain {
    pub fn new(
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> Result<Self> {
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", head_block_hash))?;
        Self::new_with_uncles(time_service, head, storage, storage2, vm_metrics, dag)
    }

    fn new_with_uncles(
        time_service: Arc<dyn TimeService>,
        head_block: Block,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> Result<Self> {
        let block_info = storage
            .get_block_info(head_block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", head_block.id()))?;
        debug!("Init chain with block_info: {:?}", block_info);
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let vm_state_accumulator_info = block_info.get_vm_state_accumulator_info();

        let txn_accumulator = info_2_accumulator(
            txn_accumulator_info.clone(),
            AccumulatorStoreType::Transaction,
            storage.as_ref(),
        );
        let block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            storage.as_ref(),
        );
        let vm_state_accumulator = info_2_accumulator(
            vm_state_accumulator_info.clone(),
            AccumulatorStoreType::VMState,
            storage.as_ref(),
        );

        let (state_root1, state_root2) = {
            debug!(
                "vm_state_accumulator num_leaves: {}, root: {:?}",
                vm_state_accumulator.num_leaves(),
                vm_state_accumulator.root_hash()
            );
            assert!(
                vm_state_accumulator.num_leaves() > 1,
                "vm_state_accumulator must have at least 2 leaves, but has {}",
                vm_state_accumulator.num_leaves()
            );

            let leaf1_idx = vm_state_accumulator.num_leaves() - 2;
            let leaf2_idx = vm_state_accumulator.num_leaves() - 1;

            debug!("Getting leaf at index {} and {}", leaf1_idx, leaf2_idx);

            let state_root1 = vm_state_accumulator
                .get_leaf(leaf1_idx)?
                .ok_or_else(|| format_err!("Can not find acc leaf at index {}", leaf1_idx))?;

            let state_root2 = vm_state_accumulator
                .get_leaf(leaf2_idx)?
                .ok_or_else(|| format_err!("Can not find acc leaf at index {}", leaf2_idx))?;

            debug!(
                "Retrieved state_root1: {:?}, state_root2: {:?}",
                state_root1, state_root2
            );
            (state_root1, state_root2)
        };

        let chain_state = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root1));
        let chain_state2 = ChainStateDB2::new(storage2.clone().into_super_arc(), Some(state_root2));
        let epoch = get_epoch_from_statedb(&chain_state2)?;
        let genesis = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("Can not find genesis hash in storage."))?;
        watch(CHAIN_WATCH_NAME, "n1253");
        let chain = Self {
            genesis_hash: genesis,
            time_service,
            txn_accumulator,
            block_accumulator,
            vm_state_accumulator,
            status: ChainStatusWithBlock {
                status: ChainStatus::new(head_block.header.clone(), block_info),
                head: head_block,
                multi_state: MultiState::new(state_root1, state_root2),
            },
            statedb: (chain_state, chain_state2),
            storage: (storage, storage2),
            epoch,
            vm_metrics,
            dag: dag.clone(),
        };
        watch(CHAIN_WATCH_NAME, "n1252");
        Ok(chain)
    }

    pub fn new_with_genesis(
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        genesis_epoch: Epoch,
        genesis_block: Block,
        mut dag: BlockDAG,
    ) -> Result<Self> {
        debug_assert!(genesis_block.header().is_genesis());
        let txn_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let block_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        );
        let vm_state_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::VMState),
        );
        let statedb = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let statedb2 = ChainStateDB2::new(storage2.clone().into_super_arc(), None);
        let genesis_header = genesis_block.header().clone();
        let executed_block = Self::execute_block_and_save(
            storage.as_ref(),
            (statedb, statedb2),
            txn_accumulator,
            block_accumulator,
            vm_state_accumulator,
            &genesis_epoch,
            None,
            genesis_block,
            None,
            0, // genesis block has no red blocks
        )?;
        dag = Self::init_dag(dag, genesis_header)?;
        Self::new(
            time_service,
            executed_block.block().id(),
            storage,
            storage2,
            None,
            dag,
        )
    }

    fn init_dag(mut dag: BlockDAG, genesis_header: BlockHeader) -> Result<BlockDAG> {
        let genesis_id = genesis_header.id();
        match dag.get_dag_state(genesis_id) {
            anyhow::Result::Ok(_dag_state) => (),
            Err(e) => match e.downcast::<StoreError>()? {
                StoreError::KeyNotFound(_) => {
                    dag.init_with_genesis(genesis_header)?;
                }
                e => {
                    return Err(e.into());
                }
            },
        }
        Ok(dag)
    }

    pub fn dag(&self) -> BlockDAG {
        self.dag.clone()
    }

    pub fn get_dag_state(&self) -> Result<DagState> {
        let current_pruning_point = self.status().head().pruning_point();
        if current_pruning_point == HashValue::zero() {
            self.dag.get_dag_state(self.genesis_hash)
        } else {
            self.dag.get_dag_state(current_pruning_point)
        }
    }

    pub fn current_block_accumulator_info(&self) -> AccumulatorInfo {
        self.block_accumulator.get_info()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.epoch.strategy())
            .expect("Invalid consensus strategy in epoch")
    }

    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    pub fn create_block_template_simple(
        &self,
        author: AccountAddress,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(
            author,
            None, // No specific parent header
            vec![],
            None, // uncles will be derived from blue blocks
            None, // use default gas limit
            None, // tips will be fetched automatically
            HashValue::zero(),
        )
    }

    pub fn create_block_template_simple_with_txns(
        &self,
        author: AccountAddress,
        user_txns: Vec<MultiSignedUserTransaction>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(
            author,
            None, // No specific parent header
            user_txns,
            None, // uncles will be derived from blue blocks
            None, // use default gas limit
            None, // tips will be fetched automatically
            HashValue::zero(),
        )
    }

    pub fn create_block_template_simple_with_uncles(
        &self,
        author: AccountAddress,
        uncles: Vec<BlockHeader>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(
            author,
            None, // No specific parent header
            vec![],
            Some(uncles),
            None, // use default gas limit
            None, // tips will be fetched automatically
            HashValue::zero(),
        )
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        parent_header: Option<BlockHeader>,
        user_txns: Vec<MultiSignedUserTransaction>,
        uncles: Option<Vec<BlockHeader>>,
        block_gas_limit: Option<u64>,
        tips: Option<Vec<HashValue>>,
        pruning_point: HashValue,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let epoch = self.epoch();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);
        let strategy = self.consensus();
        let difficulty = strategy.calculate_next_difficulty(self)?;

        // Get tips: use provided tips or fetch current DAG tips
        let tips = tips.unwrap_or_else(|| self.get_dag_state().unwrap().tips);

        // Calculate ghostdata from tips
        let ghostdata = self.dag().ghostdata(&tips)?;
        let selected_parents = tips.clone();

        // Get the parent header: use provided header or calculate from ghostdata
        let parent_header = match parent_header {
            Some(header) => header,
            None => self
                .storage
                .0
                .get_block_header_by_hash(ghostdata.selected_parent)?
                .ok_or_else(|| {
                    format_err!(
                        "Cannot find block header by {:?}",
                        ghostdata.selected_parent
                    )
                })?,
        };
        debug!(
            "Blue blocks:{:?} in chain/create_block_template_by_header",
            ghostdata.mergeset_blues
        );
        let blue_blocks = ghostdata
            .mergeset_blues
            .as_ref()
            .iter()
            .skip(1)
            .cloned()
            .map(|block| self.storage.0.get_block_by_hash(block))
            .collect::<Result<Vec<Option<Block>>>>()?
            .into_iter()
            .map(|op_block| op_block.expect("failed to get a block"))
            .collect::<Vec<_>>();

        // Use provided uncles or derive from blue blocks
        let uncles = match uncles {
            Some(u) if !u.is_empty() => u,
            _ => blue_blocks
                .iter()
                .map(|block| block.header().clone())
                .collect::<Vec<_>>(),
        };

        // Convert VM1's AccountAddress to VM2's for OpenedBlock
        let author_bytes = author.to_vec();
        let mut author_array = [0u8; 16];
        author_array.copy_from_slice(&author_bytes[..16]);
        let author_v2 = starcoin_vm2_types::account_address::AccountAddress::new(author_array);

        let chain_state = ChainStateDB::new(
            self.storage.0.clone().into_super_arc(),
            Some(self.statedb.0.state_root()),
        );
        let chain_state2 = ChainStateDB2::new(
            self.storage.1.clone().into_super_arc(),
            Some(self.statedb.1.state_root()),
        );

        let mut opened_block = OpenedBlock::new(
            self.storage.0.clone(),
            self.storage.1.clone(),
            parent_header.clone(),
            final_block_gas_limit,
            author_v2,
            self.time_service.now_millis(),
            uncles,
            difficulty,
            strategy,
            None,
            selected_parents,
            0,
            pruning_point,
            ghostdata.mergeset_reds.len() as u64,
            (Arc::new(chain_state), Arc::new(chain_state2)),
        )?;

        // split user_txns to two parts for dual VM support
        let mut vm1_txns = vec![];
        let mut vm2_txns = vec![];
        for txn in user_txns {
            match txn {
                MultiSignedUserTransaction::VM1(txn) => vm1_txns.push(txn),
                MultiSignedUserTransaction::VM2(txn) => vm2_txns.push(txn),
            }
        }
        let excluded_txns: ExcludedTxns = if parent_header.number().saturating_add(1)
            >= vm1_offline_height(parent_header.chain_id().id().into())
        {
            ExcludedTxns {
                discarded_txns: vm1_txns
                    .into_iter()
                    .map(MultiSignedUserTransaction::VM1)
                    .collect(),
                untouched_txns: vec![],
            }
        } else {
            opened_block.process_vm1_transactions(vm1_txns)?
        };

        let vm2_contains_user_transaction = !vm2_txns.is_empty();
        let excluded_txns2 = opened_block.push_txns2(vm2_txns)?;
        if vm2_contains_user_transaction {
            opened_block.finalize_block_epilogue()?;
        }
        let finalized = opened_block.finalize()?;

        Ok((finalized.template, excluded_txns.absorb(excluded_txns2)))
    }

    /// Get block hash by block number, if not exist, return Error.
    pub fn get_hash_by_number_ensure(&self, number: BlockNumber) -> Result<HashValue> {
        self.get_hash_by_number(number)?
            .ok_or_else(|| format_err!("Can not find block hash by number {}", number))
    }

    fn check_exist_block(&self, block_id: HashValue, block_number: BlockNumber) -> Result<bool> {
        Ok(self
            .get_hash_by_number(block_number)?
            .filter(|hash| hash == &block_id)
            .is_some())
    }

    // filter header by check exist
    fn exist_header_filter(&self, header: Option<BlockHeader>) -> Result<Option<BlockHeader>> {
        Ok(match header {
            Some(header) => {
                if self.check_exist_block(header.id(), header.number())? {
                    Some(header)
                } else {
                    None
                }
            }
            None => None,
        })
    }

    pub fn get_storage(&self) -> Arc<dyn Store> {
        self.storage.0.clone()
    }

    pub fn get_storage2(&self) -> Arc<dyn Store2> {
        self.storage.1.clone()
    }

    pub fn can_be_uncle(&self, _block_header: &BlockHeader) -> Result<bool> {
        // DAG blocks don't use the traditional uncle verification
        // This is handled by verify_blue_blocks in the verifier
        Ok(true)
    }

    pub fn verify_with_verifier<V>(&mut self, block: Block) -> Result<VerifiedBlock>
    where
        V: BlockVerifier,
    {
        if self.head_block().header().id() != block.parent_hash() {
            let selected_chain = Self::new(
                self.time_service.clone(),
                block.parent_hash(),
                self.storage.0.clone(),
                self.storage.1.clone(),
                self.vm_metrics.clone(),
                self.dag.clone(),
            )?;
            V::verify_block(&selected_chain, block)
        } else {
            V::verify_block(self, block)
        }
    }

    pub fn apply_with_verifier<V>(&mut self, block: Block) -> Result<ExecutedBlock>
    where
        V: BlockVerifier,
    {
        let verified_block = self.verify_with_verifier::<V>(block)?;
        watch(CHAIN_WATCH_NAME, "n1");
        let executed_block = self.execute(verified_block)?;
        watch(CHAIN_WATCH_NAME, "n2");
        self.connect(executed_block)
    }

    pub fn verify_without_save<V>(&mut self, block: Block) -> Result<ExecutedBlock>
    where
        V: BlockVerifier,
    {
        let verified_block = self.verify_with_verifier::<V>(block)?;
        watch(CHAIN_WATCH_NAME, "n1");
        self.execute_without_save(verified_block)
    }

    //TODO consider move this logic to BlockExecutor
    fn execute_block_and_save(
        storage: &dyn Store,
        statedb: (ChainStateDB, ChainStateDB2),
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        vm_state_accumulator: MerkleAccumulator,
        epoch: &Epoch,
        parent_status: Option<ChainStatus>,
        block: Block,
        vm_metrics: Option<VMMetrics>,
        red_blocks: u64,
    ) -> Result<ExecutedBlock> {
        let (statedb, statedb2) = statedb;
        let header = block.header();
        debug_assert!(header.is_genesis() || parent_status.is_some());
        debug_assert!(!header.is_genesis() || parent_status.is_none());
        let block_id = header.id();
        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let (vm1_offline, mut t) = match &parent_status {
                None => (false, vec![]),
                Some(parent) => {
                    let vm1_offline = vm1_offline_height(parent.head.chain_id().id().into());
                    if header.number() < vm1_offline {
                        let block_metadata =
                            block_metadata::from(block.to_metadata(parent.head().gas_used(), 0));
                        (false, vec![Transaction::BlockMetadata(block_metadata)])
                    } else {
                        (true, vec![])
                    }
                }
            };
            if !vm1_offline {
                t.extend(
                    block
                        .transactions()
                        .iter()
                        .cloned()
                        .map(Transaction::UserTransaction),
                );
            }
            debug_assert!((vm1_offline && t.is_empty()) || (!vm1_offline && !t.is_empty()));
            t
        };

        let transactions2 = build_block_transactions(
            block.transactions2(),
            parent_status
                .as_ref()
                .map(|p| block.to_metadata(p.head.gas_used(), red_blocks)),
        );

        assert!(!transactions2.is_empty());

        // Record VM execution start (only when pipeline timing is enabled)
        let pipeline_enabled = global_collector().is_enabled();
        let vm_exec_start = if pipeline_enabled {
            let txn_count_for_timing = transactions.len() + transactions2.len();
            global_collector().record_vm_exec_start(
                block_id,
                header.number(),
                txn_count_for_timing,
            );
            Some((std::time::Instant::now(), txn_count_for_timing))
        } else {
            None
        };

        watch(CHAIN_WATCH_NAME, "n21");
        let executed_data = starcoin_executor::block_execute(
            &statedb,
            transactions.clone(),
            epoch.block_gas_limit(),
            vm_metrics.clone(),
        )?;

        let executed_data2 = starcoin_vm2_chain::execute_transactions(
            &statedb2,
            transactions2.clone(),
            epoch.block_gas_limit() - executed_data.gas_used(),
            vm_metrics,
        )?;
        watch(CHAIN_WATCH_NAME, "n22");

        // Record VM execution end
        if let Some((start, txn_count)) = vm_exec_start {
            global_collector().record_vm_exec_end(block_id);
            let vm_exec_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            info!(
                "[Pipeline] VM Execute block {} ({} txns) took {:.3} ms",
                block_id, txn_count, vm_exec_duration_ms
            );
        }

        let (state_root, multi_state) = {
            // if no txns, state_root is kept unchanged after calling txn-execution
            let state_root1 = if header.is_genesis()
                && starcoin_data_migration::should_do_migration(header.chain_id())
            {
                starcoin_data_migration::do_migration(&statedb, header.chain_id())?
            } else {
                executed_data.state_root
            };

            let state_root2 = executed_data2.state_root;

            vm_state_accumulator.append(&[state_root1, state_root2])?;
            (
                vm_state_accumulator.root_hash(),
                MultiState::new(state_root1, state_root2),
            )
        };

        let vec_transaction_info = &executed_data.txn_infos;
        let vm2_txn_infos = &executed_data2.txn_infos;

        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root {:?}, in header {:?} fail, multi_state {:?}",
            block_id,
            state_root,
            header.state_root(),
            multi_state
        );
        let vm1_block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc.saturating_add(i.gas_used()));
        let block_gas_used = vm1_block_gas_used.saturating_add(
            vm2_txn_infos
                .iter()
                .fold(0u64, |acc, i| acc.saturating_add(i.gas_used())),
        );
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        let expected_vm2_txn_num = expected_vm2_txn_count(&transactions2);
        verify_block!(
            VerifyBlockField::State,
            vm2_txn_infos.len() == expected_vm2_txn_num,
            "invalid vm2 txn num in the block"
        );

        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            let included_txn_info_hashes2: Vec<_> =
                vm2_txn_infos.iter().map(|info| info.id()).collect();
            // NO need to check whether info_hashes is empty or not, accmulator.append will handle it.
            if included_txn_info_hashes.is_empty() {
                txn_accumulator.append(&included_txn_info_hashes2)?;
            } else {
                txn_accumulator.append(
                    included_txn_info_hashes2
                        .get(0..1)
                        .expect("vm2 block meta transaction is none"),
                )?;
                txn_accumulator.append(&included_txn_info_hashes)?;
                txn_accumulator.append(included_txn_info_hashes2.get(1..).unwrap_or_default())?;
            }
            txn_accumulator.root_hash()
        };

        verify_block!(
            VerifyBlockField::Transaction,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch in execute block and save, block_id: {:?}, executed_accumulator_root: {:?}, header.txn_accumulator_root: {:?}",  
            block_id, executed_accumulator_root, header.txn_accumulator_root(),
        );

        watch(CHAIN_WATCH_NAME, "n23");
        // Record state commit timing (only when pipeline timing is enabled)
        let commit_start = if pipeline_enabled {
            let txn_count = vm_exec_start.map(|(_, c)| c).unwrap_or(0);
            global_collector().record_state_commit_start(block_id, header.number(), txn_count);
            Some((std::time::Instant::now(), txn_count))
        } else {
            None
        };

        statedb2
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        // Record state commit end
        if let Some((start, txn_count)) = commit_start {
            global_collector().record_state_commit_end(block_id);
            let commit_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            info!(
                "[Pipeline] State Commit block {} ({} txns) took {:.3} ms",
                block_id, txn_count, commit_duration_ms
            );
        }

        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        watch(CHAIN_WATCH_NAME, "n24");
        vm_state_accumulator
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;

        let pre_total_difficulty = parent_status
            .map(|status| status.total_difficulty())
            .unwrap_or_default();

        let total_difficulty = pre_total_difficulty + header.difficulty();

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let vm_state_accumulator_info: AccumulatorInfo = vm_state_accumulator.get_info();
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
            vm_state_accumulator_info,
        );

        watch(CHAIN_WATCH_NAME, "n25");

        // save block's transaction relationship and save transaction

        let block_id = block.id();
        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .chain(
                executed_data2
                    .txn_table_infos
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into())),
            )
            .collect::<Vec<_>>();

        debug_assert!(
            txn_events.len() == txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(
            txn_events
                .into_iter()
                .map(|events| events.into_iter().map(Into::into).collect::<Vec<_>>()),
        ) {
            storage.save_contract_events_v2(*info_id, events)?;
        }

        // save vm2 txn events
        let vm2_txn_info_ids: Vec<_> = vm2_txn_infos.iter().map(|info| info.id()).collect();
        {
            debug_assert!(
                executed_data2.txn_events.len() == vm2_txn_infos.len(),
                "vm2 events' length should be equal to txn infos' length"
            );
            for (info_id, events) in vm2_txn_info_ids.iter().zip(
                executed_data2
                    .txn_events
                    .into_iter()
                    .map(|events| events.into_iter().map(Into::into).collect::<Vec<_>>()),
            ) {
                storage.save_contract_events_v2(*info_id, events)?;
            }
        }
        let transactions2_for_storage =
            append_vm2_epilogue_for_storage(transactions2, vm2_txn_infos)?;

        if txn_infos.is_empty() {
            storage.save_transaction_infos(
                executed_data2
                    .txn_infos
                    .into_iter()
                    .map(Into::into)
                    .enumerate()
                    .map(|(transaction_index, info)| {
                        StcRichTransactionInfo::new(
                            block_id,
                            header.number(),
                            info,
                            transaction_index as u32,
                            transaction_global_index
                                .checked_add(transaction_index as u64)
                                .expect("transaction_global_index overflow."),
                        )
                    })
                    .collect(),
            )?;
        } else {
            storage.save_transaction_infos(
                std::iter::once::<StcTransactionInfo>(
                    executed_data2
                        .txn_infos
                        .first()
                        .expect("vm2 block meta transaction is none")
                        .clone()
                        .into(),
                )
                .chain(txn_infos.into_iter().map(Into::into))
                .chain(executed_data2.txn_infos.into_iter().skip(1).map(Into::into))
                .enumerate()
                .map(|(transaction_index, info)| {
                    StcRichTransactionInfo::new(
                        block_id,
                        header.number(),
                        info,
                        transaction_index as u32,
                        transaction_global_index
                            .checked_add(transaction_index as u64)
                            .expect("transaction_global_index overflow."),
                    )
                })
                .collect(),
            )?;
        }

        let all_transactions: Vec<StcTransaction> = transactions
            .into_iter()
            .map(Into::into)
            .chain(transactions2_for_storage.into_iter().map(Into::into))
            .collect();
        let txn_id_vec = all_transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save transactions
        storage.save_transaction_batch(all_transactions)?;

        // save block's transactions
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        storage.save_block_txn_info_ids(
            block_id,
            std::iter::once(vm2_txn_info_ids[0])
                .chain(txn_info_ids)
                .chain(vm2_txn_info_ids.into_iter().skip(1))
                .collect(),
        )?;
        storage.commit_block(block.clone())?;

        storage.save_block_info(block_info.clone())?;

        storage.save_table_infos(txn_table_infos)?;

        watch(CHAIN_WATCH_NAME, "n26");
        Ok(ExecutedBlock::new(block, block_info, multi_state))
    }

    fn execute_save_directly(
        storage: &dyn Store,
        statedb: ChainStateDB,
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        parent_status: Option<ChainStatus>,
        block: Block,
        block_info: BlockInfo,
        executed_data: BlockExecutedData,
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        let block_id = header.id();

        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let mut t = match &parent_status {
                None => vec![],
                Some(parent) => {
                    let block_metadata =
                        block_metadata::from(block.to_metadata(parent.head().gas_used(), 0));
                    vec![Transaction::BlockMetadata(block_metadata)]
                }
            };
            t.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };
        for write_set in executed_data.write_sets {
            statedb
                .apply_write_set(write_set)
                .map_err(BlockExecutorError::BlockChainStateErr)?;
            statedb
                .commit()
                .map_err(BlockExecutorError::BlockChainStateErr)?;
        }
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::State,
            statedb.state_root() == header.state_root(),
            "verify block:{:?} state_root fail",
            block_id,
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::Transaction,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch in execute save directly, block id: {}, executed_accumulator_root: {:?}, header txn_accumulator_root: {:?}",
            block_id, executed_accumulator_root, header.txn_accumulator_root()
        );

        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        verify_block!(
            VerifyBlockField::State,
            txn_accumulator_info == block_info.txn_accumulator_info,
            "directly verify block: txn accumulator info mismatch"
        );

        verify_block!(
            VerifyBlockField::State,
            block_accumulator_info == block_info.block_accumulator_info,
            "directly verify block: block accumulator info mismatch"
        );

        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect::<Vec<_>>();

        // save block's transaction relationship and save transaction
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            storage.save_contract_events(*info_id, events)?;
        }

        storage.save_transaction_infos(
            txn_infos
                .into_iter()
                .enumerate()
                .map(|(transaction_index, info)| {
                    StcRichTransactionInfo::new(
                        block_id,
                        block.header().number(),
                        info.into(),
                        transaction_index as u32,
                        transaction_global_index
                            .checked_add(transaction_index as u64)
                            .expect("transaction_global_index overflow."),
                    )
                })
                .collect(),
        )?;

        let txn_id_vec = transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save transactions
        storage.save_transaction_batch(transactions.into_iter().map(Into::into).collect())?;

        // save block's transactions
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
        storage.commit_block(block.clone())?;

        storage.save_block_info(block_info.clone())?;

        storage.save_table_infos(txn_table_infos)?;

        Ok(ExecutedBlock::new(block, block_info, MultiState::default()))
    }

    pub fn set_output_block() {
        OUTPUT_BLOCK.store(true, Ordering::Relaxed);
    }

    fn execute_block_without_save(
        statedb: ChainStateDB,
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        epoch: &Epoch,
        parent_status: Option<ChainStatus>,
        block: Block,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        debug_assert!(header.is_genesis() || parent_status.is_some());
        debug_assert!(!header.is_genesis() || parent_status.is_none());
        let block_id = header.id();
        let transactions = {
            // genesis block do not generate BlockMetadata transaction.
            let mut t = match &parent_status {
                None => vec![],
                Some(parent) => {
                    let block_metadata =
                        block_metadata::from(block.to_metadata(parent.head().gas_used(), 0));
                    vec![Transaction::BlockMetadata(block_metadata)]
                }
            };
            t.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            t
        };

        watch(CHAIN_WATCH_NAME, "n21");
        let executed_data = starcoin_executor::block_execute(
            &statedb,
            transactions.clone(),
            epoch.block_gas_limit(),
            vm_metrics,
        )?;
        watch(CHAIN_WATCH_NAME, "n22");
        let state_root = executed_data.state_root;
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root fail",
            block_id,
        );
        let block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc.saturating_add(i.gas_used()));
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == transactions.len(),
            "invalid txn num in the block"
        );

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::Transaction,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch in executed block without save, block_id: {:?}, expected accumulator root: {:?}, header accumulator root: {:?}",
            block_id, executed_accumulator_root, header.txn_accumulator_root(),
        );

        let pre_total_difficulty = parent_status
            .map(|status| status.total_difficulty())
            .unwrap_or_default();
        let total_difficulty = pre_total_difficulty + header.difficulty();
        block_accumulator.append(&[block_id])?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
            // todo: just add default info for compatibility.
            AccumulatorInfo::default(),
        );

        watch(CHAIN_WATCH_NAME, "n25");

        debug_assert!(
            executed_data.txn_events.len() == executed_data.txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        if OUTPUT_BLOCK.load(Ordering::Relaxed) {
            println!("// {}", block.header().number());
            println!("maps.insert(");
            println!("HashValue::from_hex_literal(\"{}\").unwrap(),", block.id());
            println!(
                "(\nserde_json::from_str({:?}).unwrap(),",
                serde_json::to_string(&executed_data)?
            );
            println!(
                "\nserde_json::from_str({:?}).unwrap()\n)\n);",
                serde_json::to_string(&block_info)?
            );
        }

        Ok(ExecutedBlock::new(block, block_info, MultiState::default()))
    }

    pub fn get_txn_accumulator(&self) -> &MerkleAccumulator {
        &self.txn_accumulator
    }

    pub fn get_block_accumulator(&self) -> &MerkleAccumulator {
        &self.block_accumulator
    }

    pub fn get_vm_state_accumulator(&self) -> &MerkleAccumulator {
        &self.vm_state_accumulator
    }

    pub fn into_state_dbs(self) -> (Arc<ChainStateDB>, Arc<ChainStateDB2>) {
        (Arc::new(self.statedb.0), Arc::new(self.statedb.1))
    }
}

impl ChainReader for BlockChain {
    fn info(&self) -> ChainInfo {
        ChainInfo::new(
            self.status.head.header().chain_id(),
            self.genesis_hash,
            self.status.status.clone(),
        )
    }

    fn status(&self) -> ChainStatus {
        self.status.status.clone()
    }

    fn head_block(&self) -> ExecutedBlock {
        ExecutedBlock::new(
            self.status.head.clone(),
            self.status.status.info.clone(),
            self.status.multi_state.clone(),
        )
    }

    fn current_header(&self) -> BlockHeader {
        self.status.status.head().clone()
    }

    /// Get header by hash with filtering - only returns headers that exist on the main chain.
    /// WARNING: This function filters out uncle/fork blocks in DAG mode!
    /// Use get_header_by_hash() if you need to access all blocks including uncles.
    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        let (storage, _storage2) = &self.storage;
        storage
            .get_block_header_by_hash(hash)
            .and_then(|block_header| self.exist_header_filter(block_header))
    }

    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        let (storage, _storage2) = &self.storage;
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => storage.get_block_header_by_hash(block_id),
            })
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        let (storage, _storage2) = &self.storage;
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => storage.get_block_by_hash(block_id),
            })
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>> {
        let (storage, _storage2) = &self.storage;
        let end_num = match number {
            None => self.current_header().number(),
            Some(number) => number,
        };

        let num_leaves = self.block_accumulator.num_leaves();

        if end_num > num_leaves.saturating_sub(1) {
            bail!("Can not find block by number {}", end_num);
        };

        let len = if !reverse && (end_num.saturating_add(count) > num_leaves.saturating_sub(1)) {
            num_leaves.saturating_sub(end_num)
        } else {
            count
        };

        let ids = self.get_block_ids(end_num, reverse, len)?;
        let block_opts = storage.get_blocks(ids)?;
        let mut blocks = vec![];
        for (idx, block) in block_opts.into_iter().enumerate() {
            match block {
                Some(block) => blocks.push(block),
                None => bail!(
                    "Can not find block by number {}",
                    end_num.saturating_sub(idx as u64)
                ),
            }
        }
        Ok(blocks)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        let (storage, _storage2) = &self.storage;
        storage.get_block_by_hash(hash)
    }

    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>> {
        self.block_accumulator.get_leaf(number)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>> {
        let (storage, _) = &self.storage;
        //TODO check txn should exist on current chain.
        storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<StcRichTransactionInfo>> {
        let (storage, _storage2) = &self.storage;
        let txn_info_ids = storage.get_rich_transaction_info_ids_by_txn_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            let txn_info = storage.get_transaction_info_by_rich_info_id(txn_info_id)?;
            if let Some(txn_info) = txn_info {
                if self.exist_block(txn_info.block_id())? {
                    return Ok(Some(txn_info));
                }
            }
        }
        Ok(None)
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.statedb.0
    }

    fn chain_state_reader2(&self) -> &dyn ChainStateReader2 {
        &self.statedb.1
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        let (storage, _storage2) = &self.storage;
        match block_id {
            Some(block_id) => storage.get_block_info(block_id),
            None => Ok(Some(self.status.status.info().clone())),
        }
    }

    fn get_total_difficulty(&self) -> Result<U256> {
        Ok(self.status.status.total_difficulty())
    }

    fn exist_block(&self, block_id: HashValue) -> Result<bool> {
        let (storage, _storage2) = &self.storage;
        if let Some(header) = storage.get_block_header_by_hash(block_id)? {
            return self.check_exist_block(block_id, header.number());
        }
        Ok(false)
    }

    fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    fn consensus_strategy(&self) -> ConsensusStrategy {
        self.consensus()
    }

    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        self.block_accumulator
            .get_leaves(start_number, reverse, max_size)
    }

    fn get_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>> {
        let block = self
            .get_block_by_number(number)?
            .ok_or_else(|| format_err!("Can not find block by number {}", number))?;

        self.get_block_info(Some(block.id()))
    }

    fn time_service(&self) -> &dyn TimeService {
        self.time_service.as_ref()
    }

    fn fork(&self, block_id: HashValue) -> Result<Self> {
        let (storage, storage2) = &self.storage;
        ensure!(
            self.has_dag_block(block_id)?,
            "Block with id{} do not exists in current chain.",
            block_id
        );
        let head = storage
            .get_block_by_hash(block_id)?
            .ok_or_else(|| format_err!("Cannot find block by hash {:?}", block_id))?;
        BlockChain::new_with_uncles(
            self.time_service.clone(),
            head,
            storage.clone(),
            storage2.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )
    }

    fn find_ancestor(&self, another: &dyn ChainReader) -> Result<Option<BlockIdAndNumber>> {
        let other_header_number = another.current_header().number();
        let self_header_number = self.current_header().number();
        let min_number = std::cmp::min(other_header_number, self_header_number);
        let mut ancestor = None;
        for block_number in (0..min_number).rev() {
            let block_id_1 = another.get_hash_by_number(block_number)?;
            let block_id_2 = self.get_hash_by_number(block_number)?;
            match (block_id_1, block_id_2) {
                (Some(block_id_1), Some(block_id_2)) => {
                    if block_id_1 == block_id_2 {
                        ancestor = Some(BlockIdAndNumber::new(block_id_1, block_number));
                        break;
                    }
                }
                (_, _) => {
                    continue;
                }
            }
        }
        Ok(ancestor)
    }

    fn verify(&self, block: Block) -> Result<VerifiedBlock> {
        FullVerifier::verify_block(self, block)
    }

    fn execute(&mut self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        // Check if this is a pre-computed block
        if let Some((executed_data, block_info)) =
            MAIN_DIRECT_SAVE_BLOCK_HASH_MAP.get(&verified_block.block.header.id())
        {
            Self::execute_save_directly(
                self.storage.0.as_ref(),
                self.statedb.0.fork(),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                Some(self.status.status.clone()),
                verified_block.block,
                block_info.clone().into(),
                executed_data.clone(),
            )
        } else {
            // Use DAG execution with multi-VM support
            self.execute_dag_block(verified_block)
        }
    }

    fn execute_without_save(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        Self::execute_block_without_save(
            self.statedb.0.fork(),
            self.txn_accumulator.fork(None),
            self.block_accumulator.fork(None),
            &self.epoch,
            Some(self.status.status.clone()),
            verified_block.block,
            self.vm_metrics.clone(),
        )
    }

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<StcRichTransactionInfo>> {
        let (storage, _storage2) = &self.storage;
        let chain_header = self.current_header();
        let hashes = self
            .txn_accumulator
            .get_leaves(start_index, reverse, max_size)?;
        let mut infos = vec![];
        let txn_infos =
            storage.get_transaction_infos_by_info_ids(hashes.clone(), chain_header.id())?;
        for (i, info) in txn_infos.into_iter().enumerate() {
            match info {
                Some(info) => infos.push(info),
                None => bail!(
                    "cannot find hash({:?}) on head: {}",
                    hashes.get(i),
                    chain_header.id()
                ),
            }
        }
        Ok(infos)
    }

    fn get_events(&self, txn_info_id: HashValue) -> Result<Option<Vec<ContractEvent>>> {
        let (storage, _storage2) = &self.storage;
        storage.get_contract_events(txn_info_id)
    }

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        let (storage, _storage2) = &self.storage;
        let (statedb, statedb2) = &self.statedb;

        let block_info = match storage.get_block_info(block_id)? {
            Some(block_info) => block_info,
            None => return Ok(None),
        };

        let transaction_accumulator = MerkleAccumulator::new_with_info(
            block_info.txn_accumulator_info,
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );

        let final_transaction_global_index = transaction_accumulator
            .num_leaves()
            .checked_sub(1)
            .ok_or_else(|| format_err!("accumulator is empty"))?;
        let final_proof = transaction_accumulator
            .get_proof(final_transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "final transaction proof is not found using the accumulator leaf index: {:?}",
                    final_transaction_global_index
                )
            })?;

        let proof = transaction_accumulator
            .get_proof(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "transaction proof is not found using the accumulator leaf index: {:?}",
                    transaction_global_index
                )
            })?;

        let transaction_info_id = transaction_accumulator
            .get_leaf(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "transaction info is not found using the accumulator leaf index: {}",
                    transaction_global_index
                )
            })?;

        let proof_transaction_info_id = transaction_accumulator
            .get_leaf(final_transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "proof transaction info is not found using the accumulator leaf index: {}",
                    final_transaction_global_index
                )
            })?;

        let get_transaction_info_in_this_chain =
            |id: HashValue| -> Result<Option<StcRichTransactionInfo>> {
                // vm1's TransactionInfo's id and StcRichTransactionInfo's id are same,
                // vm2's TransactionInfo's id and StcRichTransactionInfo's id are different
                // we are not sure whether the id is from vm1 or vm2, so we need to check both.
                let mut rich_ids = storage.get_rich_ids_by_info_id(id).unwrap_or(vec![]);
                rich_ids.push(id);
                let rich_infos = storage.get_transaction_infos_by_rich_ids(rich_ids)?;
                for rich_info in rich_infos {
                    if rich_info.is_none() {
                        continue;
                    }
                    let rich_info = rich_info.unwrap();
                    if self.check_exist_block(rich_info.block_id(), rich_info.block_number)? {
                        return Ok(Some(rich_info));
                    }
                }
                Ok(None)
            };

        let proof_rich_transaction_info =
            get_transaction_info_in_this_chain(proof_transaction_info_id)?.ok_or_else(|| {
                format_err!(
                    "failed to get proof txn info by hash:{}, block id:{}",
                    proof_transaction_info_id,
                    block_id
                )
            })?;

        let proof_transaction_info = proof_rich_transaction_info.transaction_info;

        let input_rich_transaction_info = get_transaction_info_in_this_chain(transaction_info_id)?
            .ok_or_else(|| {
                format_err!(
                    "failed to get txn info by hash:{}, block id:{}",
                    transaction_info_id,
                    block_id
                )
            })?;

        let final_raw_transaction = storage
            .get_transaction(proof_transaction_info.transaction_hash())?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find txn by hash:{}",
                    proof_transaction_info.transaction_hash()
                )
            })?
            .to_transaction();

        let final_state_root_hash = proof_transaction_info
            .state_root()
            .ok_or_else(|| format_err!("Cannot get state root hash"))?;
        let final_access_path = final_raw_transaction.access_path();

        let transaction_info = get_transaction_info_in_this_chain(transaction_info_id)?
            .ok_or_else(|| {
                format_err!(
                    "failed to get txn info by hash:{}, block id:{}",
                    transaction_info_id,
                    block_id
                )
            })?;

        let event_proof = match input_rich_transaction_info.transaction_info {
            StcTransactionInfo::V1(_) => {
                if let Some(event_index) = event_index {
                    let events: Vec<ContractEvent> = storage
                        .get_contract_events(transaction_info_id)?
                        .unwrap_or_default();
                    let event = events.get(event_index as usize).cloned().ok_or_else(|| {
                        format_err!("event index out of range, events len:{}", events.len())
                    })?;
                    let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();

                    let event_proof = InMemoryAccumulator::get_proof_from_leaves(
                        event_hashes.as_slice(),
                        event_index,
                    )?;
                    Some(MultiEventWithProof::VM1(EventWithProof {
                        event,
                        proof: event_proof,
                    }))
                } else {
                    None
                }
            }
            StcTransactionInfo::V2(_) => {
                if let Some(event_index) = event_index {
                    let events = storage
                        .get_contract_events_v2(transaction_info_id)?
                        .unwrap_or_default();
                    let events = events
                        .into_iter()
                        .filter_map(|e| e.to_v2())
                        .collect::<Vec<_>>();
                    let event = events.get(event_index as usize).cloned().ok_or_else(|| {
                        format_err!("event index out of range, events len:{}", events.len())
                    })?;
                    let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();

                    let event_proof = InMemoryAccumulator::get_proof_from_leaves(
                        event_hashes.as_slice(),
                        event_index,
                    )?;
                    Some(MultiEventWithProof::VM2(EventWithProof2 {
                        event,
                        proof: event_proof,
                    }))
                } else {
                    None
                }
            }
        };

        let final_state_proof = if access_path.is_some() {
            match &final_access_path {
                MultiAccessPath::VM1(access_path) => {
                    let statedb = statedb.fork_at(final_state_root_hash);
                    Some(MultiStateProof::VM1(statedb.get_with_proof(access_path)?))
                }
                MultiAccessPath::VM2(_) => {
                    let statedb2 = statedb2.fork_at(final_state_root_hash);
                    let state_key = final_access_path.to_state_key()?.ok_or_else(|| {
                        format_err!(
                                    "the transaction to be verified is vm version 2 but the access path is not"
                                )
                    })?;
                    Some(MultiStateProof::VM2(statedb2.get_with_proof(&state_key)?))
                }
            }
        } else {
            None
        };

        let state_proof = if let Some(access_path) = access_path {
            if let Some(state_root) = transaction_info.state_root() {
                match transaction_info.txn_info() {
                    StcTransactionInfo::V1(_) => {
                        let statedb = statedb.fork_at(state_root);
                        Some(MultiStateProof::VM1(statedb.get_with_proof(&access_path.to_v1().ok_or_else(|| format_err!("the transaction to be verified is vm version 1 but the access path is not"))?)?))
                    }
                    StcTransactionInfo::V2(_) => {
                        let statedb2 = statedb2.fork_at(state_root);
                        Some(MultiStateProof::VM2(statedb2.get_with_proof(&access_path.to_state_key()?.ok_or_else(|| format_err!("the transaction to be verified is vm version 2 but the access path is not"))?)?))
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(TransactionInfoWithProof {
            transaction_info,
            proof,
            final_proof,
            event_proof,
            state_proof,
            final_state_proof,
        }))
    }

    fn current_tips_hash(&self, pruning_point: HashValue) -> Result<Vec<HashValue>> {
        self.dag
            .get_dag_state(pruning_point)
            .map(|state| state.tips)
    }

    fn has_dag_block(&self, header_id: HashValue) -> Result<bool> {
        let header = match self.storage.0.get_block_header_by_hash(header_id)? {
            Some(header) => header,
            None => return Ok(false),
        };

        if self.storage.0.get_block_info(header.id())?.is_none() {
            return Ok(false);
        }

        self.dag.has_block_connected(&header)
    }

    fn calc_ghostdata_and_check_bounded_merge_depth(
        &self,
        header: &BlockHeader,
    ) -> Result<starcoin_dag::types::ghostdata::GhostdagData> {
        let dag = self.dag();

        let ghostdata = dag.calc_ghostdata(header)?;

        dag.check_bounded_merge_depth(
            &ghostdata,
            get_merge_bound_hash(
                ghostdata.selected_parent,
                dag.clone(),
                self.storage.0.clone(),
            )?,
        )?;

        Ok(ghostdata)
    }

    fn is_dag_ancestor_of(&self, ancestor: HashValue, descendant: HashValue) -> Result<bool> {
        self.dag().check_ancestor_of(ancestor, descendant)
    }

    fn get_pruning_height(&self) -> BlockNumber {
        self.get_pruning_height()
    }

    fn get_pruning_config(&self) -> (u64, u64) {
        // Get pruning config from epoch
        let pruning_depth = self.epoch.pruning_depth();
        let pruning_finality = self.epoch.pruning_finality();
        (pruning_depth, pruning_finality)
    }

    fn get_genesis_hash(&self) -> HashValue {
        self.genesis_hash
    }

    fn dag(&self) -> BlockDAG {
        self.dag.clone()
    }

    fn get_header_by_hash(&self, block_id: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.0.get_block_header_by_hash(block_id)
    }

    fn validate_pruning_point(
        &self,
        ghostdata: &GhostdagData,
        pruning_point: HashValue,
    ) -> Result<()> {
        let chain_pruning_point = if pruning_point == HashValue::zero() {
            self.genesis_hash
        } else {
            pruning_point
        };
        let pruning_point_header = self
            .storage
            .0
            .get_block_header_by_hash(chain_pruning_point)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block header by hash when validating the block header {:?}",
                    chain_pruning_point
                )
            })?;
        let pruning_point_hash = self
            .get_hash_by_number(pruning_point_header.number())?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block hash by number when validating the block header {:?}",
                    pruning_point_header.number()
                )
            })?;
        if pruning_point_header.id() != pruning_point_hash {
            bail!(
                "Pruning point header id: {:?} not match with pruning point: {:?}",
                pruning_point_header.id(),
                pruning_point_hash
            );
        }

        let pruning_point_blue_score = self
            .dag()
            .storage
            .ghost_dag_store
            .get_blue_score(chain_pruning_point)?;
        let (pruning_depth, _pruning_finality) = self.get_pruning_config();
        if let Some(blue_score) = pruning_point_blue_score.checked_add(pruning_depth) {
            if ghostdata.blue_score < blue_score && chain_pruning_point != self.genesis_hash {
                bail!("Pruning point blue score: {:?} not match with ghostdag blue score: {:?} and pruning depth: {:?}", pruning_point_blue_score, ghostdata.blue_score, pruning_depth);
            }
        } else {
            bail!(
                "Overflow occurred when computing pruning_point_blue_score + pruning_depth: {:?} + {:?}",
                pruning_point_blue_score, pruning_depth
            );
        }
        Ok(())
    }

    fn check_parents_ready(&self, block_header: &BlockHeader) -> bool {
        block_header.parents_hash().iter().all(|parent| {
            self.has_dag_block(*parent).unwrap_or_else(|e| {
                warn!("check_parents_ready error: {:?}", e);
                false
            })
        })
    }
}

impl BlockChain {
    pub fn filter_events(&self, filter: Filter) -> Result<Vec<StcContractEventInfo>> {
        let (storage, _storage2) = &self.storage;
        let reverse = filter.reverse;
        let chain_header = self.current_header();
        let max_block_number = chain_header.number().min(filter.to_block);

        // quick return.
        if filter.from_block > max_block_number {
            return Ok(vec![]);
        }

        let (mut cur_block_number, tail) = if reverse {
            (max_block_number, filter.from_block)
        } else {
            (filter.from_block, max_block_number)
        };
        let mut event_with_infos = vec![];
        'outer: loop {
            let block = self.get_block_by_number(cur_block_number)?.ok_or_else(|| {
                anyhow::anyhow!(format!(
                    "cannot find block({}) on main chain(head: {})",
                    cur_block_number,
                    chain_header.id()
                ))
            })?;
            let block_id = block.id();
            let block_number = block.header().number();
            let mut txn_info_ids = storage.get_block_txn_info_ids(block_id)?;
            if reverse {
                txn_info_ids.reverse();
            }
            for id in txn_info_ids.iter() {
                let events = storage.get_contract_events_v2(*id)?.ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "cannot find events of txn with txn_info_id {} on main chain(header: {})",
                        id,
                        chain_header.id()
                    ))
                })?;
                let mut filtered_events = events
                    .into_iter()
                    .enumerate()
                    .filter(|(_idx, evt)| filter.matching(block_number, evt))
                    .peekable();
                if filtered_events.peek().is_none() {
                    continue;
                }

                let txn_info = storage
                    .get_transaction_info_by_info_id(*id, block_id)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(format!(
                            "cannot find txn info with txn_info_id {} on main chain(head: {})",
                            id,
                            chain_header.id()
                        ))
                    })?;

                let filtered_event_with_info =
                    filtered_events.map(|(idx, evt)| StcContractEventInfo {
                        block_hash: block_id,
                        block_number: block.header().number(),
                        transaction_hash: txn_info.transaction_hash(),
                        transaction_index: txn_info.transaction_index,
                        transaction_global_index: txn_info.transaction_global_index,
                        event_index: idx as u32,
                        event: evt,
                    });
                if reverse {
                    event_with_infos.extend(filtered_event_with_info.rev())
                } else {
                    event_with_infos.extend(filtered_event_with_info);
                }

                if let Some(limit) = filter.limit {
                    if event_with_infos.len() >= limit {
                        break 'outer;
                    }
                }
            }

            let should_break = match reverse {
                true => cur_block_number <= tail,
                false => cur_block_number >= tail,
            };

            if should_break {
                break 'outer;
            }

            if reverse {
                cur_block_number = cur_block_number.saturating_sub(1);
            } else {
                cur_block_number = cur_block_number.saturating_add(1);
            }
        }

        // remove additional events in respect limit filter.
        if let Some(limit) = filter.limit {
            event_with_infos.truncate(limit);
        }
        Ok(event_with_infos)
    }

    fn current_tips_hash(&self, pruning_point: HashValue) -> Result<Vec<HashValue>> {
        self.dag()
            .get_dag_state(pruning_point)
            .map(|state| state.tips)
    }

    fn renew_tips(
        &self,
        parent_header: &BlockHeader,
        tip_header: &BlockHeader,
        tips: Vec<HashValue>,
    ) -> Result<()> {
        if parent_header.pruning_point() == tip_header.pruning_point() {
            if tip_header.pruning_point() == HashValue::zero() {
                self.dag()
                    .save_dag_state(self.genesis_hash, DagState { tips })?;
            } else {
                self.dag()
                    .save_dag_state(tip_header.pruning_point(), DagState { tips })?;
            }
        } else {
            let new_tips = self.dag().pruning_point_manager().prune(
                &DagState { tips: tips.clone() },
                parent_header.pruning_point(),
                tip_header.pruning_point(),
            )?;
            info!("Pruning point changed, previous tips: {:?}, new tips: {:?}, previous pruning point: {:?}, current pruning point: {:?}",
                tips, new_tips, parent_header.pruning_point(), tip_header.pruning_point());
            self.dag()
                .save_dag_state(tip_header.pruning_point(), DagState { tips: new_tips })?;
        }
        Ok(())
    }

    pub fn has_dag_block(&self, header_id: HashValue) -> Result<bool> {
        let (storage, _) = &self.storage;
        let header = match storage.get_block_header_by_hash(header_id)? {
            Some(header) => header,
            None => return Ok(false),
        };

        if storage.get_block_info(header.id())?.is_none() {
            return Ok(false);
        }

        self.dag().has_block_connected(&header)
    }

    pub fn check_parents_ready(&self, header: &BlockHeader) -> bool {
        header.parents_hash().iter().all(|parent| {
            self.has_dag_block(*parent).unwrap_or_else(|e| {
                warn!("check_parents_ready error: {:?}", e);
                false
            })
        })
    }

    // legacy: pruning height should always start from genesis.
    pub fn get_pruning_height(&self) -> BlockNumber {
        let chain_id = self.status().head().chain_id();
        if chain_id.is_test() || chain_id.is_dev() {
            BlockNumber::MAX
        } else {
            0
        }
    }

    pub fn select_dag_state(&mut self, header: &BlockHeader) -> Result<Self> {
        let new_pruning_point = if header.pruning_point() == HashValue::zero() {
            self.genesis_hash
        } else {
            header.pruning_point()
        };
        let current_pruning_point = if self.status().head().pruning_point() == HashValue::zero() {
            self.genesis_hash
        } else {
            self.status().head().pruning_point()
        };

        let chain = if current_pruning_point == new_pruning_point
            || current_pruning_point == HashValue::zero()
        {
            let state = self.dag().get_dag_state(new_pruning_point)?;
            let block_id = self
                .dag()
                .ghost_dag_manager()
                .find_selected_parent(state.tips.into_iter())?;
            self.fork(block_id)?
        } else {
            // Handle pruning point change: select best header from both states
            let new_state = self.dag().get_dag_state(new_pruning_point)?;
            let current_state = self.dag().get_dag_state(current_pruning_point)?;

            let new_header = self
                .dag()
                .ghost_dag_manager()
                .find_selected_parent(new_state.tips.into_iter())?;
            let current_header = self
                .dag()
                .ghost_dag_manager()
                .find_selected_parent(current_state.tips.into_iter())?;

            let selected_header = self
                .dag()
                .ghost_dag_manager()
                .find_selected_parent([new_header, current_header].into_iter())?;

            self.fork(selected_header)?
        };

        Ok(chain)
    }
}

impl ChainWriter for BlockChain {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool {
        executed_block.block().header().parent_hash() == self.status.status.head().id()
    }

    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock> {
        info!(
            "Connect a DAG block, {:?}, number: {:?}",
            executed_block.block().id(),
            executed_block.block().header().number(),
        );

        let (storage, storage2) = &self.storage;
        let (new_tip_block, _) = (executed_block.block(), executed_block.block_info());
        let prev_vsp = self.status.status.head().id();

        // DAG logic: manage tips and select best parent
        let dag = self.dag().clone();
        let parent_header = storage
            .get_block_header_by_hash(new_tip_block.header().parent_hash())?
            .ok_or_else(|| {
                format_err!(
                    "DAG block parent should exist, block id: {:?}",
                    new_tip_block.header().parent_hash()
                )
            })?;

        // Get current tips based on pruning point
        let mut tips = if parent_header.pruning_point() == HashValue::zero() {
            self.current_tips_hash(self.genesis_hash)?
        } else {
            match self.current_tips_hash(parent_header.pruning_point()) {
                Ok(tips) => tips,
                Err(e) => match e.downcast::<StoreError>()? {
                    StoreError::KeyNotFound(_) => {
                        // Initialize tips for new pruning point
                        dag.save_dag_state(
                            parent_header.pruning_point(),
                            DagState {
                                tips: vec![parent_header.id()],
                            },
                        )?;
                        vec![parent_header.id()]
                    }
                    e => return Err(e.into()),
                },
            }
        };

        // Update tips: remove ancestors of new block
        let mut new_tips = vec![];
        for hash in tips {
            if !dag.check_ancestor_of(hash, new_tip_block.id())? {
                new_tips.push(hash);
            }
        }
        tips = new_tips;
        tips.push(new_tip_block.id());

        // Calculate ghostdata and select the best parent from tips
        let selected_block_hash = dag
            .ghost_dag_manager()
            .find_selected_parent(tips.iter().copied())?;

        // Get the selected block (might not be the executed_block)
        let (block, block_info) = if selected_block_hash == new_tip_block.id() {
            (new_tip_block.clone(), executed_block.block_info().clone())
        } else {
            let block = storage.get_block(selected_block_hash)?.ok_or_else(|| {
                format_err!(
                    "DAG block should exist, block id: {:?}",
                    selected_block_hash
                )
            })?;
            let block_info = storage
                .get_block_info(selected_block_hash)?
                .ok_or_else(|| {
                    format_err!(
                        "DAG block info should exist, block id: {:?}",
                        selected_block_hash
                    )
                })?;
            (block, block_info)
        };

        // Update accumulators (keep vm_state_accumulator for multi-VM)
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let vm_state_accumulator_info = block_info.get_vm_state_accumulator_info();

        self.txn_accumulator = info_2_accumulator(
            txn_accumulator_info.clone(),
            AccumulatorStoreType::Transaction,
            storage.as_ref(),
        );
        self.block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            storage.as_ref(),
        );
        self.vm_state_accumulator = info_2_accumulator(
            vm_state_accumulator_info.clone(),
            AccumulatorStoreType::VMState,
            storage.as_ref(),
        );

        // Get multi-state for dual VM support
        let multi_state = if selected_block_hash == new_tip_block.id() {
            // Use the executed block's multi_state
            executed_block.multi_state().clone()
        } else {
            // Get multi_state from storage for selected block
            storage.get_vm_multi_state(selected_block_hash)?
        };

        let (state_root1, state_root2) = (multi_state.state_root1(), multi_state.state_root2());

        // Update dual statedbs
        self.statedb = (
            ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root1)),
            ChainStateDB2::new(storage2.clone().into_super_arc(), Some(state_root2)),
        );

        // Update status with selected block
        self.status = ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
            multi_state,
        };
        // Update epoch from statedb after each block connection in DAG mode
        // This ensures epoch is always up-to-date during sync
        self.epoch = get_epoch_from_statedb(&self.statedb.1)?;

        // Save updated tips to DAG state
        self.renew_tips(&parent_header, new_tip_block.header(), tips)?;

        // Hint virtual selected parent only when VSP actually changes.
        if selected_block_hash != prev_vsp {
            if log_enabled!(Level::Debug) {
                let root_before = dag.storage.reachability_store.read().get_reindex_root();
                let hint_start = Instant::now();
                let hint_result = inquirer::hint_virtual_selected_parent(
                    dag.storage.reachability_store.write().deref_mut(),
                    selected_block_hash,
                );
                let root_after = dag.storage.reachability_store.read().get_reindex_root();
                debug!(
                    "block_process hint_virtual_selected_parent: vsp_changed prev={:?} new={:?} root_before={:?} root_after={:?} duration={:?} result={:?}",
                    prev_vsp,
                    selected_block_hash,
                    root_before,
                    root_after,
                    hint_start.elapsed(),
                    hint_result
                );
            } else {
                let _ = inquirer::hint_virtual_selected_parent(
                    dag.storage.reachability_store.write().deref_mut(),
                    selected_block_hash,
                );
            }
        }

        Ok(executed_block)
    }

    fn apply(&mut self, block: Block) -> Result<ExecutedBlock> {
        if !block.header().is_genesis() {
            let head_id = self.current_header().id();
            ensure!(
                head_id == block.parent_hash(),
                "apply expects parent == current head, head: {}, parent: {}",
                head_id,
                block.parent_hash()
            );
        }
        self.apply_with_verifier::<FullVerifier>(block)
    }

    fn chain_state(&self) -> &ChainStateDB {
        &self.statedb.0
    }

    fn chain_state2(&mut self) -> &ChainStateDB2 {
        &self.statedb.1
    }

    fn apply_for_sync(&mut self, block: Block) -> Result<ExecutedBlock> {
        self.apply_with_verifier::<FullVerifier>(block)
    }
}

use starcoin_vm2_executor::block_executor::BlockExecutedData as BlockExecutedData2;

/// Helper function to execute block transactions
fn execute_block_transactions(
    statedb: &ChainStateDB,
    statedb2: &ChainStateDB2,
    transactions: &[Transaction],
    transactions2: &[starcoin_vm2_types::transaction::Transaction],
    epoch: &Epoch,
    vm_metrics: Option<VMMetrics>,
) -> Result<(BlockExecutedData, BlockExecutedData2)> {
    // Execute VM1 transactions
    let executed_data = if !transactions.is_empty() {
        starcoin_executor::block_execute(
            statedb,
            transactions.to_vec(),
            epoch.block_gas_limit(),
            vm_metrics.clone(),
        )?
    } else {
        BlockExecutedData {
            state_root: statedb.state_root(),
            txn_infos: vec![],
            txn_events: vec![],
            txn_table_infos: Default::default(),
            write_sets: vec![],
        }
    };

    let vm1_gas_used = executed_data
        .txn_infos
        .iter()
        .fold(0u64, |acc, info| acc.saturating_add(info.gas_used()));

    // Execute VM2 transactions
    let executed_data2 = starcoin_vm2_chain::execute_transactions(
        statedb2,
        transactions2.to_vec(),
        epoch.block_gas_limit().saturating_sub(vm1_gas_used),
        vm_metrics,
    )?;

    Ok((executed_data, executed_data2))
}
impl BlockChain {
    fn execute_dag_block(&mut self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        info!("execute dag block:{:?}", verified_block.block.header().id());
        let block = verified_block.block;
        let selected_parent = block.parent_hash();
        let block_info_past = self
            .get_block_info(Some(selected_parent))?
            .expect("selected parent must executed");
        let header = block.header();
        let block_id = header.id();

        let selected_head = self
            .storage
            .0
            .get_block_by_hash(selected_parent)?
            .ok_or_else(|| {
                format_err!("Can not find selected block by hash {:?}", selected_parent)
            })?;

        // Prepare transactions for VM1
        let vm1_offline = header.number() >= vm1_offline_height(header.chain_id().id().into());
        let transactions = if !vm1_offline {
            let block_metadata =
                block_metadata::from(block.to_metadata(selected_head.header().gas_used(), 0));
            let mut txns = vec![Transaction::BlockMetadata(block_metadata)];
            txns.extend(
                block
                    .transactions()
                    .iter()
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            txns
        } else {
            vec![]
        };

        // Prepare transactions for VM2
        let red_blocks_count = verified_block.ghostdata.mergeset_reds.len() as u64;
        let block_metadata2 =
            block.to_metadata(selected_head.header().gas_used(), red_blocks_count);
        let transactions2 = build_block_transactions(block.transactions2(), Some(block_metadata2));

        // Fork statedb from selected parent's state root
        // Get MultiState from storage using block hash
        let parent_block_info =
            self.storage
                .0
                .get_block_info(selected_parent)?
                .ok_or_else(|| {
                    format_err!("Can not find block info for parent {:?}", selected_parent)
                })?;

        let statedb = &self.statedb.0;
        let statedb2 = &self.statedb.1;

        // Get epoch from forked statedb (read from VM2's statedb)
        let epoch = get_epoch_from_statedb(statedb2)?;

        // Check for cached state (StateDB + BlockExecutedData)
        // If cache hit, we can skip execution and apply_write_set entirely
        let state_cache = global_block_state_cache();
        let cached_state = state_cache.remove(header.txn_accumulator_root());

        // Record VM Execute timing for DAG path (only when pipeline timing is enabled)
        let dag_pipeline_enabled = global_collector().is_enabled();
        let dag_vm_exec_start = if dag_pipeline_enabled {
            let txn_count = transactions.len() + transactions2.len();
            global_collector().record_vm_exec_start(block_id, header.number(), txn_count);
            Some((std::time::Instant::now(), txn_count))
        } else {
            None
        };

        // Execute or use cached data
        let (executed_data, executed_data2, cached_statedb, cached_statedb2) =
            if let Some(cached) = cached_state {
                if cached.is_complete() {
                    info!(
                        "Using cached StateDB and executed data for block {:?}",
                        block_id
                    );
                    (
                        cached.executed_data.unwrap(),
                        cached.executed_data2.unwrap(),
                        cached.statedb,
                        cached.statedb2,
                    )
                } else {
                    // Partial cache - fall back to execution
                    let (data, data2) = execute_block_transactions(
                        statedb,
                        statedb2,
                        &transactions,
                        &transactions2,
                        &epoch,
                        self.vm_metrics.clone(),
                    )?;
                    (data, data2, None, None)
                }
            } else {
                // No cache - execute normally
                let (data, data2) = execute_block_transactions(
                    statedb,
                    statedb2,
                    &transactions,
                    &transactions2,
                    &epoch,
                    self.vm_metrics.clone(),
                )?;
                (data, data2, None, None)
            };

        // Record VM Execute end
        if let Some((start, txn_count)) = dag_vm_exec_start {
            global_collector().record_vm_exec_end(block_id);
            let dag_vm_exec_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            info!(
                "[Pipeline] DAG VM Execute block {} ({} txns) took {:.3} ms",
                block_id, txn_count, dag_vm_exec_duration_ms
            );
        }

        // If we used cached statedb, we need to flush the cached ones
        // Otherwise, we only need to flush self.statedb (executor already applied write_sets)
        let dag_commit_start = if dag_pipeline_enabled {
            let txn_count = transactions.len() + transactions2.len();
            global_collector().record_state_commit_start(block_id, header.number(), txn_count);
            Some((std::time::Instant::now(), txn_count))
        } else {
            None
        };

        if cached_statedb.is_some() && cached_statedb2.is_some() {
            // Flush cached statedbs
            let cached_db = cached_statedb.as_ref().unwrap();
            let cached_db2 = cached_statedb2.as_ref().unwrap();
            cached_db.flush()?;
            cached_db2.flush()?;
        } else {
            // Note: write_sets are already applied and committed by executor functions
            // (block_execute / execute_transactions), we only need to flush here
            statedb.flush()?;
            statedb2.flush()?;
        }

        if let Some((start, txn_count)) = dag_commit_start {
            global_collector().record_state_commit_end(block_id);
            let dag_commit_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            info!(
                "[Pipeline] DAG State Commit block {} ({} txns) took {:.3} ms",
                block_id, txn_count, dag_commit_duration_ms
            );
        }

        // Update state roots
        let (state_root, multi_state, vm_state_accumulator) = {
            let state_root1 = executed_data.state_root;
            let state_root2 = executed_data2.state_root;

            // Create local vm_state_accumulator from parent's accumulator info
            let parent_vm_state_accumulator_info =
                parent_block_info.get_vm_state_accumulator_info();
            let vm_state_accumulator = MerkleAccumulator::new_with_info(
                parent_vm_state_accumulator_info.clone(),
                self.storage
                    .0
                    .get_accumulator_store(AccumulatorStoreType::VMState),
            );

            vm_state_accumulator.append(&[state_root1, state_root2])?;
            let computed_state_root = vm_state_accumulator.root_hash();

            (
                computed_state_root,
                MultiState::new(state_root1, state_root2),
                vm_state_accumulator,
            )
        };

        // Verify state root
        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root {:?}, in header {:?} fail, multi_state {:?}",
            block_id,
            state_root,
            header.state_root(),
            multi_state
        );

        // Verify gas_used
        let block_gas_used = executed_data
            .txn_infos
            .iter()
            .fold(0u64, |acc, i| acc.saturating_add(i.gas_used()))
            .saturating_add(
                executed_data2
                    .txn_infos
                    .iter()
                    .fold(0u64, |acc, i| acc.saturating_add(i.gas_used())),
            );
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match, actual: {}, expected: {}",
            block_gas_used,
            header.gas_used()
        );

        // Verify transaction count
        let total_txn_num = executed_data.txn_infos.len() + executed_data2.txn_infos.len();
        // VM2 epilogue is executed internally (not included in transactions2).
        let expected_vm2_txn_num = expected_vm2_txn_count(&transactions2);
        let expected_txn_num = transactions.len() + expected_vm2_txn_num;
        verify_block!(
            VerifyBlockField::State,
            total_txn_num == expected_txn_num,
            "invalid txn num in the block, actual: {}, expected: {}",
            total_txn_num,
            expected_txn_num
        );

        // Calculate total difficulty
        let mut total_difficulty = header
            .difficulty()
            .checked_add(block_info_past.total_difficulty)
            .unwrap_or_else(|| {
                panic!(
                    "header difficulty overflowed: current {} previous total_difficulty {}",
                    header.difficulty(),
                    block_info_past.total_difficulty
                )
            });

        for blue in block.body.uncles.as_ref().unwrap_or(&vec![]) {
            total_difficulty = total_difficulty
                .checked_add(blue.difficulty())
                .unwrap_or_else(|| {
                    panic!(
                        "total difficulty overflowed total_difficulty {} blue block difficulty {}",
                        total_difficulty,
                        blue.difficulty(),
                    )
                });
        }

        // Create local txn_accumulator from parent's accumulator info
        let parent_txn_accumulator_info = parent_block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new_with_info(
            parent_txn_accumulator_info.clone(),
            self.storage
                .0
                .get_accumulator_store(AccumulatorStoreType::Transaction),
        );

        let transaction_global_index = txn_accumulator.num_leaves();

        // Update txn accumulator with both VM1 and VM2 transactions
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> = executed_data
                .txn_infos
                .iter()
                .map(|info| info.id())
                .collect();
            let included_txn_info_hashes2 = executed_data2
                .txn_infos
                .iter()
                .map(|info| info.id())
                .collect::<Vec<_>>();

            if vm1_offline {
                txn_accumulator.append(&included_txn_info_hashes2)?;
            } else {
                // append the block meta txns of vm2 firstly
                txn_accumulator.append(
                    included_txn_info_hashes2
                        .get(0..1)
                        .ok_or_else(|| format_err!("block meta txns of vm2 is None"))?,
                )?;

                if !included_txn_info_hashes.is_empty() {
                    txn_accumulator.append(&included_txn_info_hashes)?;
                }

                if !included_txn_info_hashes2.is_empty() {
                    txn_accumulator
                        .append(included_txn_info_hashes2.get(1..).unwrap_or_default())?;
                }
            }

            txn_accumulator.root_hash()
        };

        verify_block!(
            VerifyBlockField::Transaction,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch in execute dag block , block_id: {}, expected accumulator root: {:?}, header txn_accumulator_root: {:?}",
            block_id,
            executed_accumulator_root,
            header.txn_accumulator_root(),
        );
        let parent_block_accumulator_info = parent_block_info.get_block_accumulator_info();
        let block_accumulator = MerkleAccumulator::new_with_info(
            parent_block_accumulator_info.clone(),
            self.storage
                .0
                .get_accumulator_store(AccumulatorStoreType::Block),
        );

        // Append block to accumulator and flush
        block_accumulator.append(&[block_id])?;

        // Flush accumulators
        txn_accumulator.flush()?;
        vm_state_accumulator.flush()?;
        block_accumulator.flush()?;

        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator.get_info(),
            block_accumulator.get_info(),
            vm_state_accumulator.get_info(),
        );

        // Save block info and commit
        let (storage, _storage2) = &self.storage;
        storage.save_block_info(block_info.clone())?;
        storage.commit_block(block.clone())?;

        // DAG specific saves
        // Update k parameter for DAG
        // TODO: The k should be reload or update only on new epoch.
        self.dag()
            .ghost_dag_manager()
            .update_k(epoch.max_uncles_per_block().try_into().unwrap());

        // Commit the DAG block
        self.dag()
            .commit_trusted_block(header.to_owned(), Arc::new(verified_block.ghostdata))?;

        // Save events, table infos and transaction infos
        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let vm2_txn_infos = executed_data2.txn_infos;
        let vm2_txn_events = executed_data2.txn_events;

        // Merge table infos from both VMs
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .chain(
                executed_data2
                    .txn_table_infos
                    .into_iter()
                    .map(|(k, v)| (k.into(), v.into())),
            )
            .collect::<Vec<_>>();

        // Save events for VM1
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(
            txn_events
                .into_iter()
                .map(|events| events.into_iter().map(Into::into).collect::<Vec<_>>()),
        ) {
            storage.save_contract_events_v2(*info_id, events)?;
        }

        // Save events for VM2
        let vm2_txn_info_ids: Vec<_> = vm2_txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in vm2_txn_info_ids.iter().zip(
            vm2_txn_events
                .into_iter()
                .map(|events| events.into_iter().map(Into::into).collect::<Vec<_>>()),
        ) {
            storage.save_contract_events_v2(*info_id, events)?;
        }
        let transactions2_for_storage =
            append_vm2_epilogue_for_storage(transactions2, &vm2_txn_infos)?;

        // Save transaction infos
        if txn_infos.is_empty() {
            storage.save_transaction_infos(
                vm2_txn_infos
                    .into_iter()
                    .map(Into::into)
                    .enumerate()
                    .map(|(transaction_index, info)| {
                        StcRichTransactionInfo::new(
                            block_id,
                            header.number(),
                            info,
                            transaction_index as u32,
                            transaction_global_index
                                .checked_add(transaction_index as u64)
                                .expect("transaction_global_index overflow."),
                        )
                    })
                    .collect(),
            )?;
        } else {
            storage.save_transaction_infos(
                std::iter::once::<StcTransactionInfo>(
                    vm2_txn_infos
                        .first()
                        .expect("vm2 block meta transaction is none")
                        .clone()
                        .into(),
                )
                .chain(txn_infos.into_iter().map(Into::into))
                .chain(vm2_txn_infos.into_iter().skip(1).map(Into::into))
                .enumerate()
                .map(|(transaction_index, info)| {
                    StcRichTransactionInfo::new(
                        block_id,
                        header.number(),
                        info,
                        transaction_index as u32,
                        transaction_global_index
                            .checked_add(transaction_index as u64)
                            .expect("transaction_global_index overflow."),
                    )
                })
                .collect(),
            )?;
        }
        // Save transactions
        let all_transactions: Vec<StcTransaction> = transactions
            .into_iter()
            .map(Into::into)
            .chain(transactions2_for_storage.into_iter().map(Into::into))
            .collect();
        let txn_id_vec = all_transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        storage.save_transaction_batch(all_transactions)?;

        // Save block's transaction ids and info ids
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        let ordered_ids = if txn_info_ids.is_empty() {
            vm2_txn_info_ids
        } else {
            std::iter::once(vm2_txn_info_ids[0])
                .chain(txn_info_ids)
                .chain(vm2_txn_info_ids.into_iter().skip(1))
                .collect()
        };
        storage.save_block_txn_info_ids(block_id, ordered_ids)?;

        // Save table infos
        storage.save_table_infos(txn_table_infos)?;

        Ok(ExecutedBlock::new(block, block_info, multi_state))
    }
}

pub(crate) fn info_2_accumulator(
    accumulator_info: AccumulatorInfo,
    store_type: AccumulatorStoreType,
    node_store: &dyn Store,
) -> MerkleAccumulator {
    MerkleAccumulator::new_with_info(
        accumulator_info,
        node_store.get_accumulator_store(store_type),
    )
}

fn append_vm2_epilogue_for_storage(
    transactions2: Vec<Transaction2>,
    vm2_txn_infos: &[TransactionInfo2],
) -> Result<Vec<Transaction2>> {
    if vm2_txn_infos.len() == transactions2.len() {
        return Ok(transactions2);
    }
    ensure!(
        vm2_txn_infos.len() == transactions2.len() + 1,
        "vm2 txn info length mismatch, txn_infos: {}, txns: {}",
        vm2_txn_infos.len(),
        transactions2.len()
    );
    let block_meta = match transactions2.first() {
        Some(Transaction2::BlockMetadata(meta)) => meta.clone(),
        _ => bail!("vm2 transactions missing block metadata"),
    };
    let mut total_fee: u128 = 0;
    for (txn, info) in transactions2.iter().zip(vm2_txn_infos.iter()) {
        if let Transaction2::UserTransaction(user_txn) = txn {
            total_fee = total_fee.saturating_add(
                u128::from(info.gas_used()) * u128::from(user_txn.gas_unit_price()),
            );
        }
    }
    let total_fee = u64::try_from(total_fee).map_err(|_| format_err!("total fee overflow"))?;
    let mut txns = transactions2;
    txns.push(Transaction2::BlockEpilogue(block_meta, total_fee));
    Ok(txns)
}
