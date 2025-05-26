// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    fixed_blocks::MAIN_DIRECT_SAVE_BLOCK_HASH_MAP,
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
    ExcludedTxns, ExecutedBlock, MintedUncleNumber, TransactionInfoWithProof,
    TransactionInfoWithProof2, VerifiedBlock, VerifyBlockField,
};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_consensus::Consensus;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_executor::{BlockExecutedData, VMMetrics};
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::Store;
use starcoin_time_service::TimeService;
use starcoin_types::contract_event::StcContractEventInfo;
use starcoin_types::filter::Filter;
use starcoin_types::multi_state::MultiState;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::transaction::{RichTransactionInfo, TransactionInfo};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber, BlockTemplate},
    contract_event::ContractEvent,
    error::BlockExecutorError,
    transaction::Transaction,
    U256,
};
use starcoin_vm2_chain::{build_block_transactions, get_epoch_from_statedb};
use starcoin_vm2_state_api::{
    ChainStateReader as ChainStateReader2, ChainStateWriter as ChainStateWriter2,
};
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_storage::Store as Store2;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::{
    access_path::{AccessPath as AccessPath2, DataPath as DataPath2},
    on_chain_resource::Epoch,
};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::cmp::min;
use std::iter::Extend;
use std::option::Option::{None, Some};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{collections::HashMap, sync::Arc};

static OUTPUT_BLOCK: AtomicBool = AtomicBool::new(false);

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
    uncles: HashMap<HashValue, MintedUncleNumber>,
    epoch: Epoch,
    vm_metrics: Option<VMMetrics>,
}

impl BlockChain {
    pub fn new(
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", head_block_hash))?;
        Self::new_with_uncles(time_service, head, None, storage, storage2, vm_metrics)
    }

    fn new_with_uncles(
        time_service: Arc<dyn TimeService>,
        head_block: Block,
        uncles: Option<HashMap<HashValue, MintedUncleNumber>>,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
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
            assert!(vm_state_accumulator.num_leaves() > 1,);
            (
                vm_state_accumulator
                    .get_leaf(vm_state_accumulator.num_leaves() - 2)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find acc leaf {}",
                            vm_state_accumulator.num_leaves() - 2
                        )
                    })?,
                vm_state_accumulator
                    .get_leaf(vm_state_accumulator.num_leaves() - 1)?
                    .ok_or_else(|| {
                        format_err!(
                            "Can not find acc leaf {}",
                            vm_state_accumulator.num_leaves() - 1
                        )
                    })?,
            )
        };

        let chain_state = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root1));
        let chain_state2 = ChainStateDB2::new(storage2.clone().into_super_arc(), Some(state_root2));
        let epoch = get_epoch_from_statedb(&chain_state2)?;
        let genesis = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("Can not find genesis hash in storage."))?;
        watch(CHAIN_WATCH_NAME, "n1253");
        let mut chain = Self {
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
            uncles: HashMap::new(),
            epoch,
            vm_metrics,
        };
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
        storage2: Arc<dyn Store2>,
        genesis_epoch: Epoch,
        genesis_block: Block,
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
        let executed_block = Self::execute_block_and_save(
            (storage.as_ref(), storage2.as_ref()),
            (statedb, statedb2),
            txn_accumulator,
            block_accumulator,
            vm_state_accumulator,
            &genesis_epoch,
            None,
            genesis_block,
            None,
        )?;
        Self::new(
            time_service,
            executed_block.block().id(),
            storage,
            storage2,
            None,
        )
    }

    pub fn current_epoch_uncles_size(&self) -> u64 {
        self.uncles.len() as u64
    }

    pub fn current_block_accumulator_info(&self) -> AccumulatorInfo {
        self.block_accumulator.get_info()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.epoch.strategy()).expect("epoch consensus must be valid")
    }
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    //TODO lazy init uncles cache.
    fn update_uncle_cache(&mut self) -> Result<()> {
        self.uncles = self.epoch_uncles()?;
        Ok(())
    }

    fn epoch_uncles(&self) -> Result<HashMap<HashValue, MintedUncleNumber>> {
        let epoch = &self.epoch;
        let mut uncles: HashMap<HashValue, MintedUncleNumber> = HashMap::new();
        let executed_block = self.head_block();
        let head_block = executed_block.block();
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

    pub fn create_block_template_simple(
        &self,
        author: AccountAddress,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(author, None, vec![], vec![], None)
    }

    pub fn create_block_template_simple_with_txns(
        &self,
        author: AccountAddress,
        user_txns: Vec<MultiSignedUserTransaction>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(author, None, user_txns, vec![], None)
    }

    pub fn create_block_template_simple_with_uncles(
        &self,
        author: AccountAddress,
        uncles: Vec<BlockHeader>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        self.create_block_template(author, None, vec![], uncles, None)
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        parent_hash: Option<HashValue>,
        user_txns: Vec<MultiSignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
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

        self.create_block_template_inner(
            author,
            previous_header,
            user_txns,
            uncles,
            block_gas_limit,
        )
    }

    fn create_block_template_inner(
        &self,
        author: AccountAddress,
        previous_header: BlockHeader,
        user_txns: Vec<MultiSignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let epoch = self.epoch();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        let strategy = self.consensus();
        let difficulty = strategy.calculate_next_difficulty(self)?;
        let mut opened_block = OpenedBlock::new(
            self.storage.0.clone(),
            self.storage.1.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            self.time_service.now_millis(),
            uncles,
            difficulty,
            strategy,
            None,
        )?;
        // split user_txns to two parts
        let mut vm1_txns = vec![];
        let mut vm2_txns = vec![];
        for txn in user_txns {
            match txn {
                MultiSignedUserTransaction::VM1(txn) => vm1_txns.push(txn),
                MultiSignedUserTransaction::VM2(txn) => vm2_txns.push(txn),
            }
        }
        let excluded_txns = opened_block.push_txns(vm1_txns)?;
        let excluded_txns2 = opened_block.push_txns2(vm2_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns.absorb(excluded_txns2)))
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
        self.storage.0.clone()
    }

    pub fn get_storage2(&self) -> Arc<dyn Store2> {
        self.storage.1.clone()
    }

    pub fn can_be_uncle(&self, block_header: &BlockHeader) -> Result<bool> {
        FullVerifier::can_be_uncle(self, block_header)
    }

    pub fn verify_with_verifier<V>(&mut self, block: Block) -> Result<VerifiedBlock>
    where
        V: BlockVerifier,
    {
        V::verify_block(self, block)
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
        storage: (&dyn Store, &dyn Store2),
        statedb: (ChainStateDB, ChainStateDB2),
        txn_accumulator: MerkleAccumulator,
        block_accumulator: MerkleAccumulator,
        vm_state_accumulator: MerkleAccumulator,
        epoch: &Epoch,
        parent_status: Option<ChainStatus>,
        block: Block,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<ExecutedBlock> {
        let (storage, storage2) = storage;
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
                        let block_metadata = block.to_metadata(parent.head().gas_used());
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
                .map(|p| block.to_metadata2(p.head.gas_used())),
        );

        assert!(!transactions2.is_empty());

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

        let (state_root, multi_state) = {
            // if no txns, state_root is kept unchanged after calling txn-execution
            let state_root1 = executed_data.state_root;
            let state_root2 = executed_data2.state_root;
            vm_state_accumulator.append(&[state_root1, state_root2])?;
            (
                vm_state_accumulator.root_hash(),
                MultiState::new(state_root1, state_root2),
            )
        };

        let vec_transaction_info = &executed_data.txn_infos;
        let vm2_txn_infos = executed_data2
            .txn_infos
            .clone()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<TransactionInfo>>();

        verify_block!(
            VerifyBlockField::State,
            state_root == header.state_root(),
            "verify block:{:?} state_root {:?}, in header {:?} fail, multi_state {:?}",
            block_id,
            state_root,
            header.state_root(),
            multi_state
        );
        let block_gas_used = vec_transaction_info
            .iter()
            .chain(vm2_txn_infos.iter())
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

        verify_block!(
            VerifyBlockField::State,
            vm2_txn_infos.len() == transactions2.len(),
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
            txn_accumulator.append(&included_txn_info_hashes)?;
            txn_accumulator.append(&included_txn_info_hashes2)?;
            txn_accumulator.root_hash()
        };

        verify_block!(
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        watch(CHAIN_WATCH_NAME, "n23");
        statedb2
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;
        statedb
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

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

        // save transaction relationship and save transaction to storage2
        starcoin_vm2_chain::save_executed_transactions(
            block_id,
            header.number(),
            storage2,
            transactions2,
            executed_data2,
            // todo: how to track vm2 transaction global index?
            transaction_global_index + executed_data.txn_infos.len() as u64,
        )?;

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
                .chain(vm2_txn_infos)
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
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
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
            VerifyBlockField::State,
            executed_accumulator_root == header.txn_accumulator_root(),
            "verify block: txn accumulator root mismatch"
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
        storage
            .get_block_by_hash(hash)
            .and_then(|block| self.exist_block_filter(block))
    }

    fn get_hash_by_number(&self, number: BlockNumber) -> Result<Option<HashValue>> {
        self.block_accumulator.get_leaf(number)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        let (storage, _storage2) = &self.storage;
        //TODO check txn should exist on current chain.
        storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<RichTransactionInfo>> {
        let (storage, _storage2) = &self.storage;
        let txn_info_ids = storage.get_transaction_info_ids_by_txn_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            let txn_info = storage.get_transaction_info(txn_info_id)?;
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
        let (storage, _storage2) = &self.storage;
        match self.txn_accumulator.get_leaf(transaction_global_index)? {
            None => Ok(None),
            Some(hash) => storage.get_transaction_info(hash),
        }
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
            self.exist_block(block_id)?,
            "Block with id{} do not exists in current chain.",
            block_id
        );
        let head = storage
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
            storage.clone(),
            storage2.clone(),
            self.vm_metrics.clone(),
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
        FullVerifier::verify_block(self, block)
    }

    fn execute(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        if let Some((executed_data, block_info)) =
            MAIN_DIRECT_SAVE_BLOCK_HASH_MAP.get(&verified_block.0.header.id())
        {
            Self::execute_save_directly(
                self.storage.0.as_ref(),
                self.statedb.0.fork(),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                Some(self.status.status.clone()),
                verified_block.0,
                block_info.clone().into(),
                executed_data.clone(),
            )
        } else {
            Self::execute_block_and_save(
                (self.storage.0.as_ref(), self.storage.1.as_ref()),
                (self.statedb.0.fork(), self.statedb.1.fork()),
                self.txn_accumulator.fork(None),
                self.block_accumulator.fork(None),
                self.vm_state_accumulator.fork(None),
                &self.epoch,
                Some(self.status.status.clone()),
                verified_block.0,
                self.vm_metrics.clone(),
            )
        }
    }

    fn execute_without_save(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        Self::execute_block_without_save(
            self.statedb.0.fork(),
            self.txn_accumulator.fork(None),
            self.block_accumulator.fork(None),
            &self.epoch,
            Some(self.status.status.clone()),
            verified_block.0,
            self.vm_metrics.clone(),
        )
    }

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>> {
        let (storage, _storage2) = &self.storage;
        let chain_header = self.current_header();
        let hashes = self
            .txn_accumulator
            .get_leaves(start_index, reverse, max_size)?;
        let mut infos = vec![];
        let txn_infos = storage.get_transaction_infos(hashes.clone())?;
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
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        let (storage, _storage2) = &self.storage;
        let (statedb, _statedb2) = &self.statedb;
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

        //if can get proof by leaf_index, the leaf and transaction info should exist.
        let txn_info_hash = accumulator
            .get_leaf(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find txn info hash by index {}",
                    transaction_global_index
                )
            })?;
        let transaction_info = storage
            .get_transaction_info(txn_info_hash)?
            .ok_or_else(|| format_err!("Can not find txn info by hash:{}", txn_info_hash))?;

        let event_proof = if let Some(event_index) = event_index {
            let events = storage
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
            let statedb = statedb.fork_at(transaction_info.txn_info().state_root_hash());
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

    fn get_transaction_proof2(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath2>,
    ) -> Result<Option<TransactionInfoWithProof2>> {
        let (storage, storage2) = &self.storage;
        let (_, statedb2) = &self.statedb;
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

        //if can get proof by leaf_index, the leaf and transaction info should exist.
        let txn_info_hash = accumulator
            .get_leaf(transaction_global_index)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find txn info hash by index {}",
                    transaction_global_index
                )
            })?;
        let transaction_info = storage
            .get_transaction_info(txn_info_hash)?
            .ok_or_else(|| format_err!("Can not find txn info by hash:{}", txn_info_hash))?;

        let event_proof = if let Some(event_index) = event_index {
            let events = storage2
                .get_contract_events(txn_info_hash)?
                .unwrap_or_default();
            let event = events.get(event_index as usize).cloned().ok_or_else(|| {
                format_err!("event index out of range, events len:{}", events.len())
            })?;
            let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();

            let event_proof =
                InMemoryAccumulator::get_proof_from_leaves(event_hashes.as_slice(), event_index)?;
            Some(EventWithProof2 {
                event,
                proof: event_proof,
            })
        } else {
            None
        };
        let state_proof = if let Some(access_path) = access_path {
            let statedb = statedb2.fork_at(transaction_info.txn_info().state_root_hash());
            let state_key = match access_path.path {
                DataPath2::Code(module_name) => {
                    StateKey::module(&access_path.address, &module_name)
                }
                DataPath2::Resource(struct_tag) => {
                    StateKey::resource(&access_path.address, &struct_tag)?
                }
                DataPath2::ResourceGroup(struct_tag) => {
                    StateKey::resource_group(&access_path.address, &struct_tag)
                }
            };
            Some(statedb.get_with_proof(&state_key)?)
        } else {
            None
        };
        Ok(Some(TransactionInfoWithProof2 {
            transaction_info,
            proof: txn_proof,
            event_proof,
            state_proof,
        }))
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

                let txn_info = storage.get_transaction_info(*id)?.ok_or_else(|| {
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
                        event: evt.into(),
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
}

impl ChainWriter for BlockChain {
    fn can_connect(&self, executed_block: &ExecutedBlock) -> bool {
        executed_block.block().header().parent_hash() == self.status.status.head().id()
    }

    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<ExecutedBlock> {
        let (storage, storage2) = &self.storage;
        let (block, block_info) = (executed_block.block(), executed_block.block_info());
        debug_assert!(block.header().parent_hash() == self.status.status.head().id());
        //TODO try reuse accumulator and state db.
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

        let (state_root1, state_root2) = {
            let state = executed_block.multi_state();
            (state.state_root1(), state.state_root2())
        };

        self.statedb = (
            ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root1)),
            ChainStateDB2::new(storage2.clone().into_super_arc(), Some(state_root2)),
        );
        self.status = ChainStatusWithBlock {
            status: ChainStatus::new(block.header().clone(), block_info.clone()),
            head: block.clone(),
            multi_state: executed_block.multi_state().clone(),
        };
        if self.epoch.end_block_number() == block.header().number() {
            self.epoch = get_epoch_from_statedb(&self.statedb.1)?;
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
        self.apply_with_verifier::<FullVerifier>(block)
    }

    fn chain_state(&mut self) -> &ChainStateDB {
        &self.statedb.0
    }

    fn chain_state2(&mut self) -> &ChainStateDB2 {
        &self.statedb.1
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
