// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::verifier::{BlockVerifier, FullVerifier};
use anyhow::{ensure, format_err, Result};
use consensus::Consensus;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_chain_api::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, ExcludedTxns, ExecutedBlock,
    VerifiedBlock, VerifyBlockField,
};
use starcoin_executor::BlockExecutedData;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::block::BlockIdAndNumber;
use starcoin_types::contract_event::ContractEventInfo;
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::{ChainInfo, ChainStatus};
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    error::BlockExecutorError,
    stress_test::TPS,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    U256,
};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_resource::{
    BlockMetadata, Epoch, EpochData, EpochInfo, GlobalTimeOnChain,
};
use starcoin_vm_types::time::TimeService;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use std::cmp::min;
use std::iter::Extend;
use std::option::Option::Some;
use std::{collections::HashSet, sync::Arc};
use storage::Store;

//TODO consider add BlockInfo to ChainStatus.
pub struct ChainStatusWithInfo {
    pub status: ChainStatus,
    pub info: BlockInfo,
    pub head: Block,
}

pub struct BlockChain {
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    status: ChainStatusWithInfo,
    statedb: ChainStateDB,
    storage: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    uncles: HashSet<HashValue>,
    epoch: Epoch,
}

impl BlockChain {
    pub fn new(
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
    ) -> Result<Self> {
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block by hash {:?}", head_block_hash))?;
        let block_info = storage
            .get_block_info(head_block_hash)?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", head_block_hash))?;
        debug!("Init chain with block_info: {:?}", block_info);
        let state_root = head.header().state_root();
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let block_accumulator_info = block_info.get_block_accumulator_info();
        let chain_state = ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root));
        let epoch = get_epoch_from_statedb(&chain_state)?;
        let mut chain = Self {
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
            status: ChainStatusWithInfo {
                status: ChainStatus::new(head.header.clone(), block_info.total_difficulty),
                info: block_info,
                head,
            },
            statedb: chain_state,
            storage,
            uncles: HashSet::new(),
            epoch,
        };
        chain.update_uncle_cache()?;
        Ok(chain)
    }

    pub fn new_with_genesis(
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
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
        let statedb = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let executed_block = Self::execute_block_and_save(
            storage.as_ref(),
            statedb,
            txn_accumulator,
            block_accumulator,
            &genesis_epoch,
            None,
            genesis_block,
        )?;
        Self::new(time_service, executed_block.block.id(), storage)
    }

    pub fn new_chain(&self, head_block_hash: HashValue) -> Result<Self> {
        let mut chain = Self::new(
            self.time_service.clone(),
            head_block_hash,
            self.storage.clone(),
        )?;
        chain.update_uncle_cache()?;
        Ok(chain)
    }

    pub fn current_epoch_uncles_size(&self) -> u64 {
        self.uncles.len() as u64
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        self.epoch.strategy()
    }
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    pub fn update_uncle_cache(&mut self) -> Result<()> {
        self.uncles = self.epoch_uncles()?.iter().cloned().collect();
        Ok(())
    }

    fn epoch_uncles(&self) -> Result<Vec<HashValue>> {
        let epoch = &self.epoch;
        let mut uncles = Vec::new();
        let mut block = self.head_block();
        let mut number = block.header().number();
        loop {
            if let Some(block_uncles) = block.uncles() {
                block_uncles.iter().for_each(|header| {
                    uncles.push(header.id());
                });
            }
            if number == 0 {
                break;
            }

            number -= 1;

            if epoch.start_block_number() > number || epoch.end_block_number() <= number {
                break;
            }

            block = self
                .get_block_by_number(number)?
                .ok_or_else(|| format_err!("Can not find block by number {}", number))?;
        }
        Ok(uncles)
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        author_auth_key: Option<AuthenticationKey>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        //FIXME create block template by parent may be use invalid chain state, such as epoch.
        //So the right way should bean create a BlockChain by parent_hash, then create block template.
        //the timestamp should bean a argument, if want to mock a early block.
        let previous_header = match parent_hash {
            Some(hash) => self
                .get_header(hash)?
                .ok_or_else(|| format_err!("Can find block header by {:?}", hash))?,
            None => self.current_header(),
        };

        self.create_block_template_inner(
            author,
            author_auth_key,
            previous_header,
            user_txns,
            uncles,
            block_gas_limit,
        )
    }

    fn create_block_template_inner(
        &self,
        author: AccountAddress,
        author_auth_key: Option<AuthenticationKey>,
        previous_header: BlockHeader,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let epoch = self.epoch();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        let strategy = epoch.strategy();
        let difficulty = strategy.calculate_next_difficulty(self)?;
        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            author_auth_key,
            self.time_service.now_millis(),
            uncles,
            difficulty,
            strategy,
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

    fn check_exist_transaction_info(&self, txn_info_id: HashValue) -> bool {
        if let Ok(node) = self.txn_accumulator.get_node(txn_info_id) {
            return node.is_some();
        }
        false
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
                if self.check_exist_block(block.id(), block.header().number)? {
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
                if self.check_exist_block(header.id(), header.number)? {
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

    fn total_txns_in_blocks(
        &self,
        start_number: BlockNumber,
        end_number: BlockNumber,
    ) -> Result<u64> {
        let txn_num_in_start_block = self
            .get_block_info_by_number(start_number)?
            .ok_or_else(|| format_err!("Can not find block info by number {}", start_number))?
            .get_txn_accumulator_info()
            .num_leaves;
        let txn_num_in_end_block = self
            .get_block_info_by_number(end_number)?
            .ok_or_else(|| format_err!("Can not find block info by number {}", end_number))?
            .get_txn_accumulator_info()
            .num_leaves;

        Ok(txn_num_in_end_block - txn_num_in_start_block)
    }

    pub fn can_be_uncle(&self, block_header: &BlockHeader) -> Result<bool> {
        FullVerifier::can_be_uncle(self, block_header)
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
    ) -> Result<ExecutedBlock> {
        let header = block.header();
        debug_assert!(header.is_genesis() || parent_status.is_some());
        debug_assert!(!header.is_genesis() || parent_status.is_none());
        let block_id = header.id();
        let txns = {
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

        let executed_data =
            starcoin_executor::block_execute(&statedb, txns.clone(), epoch.block_gas_limit())?;

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
            .fold(0u64, |acc, i| acc + i.gas_used());
        verify_block!(
            VerifyBlockField::State,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::State,
            vec_transaction_info.len() == txns.len(),
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
            executed_accumulator_root == header.accumulator_root(),
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

        let pre_total_difficulty = parent_status
            .map(|status| status.total_difficulty())
            .unwrap_or_default();

        let total_difficulty = pre_total_difficulty + header.difficulty();

        block_accumulator.append(&[block_id])?;
        block_accumulator.flush()?;

        let txn_accumulator_info: AccumulatorInfo = txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = block_accumulator.get_info();
        let block_info = BlockInfo::new_with_accumulator_info(
            block_id,
            txn_accumulator_info,
            block_accumulator_info,
            total_difficulty,
        );

        // save block's transaction relationship and save transaction
        Self::save(
            storage,
            block.clone(),
            block_info.clone(),
            txns,
            (executed_data.txn_infos, executed_data.txn_events),
        )?;
        Ok(ExecutedBlock { block, block_info })
    }

    fn save(
        storage: &dyn Store,
        block: Block,
        block_info: BlockInfo,
        transactions: Vec<Transaction>,
        txn_infos: (Vec<TransactionInfo>, Vec<Vec<ContractEvent>>),
    ) -> Result<()> {
        let block_id = block.id();
        let (txn_infos, txn_events) = txn_infos;
        debug_assert!(
            transactions.len() == txn_infos.len(),
            "block txns' length should be equal to txn infos' length"
        );
        debug_assert!(
            txn_events.len() == txn_infos.len(),
            "events' length should be equal to txn infos' length"
        );
        let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
        for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
            storage.save_contract_events(*info_id, events)?;
        }
        storage.save_block_txn_info_ids(block_id, txn_info_ids)?;
        storage.save_transaction_infos(txn_infos)?;

        let txn_id_vec = transactions
            .iter()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save block's transactions
        storage.save_block_transactions(block_id, txn_id_vec)?;
        // save transactions
        storage.save_transaction_batch(transactions)?;
        //TODO rework on blockstate
        let block_state = BlockState::Executed;
        storage.commit_block(block.clone(), block_state)?;
        storage.save_block_info(block_info)?;
        Ok(())
    }
}

impl ChainReader for BlockChain {
    fn info(&self) -> ChainInfo {
        //TODO implements
        unimplemented!()
    }

    fn status(&self) -> ChainStatus {
        self.status.status.clone()
    }

    fn head_block(&self) -> Block {
        self.status.head.clone()
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

    fn get_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>> {
        let mut block_vec = vec![];
        let mut current_num = match number {
            None => self.current_header().number(),
            Some(number) => number,
        };
        let mut tmp_count = count;
        loop {
            let block = self
                .get_block_by_number(current_num)?
                .ok_or_else(|| format_err!("Can not find block by number {}", current_num))?;
            block_vec.push(block);
            if current_num == 0 || tmp_count == 1 {
                break;
            }
            current_num -= 1;
            tmp_count -= 1;
        }
        Ok(block_vec)
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

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>> {
        let txn_info_ids = self.storage.get_transaction_info_ids_by_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            if self.check_exist_transaction_info(txn_info_id) {
                return self.storage.get_transaction_info(txn_info_id);
            }
        }
        Ok(None)
    }

    fn get_latest_block_by_uncle(&self, uncle_id: HashValue, times: u64) -> Result<Option<Block>> {
        let mut number = self.current_header().number();
        let latest_number = number;
        loop {
            if number == 0 || (number + times) <= latest_number {
                break;
            }

            let block = self
                .get_block_by_number(number)?
                .ok_or_else(|| format_err!("Can not find block by number {}", number))?;

            for uncle in block.uncles().unwrap_or_default() {
                if uncle.id() == uncle_id {
                    return Ok(Some(block));
                }
            }

            number -= 1;
        }

        Ok(None)
    }

    fn get_transaction_info_by_version(&self, version: u64) -> Result<Option<TransactionInfo>> {
        match self.txn_accumulator.get_leaf(version)? {
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
            None => Ok(Some(self.status.info.clone())),
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

    fn epoch_info(&self) -> Result<EpochInfo> {
        self.get_epoch_info_by_number(None)
    }

    fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    fn get_epoch_info_by_number(&self, number: Option<BlockNumber>) -> Result<EpochInfo> {
        let (epoch, epoch_data) = match number {
            None => (
                self.epoch.clone(),
                get_epoch_data_from_statedb(&self.statedb)?,
            ),
            Some(block_number) => {
                let header = self.get_header_by_number(block_number)?.ok_or_else(|| {
                    format_err!("Can not find header by block number:{}", block_number)
                })?;
                let statedb = ChainStateDB::new(
                    self.storage.clone().into_super_arc(),
                    Some(header.state_root()),
                );
                (
                    get_epoch_from_statedb(&statedb)?,
                    get_epoch_data_from_statedb(&statedb)?,
                )
            }
        };
        Ok(EpochInfo::new(epoch, epoch_data))
    }

    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain> {
        if let Some(block) = self.get_block_by_number(number)? {
            let chain_state = ChainStateDB::new(
                self.storage.clone().into_super_arc(),
                Some(block.header().state_root()),
            );
            let account_reader = AccountStateReader::new(&chain_state);
            Ok(account_reader
                .get_resource::<GlobalTimeOnChain>(genesis_address())?
                .ok_or_else(|| format_err!("GlobalTime is none."))?)
        } else {
            Err(format_err!("Block is none when query global time."))
        }
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

    /// Get tps for an epoch, the epoch includes the block given by `number`.
    /// If `number` is absent, return tps for the latest epoch
    fn tps(&self, number: Option<BlockNumber>) -> Result<TPS> {
        let epoch_info = self.get_epoch_info_by_number(number)?;
        let start_block_number = epoch_info.start_block_number();
        let end_block_number = epoch_info.end_block_number();
        let current_block_number = self.current_header().number();
        let start_block_time = self
            .get_header_by_number(start_block_number)?
            .ok_or_else(|| {
                format_err!("Can not find block header by number {}", start_block_number)
            })?
            .timestamp();
        let result = if end_block_number < current_block_number {
            let end_block_time = self
                .get_header_by_number(end_block_number)?
                .ok_or_else(|| {
                    format_err!("Can not find block header by number {}", end_block_number)
                })?
                .timestamp();
            let duration = (end_block_time - start_block_time) / 1000;
            let total_txns = self.total_txns_in_blocks(start_block_number, end_block_number)?;
            TPS::new(total_txns, duration, total_txns / duration)
        } else {
            let duration = (self.current_header().timestamp() - start_block_time) / 1000;
            let total_txns = self.total_txns_in_blocks(start_block_number, current_block_number)?;
            TPS::new(total_txns, duration, total_txns / duration)
        };
        Ok(result)
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
        BlockChain::new(self.time_service.clone(), block_id, self.storage.clone())
    }

    fn epoch_uncles(&self) -> &HashSet<HashValue> {
        &self.uncles
    }

    fn find_ancestor(&self, another: &dyn ChainReader) -> Result<Option<BlockIdAndNumber>> {
        let other_header_number = another.current_header().number();
        let self_header_number = self.current_header().number();
        let min_number = std::cmp::min(other_header_number, self_header_number);
        let max_number = std::cmp::max(other_header_number, self_header_number);
        let mut ancestor = None;
        for block_number in min_number..max_number {
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
                    break;
                }
            }
        }
        Ok(ancestor)
    }

    fn verify(&self, block: Block) -> Result<VerifiedBlock> {
        FullVerifier::verify_block(self, block)
    }

    fn execute(&self, verified_block: VerifiedBlock) -> Result<ExecutedBlock> {
        Self::execute_block_and_save(
            self.storage.as_ref(),
            self.statedb.fork(),
            self.txn_accumulator.fork(),
            self.block_accumulator.fork(),
            &self.epoch,
            Some(self.status.status.clone()),
            verified_block.0,
        )
    }
}

impl BlockChain {
    pub fn filter_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
        let reverse = filter.reverse;
        let chain_header = self.current_header();
        let max_block_number = chain_header.number.min(filter.to_block);

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
            let block_number = block.header().number;
            let mut txn_info_ids = self
                .storage
                .get_block_txn_info_ids(block_id)?
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>();
            if reverse {
                txn_info_ids.reverse();
            }
            for (idx, id) in txn_info_ids.iter() {
                let events = self.storage.get_contract_events(*id)?.ok_or_else(|| {
                    anyhow::anyhow!(format!(
                        "cannot find events of txn with txn_info_id {} on main chain(header: {})",
                        id,
                        chain_header.id()
                    ))
                })?;
                let mut filtered_events = events
                    .into_iter()
                    .filter(|evt| filter.matching(block_number, evt))
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

                let filtered_event_with_info = filtered_events.map(|evt| ContractEventInfo {
                    block_hash: block_id,
                    block_number: block.header().number,
                    transaction_hash: txn_info.transaction_hash(),
                    transaction_index: *idx as u32,
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
                cur_block_number -= 1;
            } else {
                cur_block_number += 1;
            }
        }

        // remove additional events in respect limit filter.
        if let Some(limit) = filter.limit {
            event_with_infos.truncate(limit);
        }
        Ok(event_with_infos)
    }
}

impl BlockChain {
    pub fn verify_with_verifier<V>(&mut self, block: Block) -> Result<VerifiedBlock>
    where
        V: BlockVerifier,
    {
        V::verify_block(self, block)
    }

    pub fn apply_with_verifier<V>(&mut self, block: Block) -> Result<()>
    where
        V: BlockVerifier,
    {
        let verified_block = self.verify_with_verifier::<V>(block)?;
        let executed_block = self.execute(verified_block)?;
        self.connect(executed_block)
    }

    pub fn update_chain_head(&mut self, block: Block) -> Result<()> {
        let block_info = self
            .storage
            .get_block_info(block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", block.id()))?;
        self.update_chain_head_with_info(block, block_info)
    }

    //TODO refactor update_chain_head and update_chain_head_with_info
    pub fn update_chain_head_with_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<()> {
        debug_assert!(block.header().parent_hash == self.status.status.head().id());
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

        if self.epoch.end_block_number() == block.header().number() {
            self.epoch = get_epoch_from_statedb(&self.statedb)?;
            self.update_uncle_cache()?;
        } else {
            if let Some(block_uncles) = block.uncles() {
                block_uncles.iter().for_each(|header| {
                    self.uncles.insert(header.id());
                });
            }
        }
        self.status = ChainStatusWithInfo {
            status: ChainStatus::new(block.header().clone(), block_info.total_difficulty),
            info: block_info,
            head: block,
        };
        Ok(())
    }
}

impl ChainWriter for BlockChain {
    fn connect(&mut self, executed_block: ExecutedBlock) -> Result<()> {
        self.update_chain_head_with_info(executed_block.block, executed_block.block_info)
    }

    fn apply(&mut self, block: Block) -> Result<()> {
        self.apply_with_verifier::<FullVerifier>(block)
    }

    fn chain_state(&mut self) -> &dyn ChainState {
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

fn get_epoch_data_from_statedb(statedb: &ChainStateDB) -> Result<EpochData> {
    let account_reader = AccountStateReader::new(statedb);
    account_reader
        .get_resource::<EpochData>(genesis_address())?
        .ok_or_else(|| format_err!("Epoch is none."))
}
