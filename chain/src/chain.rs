// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verifier::{BlockVerifier, DagBasicVerifier, DagVerifier, FullVerifier};
use anyhow::{anyhow, bail, ensure, format_err, Ok, Result};
use bcs_ext::BCSCodec;
use sp_utils::stop_watch::{watch, CHAIN_WATCH_NAME};
use starcoin_accumulator::inmemory::InMemoryAccumulator;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_chain_api::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, EventWithProof, ExcludedTxns,
    ExecutedBlock, MintedUncleNumber, TransactionInfoWithProof, VerifiedBlock, VerifyBlockField,
};
use starcoin_consensus::Consensus;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::consenses_state::DagState;
use starcoin_dag::consensusdb::prelude::StoreError;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{AccountStateReader, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_time_service::TimeService;
use starcoin_types::block::{BlockIdAndNumber, DagHeaderType};
use starcoin_types::contract_event::ContractEventInfo;
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    contract_event::ContractEvent,
    error::BlockExecutorError,
    transaction::{SignedUserTransaction, Transaction},
    U256,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_config::FlexiDagConfig;
use starcoin_vm_types::on_chain_resource::Epoch;
use starcoin_vm_types::state_view::StateReaderExt;
use std::cmp::{min, Ordering};
use std::collections::HashSet;
use std::iter::Extend;
use std::option::Option::{None, Some};
use std::{collections::HashMap, sync::Arc};

pub struct ChainStatusWithBlock {
    pub status: ChainStatus,
    pub head: Block,
}

pub struct BlockChain {
    genesis_hash: HashValue,
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    status: ChainStatusWithBlock,
    statedb: ChainStateDB,
    storage: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    uncles: HashMap<HashValue, MintedUncleNumber>,
    epoch: Epoch,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
}

impl BlockChain {
    pub fn new(
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> Result<Self> {
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", head_block_hash))?;
        Self::new_with_uncles(time_service, head, None, storage, vm_metrics, dag)
    }

    fn new_with_uncles(
        time_service: Arc<dyn TimeService>,
        head_block: Block,
        uncles: Option<HashMap<HashValue, MintedUncleNumber>>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        mut dag: BlockDAG,
    ) -> Result<Self> {
        let block_info = storage
            .get_block_info(head_block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", head_block.id()))?;
        debug!("Init chain with block_info: {:?}", block_info);
        let state_root = head_block.header().state_root();
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let chain_state = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root));
        let epoch = get_epoch_from_statedb(&chain_state)?;
        let genesis = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("Can not find genesis hash in storage."))?;
        watch(CHAIN_WATCH_NAME, "n1253");
        let mut chain = Self {
            genesis_hash: genesis,
            time_service,
            txn_accumulator: info_2_accumulator(
                txn_accumulator_info.clone(),
                AccumulatorStoreType::Transaction,
                storage.as_ref(),
            ),
            block_accumulator: info_2_accumulator(
                block_accumulator_info.clone(),
                AccumulatorStoreType::Block,
                storage.as_ref(),
            ),
            status: ChainStatusWithBlock {
                status: ChainStatus::new(head_block.header.clone(), block_info),
                head: head_block,
            },
            statedb: chain_state,
            storage: storage.clone(),
            uncles: HashMap::new(),
            epoch,
            vm_metrics,
            dag: dag.clone(),
        };
        let current_header = chain.current_header();
        if chain.check_dag_type(&current_header)? != DagHeaderType::Single {
            dag.set_reindex_root(chain.get_block_dag_origin()?)?;
        }
        watch(CHAIN_WATCH_NAME, "n1251");
        match uncles {
            Some(data) => chain.uncles = data,
            None => chain.update_uncle_cache()?,
        }
        watch(CHAIN_WATCH_NAME, "n1252");
        Ok(chain)
    }

    pub fn new_with_genesis(
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        genesis_epoch: Epoch,
        genesis_block: Block,
        dag: BlockDAG,
    ) -> Result<Self> {
        debug_assert!(genesis_block.header().is_genesis());
        let txn_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let block_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        );
        let statedb = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let executed_block = Self::execute_block_and_save(
            storage.as_ref(),
            statedb,
            txn_accumulator,
            block_accumulator,
            &genesis_epoch,
            None,
            genesis_block,
            None,
        )?;
        Self::new(time_service, executed_block.block.id(), storage, None, dag)
    }

    pub fn current_epoch_uncles_size(&self) -> u64 {
        self.uncles.len() as u64
    }

    pub fn current_block_accumulator_info(&self) -> AccumulatorInfo {
        self.block_accumulator.get_info()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        self.epoch.strategy()
    }
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    pub fn dag(&self) -> BlockDAG {
        self.dag.clone()
    }

    //TODO lazy init uncles cache.
    fn update_uncle_cache(&mut self) -> Result<()> {
        self.uncles = self.epoch_uncles()?;
        Ok(())
    }

    fn epoch_uncles(&self) -> Result<HashMap<HashValue, MintedUncleNumber>> {
        let epoch = &self.epoch;
        let mut uncles: HashMap<HashValue, MintedUncleNumber> = HashMap::new();
        let head_block = self.head_block().block;
        let head_number = head_block.header().number();
        if head_number < epoch.start_block_number() || head_number >= epoch.end_block_number() {
            return Err(format_err!(
                "head block {} not in current epoch: {:?}.",
                head_number,
                epoch
            ));
        }
        for block_number in epoch.start_block_number()..epoch.end_block_number() {
            let block_uncles = if block_number == head_number {
                head_block.uncle_ids()
            } else {
                self.get_block_by_number(block_number)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find block by number {}, head block number: {}",
                            block_number,
                            head_number
                        )
                    })?
                    .uncle_ids()
            };
            block_uncles.into_iter().for_each(|uncle_id| {
                uncles.insert(uncle_id, block_number);
            });
            if block_number == head_number {
                break;
            }
        }

        Ok(uncles)
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
        tips: Option<Vec<HashValue>>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        //FIXME create block template by parent may be use invalid chain state, such as epoch.
        //So the right way should be creating a BlockChain by parent_hash, then create block template.
        //the timestamp should be an argument, if want to mock an early block.
        let previous_header = match parent_hash {
            Some(hash) => self
                .get_header(hash)?
                .ok_or_else(|| format_err!("Can find block header by {:?}", hash))?,
            None => self.current_header(),
        };

        self.create_block_template_by_header(
            author,
            previous_header,
            user_txns,
            uncles,
            block_gas_limit,
            tips,
        )
    }

    pub fn create_block_template_by_header(
        &self,
        author: AccountAddress,
        previous_header: BlockHeader,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
        tips: Option<Vec<HashValue>>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let current_number = previous_header.number().saturating_add(1);
        let epoch = self.epoch();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);
        let (_, tips_hash) = if current_number <= self.dag_fork_height()?.unwrap_or(u64::MAX) {
            (None, None)
        } else if tips.is_some() {
            (Some(self.get_block_dag_genesis(&previous_header)?), tips)
        } else {
            let result = self.current_tips_hash(&previous_header)?.expect("the block number is larger than the dag fork number but the state data doese not exis");
            (Some(result.0), Some(result.1))
        };
        let strategy = epoch.strategy();
        let difficulty = strategy.calculate_next_difficulty(self)?;
        let (uncles, blue_blocks) = {
            match &tips_hash {
                None => (uncles, None),
                Some(tips) => {
                    let mut blues = self.dag.ghostdata(tips)?.mergeset_blues.to_vec();
                    info!(
                        "create block template with tips:{:?}, ghostdata blues:{:?}",
                        &tips_hash, blues
                    );
                    let mut blue_blocks = vec![];
                    let _selected_parent = blues.remove(0);
                    for blue in &blues {
                        let block = self
                            .storage
                            .get_block_by_hash(blue.to_owned())?
                            .expect("Block should exist");
                        blue_blocks.push(block);
                    }
                    (
                        blue_blocks
                            .as_slice()
                            .iter()
                            .map(|b| b.header.clone())
                            .collect(),
                        Some(blue_blocks),
                    )
                }
            }
        };
        info!("Blue blocks:{:?}", blue_blocks);
        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            self.time_service.now_millis(),
            uncles,
            difficulty,
            strategy,
            None,
            Some(tips_hash.unwrap_or_default()),
            blue_blocks,
        )?;
        let excluded_txns = opened_block.push_txns(user_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns))
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

    // filter block by check exist
    fn exist_block_filter(&self, block: Option<Block>) -> Result<Option<Block>> {
        Ok(match block {
            Some(block) => {
                if self.check_exist_block(block.id(), block.header().number())? {
                    Some(block)
                } else {
                    None
                }
            }
            None => None,
        })
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
        self.storage.clone()
    }

    pub fn can_be_uncle(&self, block_header: &BlockHeader) -> Result<bool> {
        FullVerifier::can_be_uncle(self, block_header)
    }

    pub fn verify_with_verifier<V>(&mut self, block: Block) -> Result<VerifiedBlock>
    where
        V: BlockVerifier,
    {
        if self.check_dag_type(block.header())? == DagHeaderType::Normal {
            let selected_chain = Self::new(
                self.time_service.clone(),
                block.parent_hash(),
                self.storage.clone(),
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

    //TODO remove this function.
    pub fn update_chain_head(&mut self, block: Block) -> Result<ExecutedBlock> {
        let block_info = self
            .storage
            .get_block_info(block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", block.id()))?;
        self.connect(ExecutedBlock { block, block_info })
    }

    fn check_parents_coherent(&self, header: &BlockHeader) -> Result<HashValue> {
        if self.check_dag_type(header)? != DagHeaderType::Normal {
            bail!("Block is not a dag block.");
        }

        let results = header.parents_hash().ok_or_else(|| anyhow!("dag block has no parents."))?.into_iter().map(|parent_hash| {
            let header = self.storage.get_block_header_by_hash(parent_hash)?.ok_or_else(|| anyhow!("failed to find the block header in the block storage when checking the dag block exists, block hash: {:?}, number: {:?}", header.id(), header.number()))?;
            let dag_genesis_hash = self.get_block_dag_genesis(&header)?;
            let dag_genesis = self.storage.get_block_header_by_hash(dag_genesis_hash)?.ok_or_else(|| anyhow!("failed to find the block header in the block storage when checking the dag block exists, block hash: {:?}, number: {:?}", header.id(), header.number()))?;
            Ok(dag_genesis.parent_hash())
        }).collect::<Result<HashSet<_>>>()?;

        if results.len() == 1 {
            Ok(results
                .into_iter()
                .next()
                .expect("the len of the results is larger than 1 but no the first elemen!"))
        } else {
            bail!("dag block: {:?}, number: {:?} has multiple parents whose dags are not the same one! Their dag genesis are: {:?}", header.id(), header.number(), results);
        }
    }

    fn execute_dag_block(&mut self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        let origin = self.check_parents_coherent(verified_block.0.header())?;
        info!("execute dag block:{:?}", verified_block.0);
        let block = verified_block.0;
        let selected_parent = block.parent_hash();
        let blues = block.uncle_ids();
        let block_info_past = self
            .storage
            .get_block_info(selected_parent)?
            .expect("selected parent must executed");
        let header = block.header();
        let block_id = header.id();
        //TODO::FIXEME
        let selected_head = self
            .storage
            .get_block_by_hash(selected_parent)?
            .ok_or_else(|| {
                format_err!("Can not find selected block by hash {:?}", selected_parent)
            })?;
        let block_metadata = block.to_metadata(selected_head.header().gas_used());
        let mut transactions = vec![Transaction::BlockMetadata(block_metadata)];
        let mut total_difficulty = header.difficulty() + block_info_past.total_difficulty;

        for blue in blues {
            let blue_block = self
                .storage
                .get_block_by_hash(blue)?
                .expect("block blue need exist");
            transactions.extend(
                blue_block
                    .transactions()
                    .iter()
                    .skip(1)
                    .cloned()
                    .map(Transaction::UserTransaction),
            );
            total_difficulty += blue_block.header.difficulty();
        }
        transactions.extend(
            block
                .transactions()
                .iter()
                .cloned()
                .map(Transaction::UserTransaction),
        );
        watch(CHAIN_WATCH_NAME, "n21");
        let statedb = self.statedb.fork_at(selected_head.header.state_root());
        let epoch = get_epoch_from_statedb(&statedb)?;
        let executed_data = starcoin_executor::block_execute(
            &statedb,
            transactions.clone(),
            epoch.block_gas_limit(), //TODO: Fix me
            self.vm_metrics.clone(),
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
        let txn_accumulator = info_2_accumulator(
            block_info_past.txn_accumulator_info,
            AccumulatorStoreType::Transaction,
            self.storage.as_ref(),
        );
        let block_accumulator = info_2_accumulator(
            block_info_past.block_accumulator_info,
            AccumulatorStoreType::Block,
            self.storage.as_ref(),
        );
        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        watch(CHAIN_WATCH_NAME, "n23");
        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        watch(CHAIN_WATCH_NAME, "n24");
        txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
        );

        watch(CHAIN_WATCH_NAME, "n25");

        // save block's transaction relationship and save transaction

        let block_id = block.id();
        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .collect::<Vec<_>>();

        debug_assert!(
            txn_events.len() == txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            self.storage.save_contract_events(*info_id, events)?;
        }

        self.storage.save_transaction_infos(
            txn_infos
                .into_iter()
                .enumerate()
                .map(|(transaction_index, info)| {
                    RichTransactionInfo::new(
                        block_id,
                        block.header().number(),
                        info,
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
        self.storage.save_transaction_batch(transactions)?;

        // save block's transactions
        self.storage
            .save_block_transaction_ids(block_id, txn_id_vec)?;
        self.storage
            .save_block_txn_info_ids(block_id, txn_info_ids)?;
        self.storage.commit_block(block.clone())?;
        self.storage.save_block_info(block_info.clone())?;

        self.storage.save_table_infos(txn_table_infos)?;
        let result = self.dag.commit(header.to_owned(), origin);
        match result {
            anyhow::Result::Ok(_) => (),
            Err(e) => {
                if let Some(StoreError::KeyAlreadyExists(_)) = e.downcast_ref::<StoreError>() {
                    info!("dag block already exist, ignore");
                } else {
                    return Err(e);
                }
            }
        }
        watch(CHAIN_WATCH_NAME, "n26");
        Ok(ExecutedBlock { block, block_info })
    }

    //TODO consider move this logic to BlockExecutor
    fn execute_block_and_save(
        storage: &dyn Store,
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
                    let block_metadata = block.to_metadata(parent.head().gas_used());
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
            "verify legacy block:{:?} state_root fail",
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

        let transaction_global_index = txn_accumulator.num_leaves();

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        watch(CHAIN_WATCH_NAME, "n23");
        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        watch(CHAIN_WATCH_NAME, "n24");
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
        let block_info = BlockInfo::new(
            block_id,
            total_difficulty,
            txn_accumulator_info,
            block_accumulator_info,
        );

        watch(CHAIN_WATCH_NAME, "n25");

        // save block's transaction relationship and save transaction

        let block_id = block.id();
        let txn_infos = executed_data.txn_infos;
        let txn_events = executed_data.txn_events;
        let txn_table_infos = executed_data
            .txn_table_infos
            .into_iter()
            .collect::<Vec<_>>();

        debug_assert!(
            txn_events.len() == txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            storage.save_contract_events(*info_id, events)?;
        }

        storage.save_transaction_infos(
            txn_infos
                .into_iter()
                .enumerate()
                .map(|(transaction_index, info)| {
                    RichTransactionInfo::new(
                        block_id,
                        block.header().number(),
                        info,
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
        storage.save_transaction_batch(transactions)?;

        // save block's transactions
        storage.save_block_transaction_ids(block_id, txn_id_vec)?;
        storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
        storage.commit_block(block.clone())?;
        storage.save_block_info(block_info.clone())?;
        storage.save_table_infos(txn_table_infos)?;
        watch(CHAIN_WATCH_NAME, "n26");
        Ok(ExecutedBlock { block, block_info })
    }

    pub fn get_txn_accumulator(&self) -> &MerkleAccumulator {
        &self.txn_accumulator
    }

    pub fn get_block_accumulator(&self) -> &MerkleAccumulator {
        &self.block_accumulator
    }

    pub fn init_dag_with_genesis(&mut self, genesis: BlockHeader) -> Result<()> {
        if self.check_dag_type(&genesis)? == DagHeaderType::Genesis {
            let dag_genesis_id = genesis.id();
            info!(
                "Init dag genesis {dag_genesis_id} height {}",
                genesis.number()
            );
            self.dag.init_with_genesis(genesis)?;
        }
        Ok(())
    }

    pub fn get_block_dag_genesis(&self, header: &BlockHeader) -> Result<HashValue> {
        let dag_fork_height = self
            .dag_fork_height()?
            .ok_or_else(|| anyhow!("unset dag fork height"))?;
        let block_info = self
            .storage
            .get_block_info(header.id())?
            .ok_or_else(|| anyhow!("Cannot find block info by hash {:?}", header.id()))?;
        let block_accumulator = MerkleAccumulator::new_with_info(
            block_info.get_block_accumulator_info().clone(),
            self.storage
                .get_accumulator_store(AccumulatorStoreType::Block),
        );
        let dag_genesis = block_accumulator
            .get_leaf(dag_fork_height)?
            .ok_or_else(|| anyhow!("failed to get the dag genesis"))?;

        Ok(dag_genesis)
    }

    pub fn get_block_dag_origin(&self) -> Result<HashValue> {
        let dag_genesis = self.get_block_dag_genesis(&self.current_header())?;
        let block_header = self
            .storage
            .get_block_header_by_hash(dag_genesis)?
            .ok_or_else(|| anyhow!("Cannot find block by hash {:?}", dag_genesis))?;

        Ok(HashValue::sha3_256_of(
            &[block_header.parent_hash(), block_header.id()].encode()?,
        ))
    }

    pub fn get_dag_state_by_block(&self, header: &BlockHeader) -> Result<(HashValue, DagState)> {
        let dag_genesis = self.get_block_dag_genesis(header)?;
        Ok((dag_genesis, self.dag.get_dag_state(dag_genesis)?))
    }

    pub fn check_dag_type(&self, header: &BlockHeader) -> Result<DagHeaderType> {
        let dag_height = self.dag_fork_height()?.unwrap_or(u64::MAX);
        if header.is_genesis() {
            return Ok(DagHeaderType::Single);
        }
        let no_parents = header.parents_hash().unwrap_or_default().is_empty();

        let result = match (no_parents, header.number().cmp(&dag_height)) {
            (true, Ordering::Greater) => {
                Err(anyhow!("block header with suitable height but no parents"))
            }
            (false, Ordering::Greater) => Ok(DagHeaderType::Normal),

            (true, Ordering::Equal) => Ok(DagHeaderType::Genesis),
            (false, Ordering::Equal) => Err(anyhow!(
                "block header with dag genesis height but having parents"
            )),

            (true, Ordering::Less) => Ok(DagHeaderType::Single),
            (false, Ordering::Less) => Err(anyhow!(
                "block header with smaller height but having parents"
            )),
        };
        result
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
        ExecutedBlock::new(self.status.head.clone(), self.status.status.info.clone())
    }

    fn current_header(&self) -> BlockHeader {
        self.status.status.head().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage
            .get_block_header_by_hash(hash)
            .and_then(|block_header| self.exist_header_filter(block_header))
    }

    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => self.storage.get_block_header_by_hash(block_id),
            })
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.get_hash_by_number(number)
            .and_then(|block_id| match block_id {
                None => Ok(None),
                Some(block_id) => self.storage.get_block_by_hash(block_id),
            })
    }

    fn get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>> {
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
        let block_opts = self.storage.get_blocks(ids)?;
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
        self.storage
            .get_block_by_hash(hash)
            .and_then(|block| self.exist_block_filter(block))
    }

    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>> {
        self.block_accumulator.get_leaf(number)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        //TODO check txn should exist on current chain.
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<RichTransactionInfo>> {
        let txn_info_ids = self
            .storage
            .get_transaction_info_ids_by_txn_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            let txn_info = self.storage.get_transaction_info(txn_info_id)?;
            if let Some(txn_info) = txn_info {
                if self.exist_block(txn_info.block_id())? {
                    return Ok(Some(txn_info));
                }
            }
        }
        Ok(None)
    }

    fn get_transaction_info_by_global_index(
        &self,
        transaction_global_index: u64,
    ) -> Result<Option<RichTransactionInfo>> {
        match self.txn_accumulator.get_leaf(transaction_global_index)? {
            None => Ok(None),
            Some(hash) => self.storage.get_transaction_info(hash),
        }
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.statedb
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        match block_id {
            Some(block_id) => self.storage.get_block_info(block_id),
            None => Ok(Some(self.status.status.info().clone())),
        }
    }

    fn get_total_difficulty(&self) -> Result<U256> {
        Ok(self.status.status.total_difficulty())
    }

    fn exist_block(&self, block_id: HashValue) -> Result<bool> {
        if let Some(header) = self.storage.get_block_header_by_hash(block_id)? {
            return self.check_exist_block(block_id, header.number());
        }
        Ok(false)
    }

    fn epoch(&self) -> &Epoch {
        &self.epoch
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
        ensure!(
            self.exist_block(block_id)?,
            "Block with id{} do not exists in current chain.",
            block_id
        );
        let head = self
            .storage
            .get_block_by_hash(block_id)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", block_id))?;
        // if fork block_id is at same epoch, try to reuse uncles cache.
        let uncles = if head.header().number() >= self.epoch.start_block_number() {
            Some(
                self.uncles
                    .iter()
                    .filter(|(_uncle_id, uncle_number)| **uncle_number <= head.header().number())
                    .map(|(uncle_id, uncle_number)| (*uncle_id, *uncle_number))
                    .collect::<HashMap<HashValue, MintedUncleNumber>>(),
            )
        } else {
            None
        };

        BlockChain::new_with_uncles(
            self.time_service.clone(),
            head,
            uncles,
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
            //TODO: check missing blocks need to be clean
        )
    }

    fn epoch_uncles(&self) -> &HashMap<HashValue, MintedUncleNumber> {
        &self.uncles
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
        if self.check_dag_type(block.header())? == DagHeaderType::Normal {
            DagBasicVerifier::verify_header(self, block.header())?;
            Ok(VerifiedBlock(block))
        } else {
            FullVerifier::verify_block(self, block)
        }
    }

    fn execute(&mut self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        let header = verified_block.0.header().clone();
        if self.check_dag_type(&header)? != DagHeaderType::Normal {
            let executed = Self::execute_block_and_save(
                self.storage.as_ref(),
                self.statedb.fork(),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                &self.epoch,
                Some(self.status.status.clone()),
                verified_block.0,
                self.vm_metrics.clone(),
            )?;
            self.init_dag_with_genesis(header)?;
            Ok(executed)
        } else {
            self.execute_dag_block(verified_block)
        }
    }

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>> {
        let chain_header = self.current_header();
        let hashes = self
            .txn_accumulator
            .get_leaves(start_index, reverse, max_size)?;
        let mut infos = vec![];
        let txn_infos = self.storage.get_transaction_infos(hashes.clone())?;
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
        self.storage.get_contract_events(txn_info_id)
    }

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        let block_info = match self.get_block_info(Some(block_id))? {
            Some(block_info) => block_info,
            None => return Ok(None),
        };
        let accumulator = self
            .txn_accumulator
            .fork(Some(block_info.txn_accumulator_info));
        let txn_proof = match accumulator.get_proof(transaction_global_index)? {
            Some(proof) => proof,
            None => return Ok(None),
        };

        // If we can get proof by leaf_index, the leaf and transaction info should exist.
        let txn_info_hash = self
            .txn_accumulator
            .get_leaf(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find txn info hash by index {}",
                    transaction_global_index
                )
            })?;
        let transaction_info = self
            .storage
            .get_transaction_info(txn_info_hash)?
            .ok_or_else(|| format_err!("Can not find txn info by hash:{}", txn_info_hash))?;

        let event_proof = if let Some(event_index) = event_index {
            let events = self
                .storage
                .get_contract_events(txn_info_hash)?
                .unwrap_or_default();
            let event = events.get(event_index as usize).cloned().ok_or_else(|| {
                format_err!("event index out of range, events len:{}", events.len())
            })?;
            let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();

            let event_proof =
                InMemoryAccumulator::get_proof_from_leaves(event_hashes.as_slice(), event_index)?;
            Some(EventWithProof {
                event,
                proof: event_proof,
            })
        } else {
            None
        };
        let state_proof = if let Some(access_path) = access_path {
            let statedb = self
                .statedb
                .fork_at(transaction_info.txn_info().state_root_hash());
            Some(statedb.get_with_proof(&access_path)?)
        } else {
            None
        };
        Ok(Some(TransactionInfoWithProof {
            transaction_info,
            proof: txn_proof,
            event_proof,
            state_proof,
        }))
    }

    fn current_tips_hash(
        &self,
        header: &BlockHeader,
    ) -> Result<Option<(HashValue, Vec<HashValue>)>> {
        let (dag_genesis, dag_state) = self.get_dag_state_by_block(header)?;
        Ok(Some((dag_genesis, dag_state.tips)))
    }

    fn has_dag_block(&self, header_id: HashValue) -> Result<bool> {
        let dag_fork_height = if let Some(height) = self.dag_fork_height()? {
            height
        } else {
            return Ok(false);
        };

        let header = match self.storage.get_block_header_by_hash(header_id)? {
            Some(header) => header,
            None => return Ok(false),
        };

        let block_info = match self.storage.get_block_info(header.id())? {
            Some(block_info) => block_info,
            None => return Ok(false),
        };
        let block_accumulator = MerkleAccumulator::new_with_info(
            block_info.get_block_accumulator_info().clone(),
            self.storage
                .get_accumulator_store(AccumulatorStoreType::Block),
        );
        let dag_genesis = match block_accumulator.get_leaf(dag_fork_height)? {
            Some(dag_genesis) => dag_genesis,
            None => return Ok(false),
        };

        let current_chain_block_accumulator = MerkleAccumulator::new_with_info(
            self.status.status.info.get_block_accumulator_info().clone(),
            self.storage
                .get_accumulator_store(AccumulatorStoreType::Block),
        );
        let current_chain_dag_genesis =
            match current_chain_block_accumulator.get_leaf(dag_fork_height)? {
                Some(dag_genesis) => dag_genesis,
                None => return Ok(false),
            };

        if current_chain_dag_genesis != dag_genesis {
            return Ok(false);
        }

        self.dag.has_dag_block(header.id())
    }

    fn check_dag_type(&self, header: &BlockHeader) -> Result<DagHeaderType> {
        self.check_dag_type(header)
    }
}

impl BlockChain {
    pub fn filter_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
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
            let mut txn_info_ids = self.storage.get_block_txn_info_ids(block_id)?;
            if reverse {
                txn_info_ids.reverse();
            }
            for id in txn_info_ids.iter() {
                let events = self.storage.get_contract_events(*id)?.ok_or_else(|| {
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

                let txn_info = self.storage.get_transaction_info(*id)?.ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "cannot find txn info with txn_info_id {} on main chain(head: {})",
                        id,
                        chain_header.id()
                    ))
                })?;

                let filtered_event_with_info =
                    filtered_events.map(|(idx, evt)| ContractEventInfo {
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

    fn connect_dag(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock> {
        let dag = self.dag.clone();
        let (new_tip_block, _) = (executed_block.block(), executed_block.block_info());
        let (dag_genesis, mut tips) = self
            .current_tips_hash(new_tip_block.header())?
            .expect("tips should exists in dag");
        let parents = executed_block
            .block
            .header
            .parents_hash()
            .expect("Dag parents need exist");
        if !tips.contains(&new_tip_block.id()) {
            for hash in parents {
                tips.retain(|x| *x != hash);
            }
            if !dag.check_ancestor_of(new_tip_block.id(), tips.clone())? {
                tips.push(new_tip_block.id());
            }
        }
        // Caculate the ghostdata of the virutal node created by all tips.
        // And the ghostdata.selected of the tips will be the latest head.
        let block_hash = {
            let ghost_of_tips = dag.ghostdata(tips.as_slice())?;
            ghost_of_tips.selected_parent
        };
        let (block, block_info) = {
            let block = self
                .storage
                .get_block(block_hash)?
                .expect("Dag block should exist");
            let block_info = self
                .storage
                .get_block_info(block_hash)?
                .expect("Dag block info should exist");
            (block, block_info)
        };

        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let state_root = block.header().state_root();

        self.txn_accumulator = info_2_accumulator(
            txn_accumulator_info.clone(),
            AccumulatorStoreType::Transaction,
            self.storage.as_ref(),
        );
        self.block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            self.storage.as_ref(),
        );

        self.statedb = ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));

        self.status = ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
        };
        if self.epoch.end_block_number() == block.header().number() {
            self.epoch = get_epoch_from_statedb(&self.statedb)?;
        }
        self.dag.save_dag_state(dag_genesis, DagState { tips })?;
        Ok(executed_block)
    }

    pub fn dag_fork_height(&self) -> Result<Option<BlockNumber>> {
        if self.status().head().chain_id().is_test() {
            Ok(Some((20)))
        } else {
         Ok(self
            .statedb
            .get_on_chain_config::<FlexiDagConfig>()?
            .map(|c| c.effective_height))
        }
    }
}

impl ChainWriter for BlockChain {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool {
        executed_block.block.header().parent_hash() == self.status.status.head().id()
    }

    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock> {
        if self.check_dag_type(executed_block.block.header())? == DagHeaderType::Normal {
            info!(
                "connect a dag block, {:?}, number: {:?}",
                executed_block.block.id(),
                executed_block.block.header().number(),
            );
            return self.connect_dag(executed_block);
        }
        let (block, block_info) = (executed_block.block(), executed_block.block_info());
        //TODO try reuse accumulator and state db.
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let state_root = block.header().state_root();
        self.txn_accumulator = info_2_accumulator(
            txn_accumulator_info.clone(),
            AccumulatorStoreType::Transaction,
            self.storage.as_ref(),
        );
        self.block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            self.storage.as_ref(),
        );

        self.statedb = ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));
        self.status = ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
        };
        if self.epoch.end_block_number() == block.header().number() {
            self.epoch = get_epoch_from_statedb(&self.statedb)?;
            self.update_uncle_cache()?;
        } else if let Some(block_uncles) = block.uncles() {
            block_uncles.iter().for_each(|uncle_header| {
                self.uncles
                    .insert(uncle_header.id(), block.header().number());
            });
        }
        Ok(executed_block)
    }

    fn apply(&mut self, block: Block) -> Result<ExecutedBlock> {
        if self.check_dag_type(block.header())? != DagHeaderType::Normal {
            self.apply_with_verifier::<FullVerifier>(block)
        } else {
            self.apply_with_verifier::<DagVerifier>(block)
        }
    }

    fn chain_state(&mut self) -> &ChainStateDB {
        &self.statedb
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

fn get_epoch_from_statedb(statedb: &ChainStateDB) -> Result<Epoch> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader
        .get_resource::<Epoch>(genesis_address())?
        .ok_or_else(|| format_err!("Epoch is none."))
}
