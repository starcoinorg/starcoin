// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use consensus::Consensus;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator, MerkleAccumulator,
};
use starcoin_chain_api::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, ExcludedTxns, VerifyBlockField,
};
use starcoin_executor::BlockExecutedData;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_types::contract_event::ContractEventInfo;
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::{
    account_address::AccountAddress,
    block::{
        Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate,
        ALLOWED_FUTURE_BLOCKTIME,
    },
    contract_event::ContractEvent,
    error::BlockExecutorError,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    U256,
};
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_resource::{Epoch, EpochInfo, GlobalTimeOnChain};
use starcoin_vm_types::time::TimeService;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use std::cmp::min;
use std::iter::Extend;
use std::{collections::HashSet, sync::Arc};
use storage::Store;

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

pub struct BlockChain {
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    head: Option<Block>,
    chain_state: ChainStateDB,
    storage: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    uncles: HashSet<HashValue>,
    epoch: Option<Epoch>,
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
            head: Some(head),
            chain_state: ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root)),
            storage,
            uncles: HashSet::new(),
            epoch: None,
        };
        chain.update_epoch_and_uncle_cache()?;
        Ok(chain)
    }

    pub fn init_empty_chain(time_service: Arc<dyn TimeService>, storage: Arc<dyn Store>) -> Self {
        let txn_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        let block_accumulator = MerkleAccumulator::new_empty(
            storage.get_accumulator_store(AccumulatorStoreType::Block),
        );
        Self {
            time_service,
            txn_accumulator,
            block_accumulator,
            head: None,
            chain_state: ChainStateDB::new(storage.clone().into_super_arc(), None),
            storage,
            uncles: HashSet::new(),
            epoch: None,
        }
    }

    pub fn new_chain(&self, head_block_hash: HashValue) -> Result<Self> {
        let mut chain = Self::new(
            self.time_service.clone(),
            head_block_hash,
            self.storage.clone(),
        )?;
        chain.update_epoch_and_uncle_cache()?;
        Ok(chain)
    }

    pub fn current_epoch_uncles_size(&self) -> usize {
        self.uncles.len()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        self.epoch.as_ref().unwrap().strategy()
    }
    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    pub fn update_epoch_and_uncle_cache(&mut self) -> Result<()> {
        let epoch_resource = self.get_epoch_resource_by_number(None)?;
        self.uncles = self
            .epoch_uncles(&epoch_resource)?
            .iter()
            .cloned()
            .collect();
        self.epoch = Some(epoch_resource);
        Ok(())
    }

    pub fn can_be_uncle(&self, block_header: &BlockHeader) -> bool {
        let epoch = self.epoch.as_ref().expect("epoch is none.");
        epoch.start_block_number() <= block_header.number()
            && epoch.end_block_number() > block_header.number()
            && self.exist_block(block_header.parent_hash())
            && !self.exist_block(block_header.id())
            && !self.uncles.contains(&block_header.id())
            && block_header.number() <= self.current_header().number()
    }

    fn epoch_uncles(&self, epoch_resource: &Epoch) -> Result<Vec<HashValue>> {
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

            if epoch_resource.start_block_number() > number
                || epoch_resource.end_block_number() <= number
            {
                break;
            }

            block = self
                .get_block_by_number(number)?
                .ok_or_else(|| format_err!("Can not find block by number {}", number))?;
        }
        Ok(uncles)
    }

    fn get_block_info_inner(&self, block_id: HashValue) -> Result<BlockInfo> {
        Ok(self
            .storage
            .get_block_info(block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", block_id))?)
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
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        ensure!(self.exist_block(block_id), "Block id not exist");

        let previous_header = self
            .get_header(block_id)?
            .ok_or_else(|| format_err!("Can find block header by {:?}", block_id))?;
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
        let on_chain_block_gas_limit = self.get_on_chain_block_gas_limit()?;
        let final_block_gas_limit = block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);
        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            author_auth_key,
            self.time_service.now_millis(),
            uncles,
        )?;
        let excluded_txns = opened_block.push_txns(user_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns))
    }

    pub fn get_on_chain_block_gas_limit(&self) -> Result<u64> {
        self.epoch
            .as_ref()
            .map(|epoch| epoch.block_gas_limit())
            .ok_or_else(|| format_err!("Chain EpochResource is empty."))
    }

    pub fn find_block_by_number(&self, number: u64) -> Result<HashValue> {
        self.block_accumulator
            .get_leaf(number)?
            .ok_or_else(|| format_err!("Can not find block by number {}", number))
    }

    fn transaction_info_exist(&self, txn_info_id: HashValue) -> bool {
        if let Ok(node) = self.txn_accumulator.get_node(txn_info_id) {
            return node.is_some();
        }
        false
    }

    pub fn block_exist_by_number(
        &self,
        block_id: HashValue,
        block_num: BlockNumber,
    ) -> Result<bool> {
        if let Some(block_header) = self.get_header_by_number(block_num)? {
            if block_id == block_header.id() {
                return Ok(true);
            } else {
                debug!(
                    "block id miss match {:?} : {:?}",
                    block_id,
                    block_header.id()
                );
            }
        }

        Ok(false)
    }

    pub fn get_storage(&self) -> Arc<dyn Store> {
        self.storage.clone()
    }

    fn block_with_number(&self, number: Option<BlockNumber>) -> Result<Option<Block>> {
        let num = match number {
            Some(n) => n,
            None => self.current_header().number(),
        };

        self.get_block_by_number(num)
    }

    fn get_epoch_resource_by_number(&self, number: Option<BlockNumber>) -> Result<Epoch> {
        if let Some(block) = self.block_with_number(number)? {
            let chain_state = ChainStateDB::new(
                self.storage.clone().into_super_arc(),
                Some(block.header().state_root()),
            );
            let account_reader = AccountStateReader::new(&chain_state);
            let epoch = account_reader
                .get_resource::<Epoch>(genesis_address())?
                .ok_or_else(|| format_err!("Epoch is none."))?;
            Ok(epoch)
        } else {
            Err(format_err!("Block is none when query epoch resource."))
        }
    }

    pub fn get_chain_info(&self) -> Result<ChainInfo> {
        //TODO cache the chain info.
        let header = self.current_header();
        let total_difficulty = self.get_total_difficulty()?;
        Ok(ChainInfo::new(header, total_difficulty))
    }

    fn ensure_head(&self) -> &Block {
        self.head
            .as_ref()
            .expect("head block must some after chain init.")
    }
}

impl ChainReader for BlockChain {
    fn head_block(&self) -> Block {
        self.ensure_head().clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.ensure_head().header().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        let header = if let Some(block) = self.get_block(hash)? {
            Some(block.header().clone())
        } else {
            None
        };

        Ok(header)
    }

    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        let block_id = self.find_block_by_number(number)?;
        self.storage.get_block_header_by_hash(block_id)
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        let block_id = self.find_block_by_number(number)?;
        self.storage.get_block_by_hash(block_id)
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
        let block = self.storage.get_block_by_hash(hash)?;
        match block {
            Some(b) => {
                let block_exit =
                    self.block_exist_by_number(b.header().id(), b.header().number())?;
                if block_exit {
                    return Ok(Some(b));
                }
            }
            None => {
                debug!("Get block {:?} from storage return none.", hash);
            }
        }

        Ok(None)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>> {
        let txn_info_ids = self.storage.get_transaction_info_ids_by_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            if self.transaction_info_exist(txn_info_id) {
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

            if block.uncles().is_some() {
                for uncle in block.uncles().expect("uncles is none.") {
                    if uncle.id() == uncle_id {
                        return Ok(Some(block));
                    }
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
        &self.chain_state
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        let id = match block_id {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        self.storage.get_block_info(id)
    }

    fn get_total_difficulty(&self) -> Result<U256> {
        let id = self.head_block().id();
        let block_info = self
            .storage
            .get_block_info(id)?
            .ok_or_else(|| format_err!("Can not find block info by id {}", id))?;
        Ok(block_info.total_difficulty)
    }

    fn exist_block(&self, block_id: HashValue) -> bool {
        if let Ok(Some(header)) = self.storage.get_block_header_by_hash(block_id) {
            if let Ok(exist) = self.block_exist_by_number(block_id, header.number()) {
                return exist;
            }
        }
        false
    }

    fn epoch_info(&self) -> Result<EpochInfo> {
        self.get_epoch_info_by_number(None)
    }

    fn get_epoch_info_by_number(&self, number: Option<BlockNumber>) -> Result<EpochInfo> {
        if let Some(block) = self.block_with_number(number)? {
            let chain_state = ChainStateDB::new(
                self.storage.clone().into_super_arc(),
                Some(block.header().state_root()),
            );
            let account_reader = AccountStateReader::new(&chain_state);
            account_reader.get_epoch_info()
        } else {
            Err(format_err!("Block is none when query epoch info."))
        }
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
        max_size: usize,
    ) -> Result<Vec<HashValue>> {
        self.block_accumulator
            .get_leaves(start_number, reverse, max_size)
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
                    "cannot find block({}) on master chain(head: {})",
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
                        "cannot find events of txn with txn_info_id {} on master chain(header: {})",
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
                        "cannot find txn info with txn_info_id {} on master chain(head: {})",
                        id,
                        chain_header.id()
                    ))
                })?;

                let filtered_event_with_info = filtered_events.map(|evt| ContractEventInfo {
                    block_hash: block_id,
                    block_number: block.header().number,
                    transaction_hash: txn_info.transaction_hash(),
                    transaction_index: *idx as u64,
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
    #[cfg(test)]
    pub fn save_fot_test(
        &mut self,
        block_id: HashValue,
        transactions: Vec<Transaction>,
        txn_infos: Option<(Vec<TransactionInfo>, Vec<Vec<ContractEvent>>)>,
    ) -> Result<()> {
        self.save(block_id, transactions, txn_infos)
    }

    fn save(
        &mut self,
        block_id: HashValue,
        transactions: Vec<Transaction>,
        txn_infos: Option<(Vec<TransactionInfo>, Vec<Vec<ContractEvent>>)>,
    ) -> Result<()> {
        if txn_infos.is_some() {
            let (txn_infos, txn_events) = txn_infos.expect("txn infos is none.");
            ensure!(
                transactions.len() == txn_infos.len(),
                "block txns' length should be equal to txn infos' length"
            );
            ensure!(
                txn_events.len() == txn_infos.len(),
                "events' length should be equal to txn infos' length"
            );
            let txn_info_ids: Vec<_> = txn_infos.iter().map(|info| info.id()).collect();
            for (info_id, events) in txn_info_ids.iter().zip(txn_events.into_iter()) {
                self.storage.save_contract_events(*info_id, events)?;
            }
            self.storage
                .save_block_txn_info_ids(block_id, txn_info_ids)?;
            self.storage.save_transaction_infos(txn_infos)?;
        }

        let txn_id_vec = transactions
            .iter()
            .cloned()
            .map(|user_txn| user_txn.id())
            .collect::<Vec<HashValue>>();
        // save block's transactions
        self.storage.save_block_transactions(block_id, txn_id_vec)?;
        // save transactions
        self.storage.save_transaction_batch(transactions)?;

        Ok(())
    }

    fn verify_header(&self, header: &BlockHeader, is_uncle: bool, epoch: &EpochInfo) -> Result<()> {
        let parent_hash = header.parent_hash();
        if !is_uncle {
            let current_head_id = self.head_block().id();
            let current_number = self.head_block().header().number();
            let expect_number = current_number + 1;

            verify_block!(
                VerifyBlockField::Header,
                expect_number == header.number,
                "Invalid block: Unexpect block number, expect:{}, got: {}.",
                expect_number,
                header.number
            );

            verify_block!(
                VerifyBlockField::Header,
                current_head_id == parent_hash,
                "Invalid block: Parent id mismatch, expect:{}, got: {}, number:{}.",
                current_head_id.to_hex(),
                parent_hash.to_hex(),
                header.number
            );
        }
        // do not check genesis block timestamp check
        if !header.is_genesis() {
            //block header time check in block prologue.

            let now = self.time_service.now_millis();
            verify_block!(
                VerifyBlockField::Header,
                header.timestamp() <= ALLOWED_FUTURE_BLOCKTIME + now,
                "Invalid block: block timestamp too new, now:{}, block time:{}",
                now,
                header.timestamp()
            );
        }

        let consensus = epoch.epoch().strategy();
        // TODO 最小值是否需要
        if let Err(err) = if is_uncle {
            let uncle_branch =
                BlockChain::new(self.time_service(), parent_hash, self.storage.clone())?;
            consensus.verify(&uncle_branch, epoch, header)
        } else {
            consensus.verify(self, epoch, header)
        } {
            return Err(
                ConnectBlockError::VerifyBlockFailed(VerifyBlockField::Consensus, err).into(),
            );
        };
        Ok(())
    }

    fn verify_uncles(&self, uncles: &[BlockHeader], header: &BlockHeader) -> Result<()> {
        verify_block!(
            VerifyBlockField::Uncle,
            uncles.len() <= MAX_UNCLE_COUNT_PER_BLOCK,
            "too many uncles {} in block {}",
            uncles.len(),
            header.id()
        );
        let mut uncle_ids = HashSet::<HashValue>::new();
        for uncle in uncles {
            verify_block!(
                VerifyBlockField::Uncle,
                !uncle_ids.contains(&uncle.id()),
                "repeat uncle {:?} in current block {:?}",
                uncle.id(),
                header.id()
            );

            verify_block!(
                VerifyBlockField::Uncle,
                uncle.number < header.number ,
               "uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}", uncle.number, header.number
            );

            uncle_ids.insert(uncle.id());
        }

        let (epoch_start_number, epoch_end_number) = if let Some(epoch) = &self.epoch {
            verify_block!(
                VerifyBlockField::Uncle,
                header.number() > epoch.start_block_number()
                    && header.number() <= epoch.end_block_number(),
                "block number is {:?}, epoch start number is {:?}, epoch end number is {:?}",
                header.number(),
                epoch.start_block_number(),
                epoch.end_block_number(),
            );
            (epoch.start_block_number(), epoch.end_block_number())
        } else {
            return Err(ConnectBlockError::VerifyBlockFailed(
                VerifyBlockField::Uncle,
                format_err!("epoch is none.",),
            )
            .into());
        };

        //let uncle_ids: Vec<_> = uncles.iter().map(|uncle| uncle.id()).collect();
        debug!("verify block : {:?} uncle ids {:?}", header.id(), uncle_ids);
        for uncle_id in uncle_ids {
            verify_block!(
                VerifyBlockField::Uncle,
                !self.exist_block(uncle_id),
                "legal master block can not be uncle block,block id is {:?}",
                uncle_id
            );

            debug!("uncle block exists in master,uncle id is {:?}", uncle_id,);
            verify_block!(
                VerifyBlockField::Uncle,
                !self.uncles.contains(&uncle_id),
                "uncle block exists in master,uncle id is {:?}",
                uncle_id
            );
        }

        for uncle in uncles {
            verify_block!(
                VerifyBlockField::Uncle,
                uncle.number() >= epoch_start_number && uncle.number() < epoch_end_number,
                "uncle not in epoch, id is: {:?}, number is: {:?}, epoch start {:?} and end {:?}",
                uncle.id(),
                uncle.number(),
                epoch_start_number,
                epoch_end_number
            );

            verify_block!(
                VerifyBlockField::Uncle,
                self.exist_block(uncle.parent_hash()),
                "can't find parent, uncle id is {:?}, and parent id is {:?}",
                uncle.id(),
                uncle.parent_hash()
            );

            let parent_header = self.get_header(uncle.parent_hash())?;
            verify_block!(
                VerifyBlockField::Uncle,
                parent_header.is_some(),
                "can't find parent header by id {:?} for uncle id {:?} ",
                uncle.parent_hash(),
                uncle.id()
            );

            let parent_header = parent_header.expect("Parent header is none.");
            let next_number = parent_header.number() + 1;
            verify_block!(
                VerifyBlockField::Uncle,
                next_number == uncle.number(),
                "illegal uncle number {}, parent number is {}",
                uncle.number(),
                parent_header.number()
            );
        }

        Ok(())
    }

    fn apply_inner(
        &mut self,
        block: Block,
        execute: bool,
        state_reader: Option<&dyn ChainStateReader>,
    ) -> Result<()> {
        let header = block.header().clone();
        let block_id = header.id();
        let is_genesis = header.is_genesis();
        let gas_limit = if is_genesis {
            u64::MIN
        } else {
            self.get_on_chain_block_gas_limit()?
        };

        verify_block!(
            VerifyBlockField::Header,
            header.gas_used() <= gas_limit,
            "invalid block: gas_used should not greater than gas_limit"
        );

        verify_block!(
            VerifyBlockField::Header,
            block.body.hash() == header.body_hash(),
            "verify block:{:?} body hash fail.",
            header.id(),
        );

        let mut switch_epoch = false;
        if !is_genesis {
            let account_reader = match state_reader {
                Some(state_reader) => AccountStateReader::new(state_reader),
                None => AccountStateReader::new(&self.chain_state),
            };
            let epoch_info = account_reader.get_epoch_info()?;
            self.verify_header(&header, false, &epoch_info)?;

            // verify block accumulator
            let parent_block_info = self.get_block_info(Some(header.parent_hash()))?;
            verify_block!(
                VerifyBlockField::Header,
                parent_block_info.is_some(),
                "Can not find BlockInfo by parent id : {:?}",
                header.parent_hash(),
            );
            let parent_block_info = parent_block_info.expect("Parent block info is none.");
            verify_block!(
                VerifyBlockField::Header,
                parent_block_info
                    .get_block_accumulator_info()
                    .get_accumulator_root()
                    == &header.parent_block_accumulator_root(),
                "Block accumulator root miss match {:?} : {:?}",
                parent_block_info
                    .get_block_accumulator_info()
                    .get_accumulator_root(),
                header.parent_block_accumulator_root(),
            );

            verify_block!(
                VerifyBlockField::Header,
                block.body.hash() == header.body_hash(),
                "verify block:{:?} body hash fail.",
                header.id(),
            );

            if header.number() == epoch_info.end_block_number() {
                switch_epoch = true;
            }

            if switch_epoch {
                verify_block!(
                    VerifyBlockField::Uncle,
                    block.uncles().is_none(),
                    "invalid block: block uncle must be empty."
                );
            }
            if let Some(uncles) = block.uncles() {
                for uncle_header in uncles {
                    verify_block!(
                        VerifyBlockField::Uncle,
                        self.can_be_uncle(uncle_header),
                        "invalid block: block {} can not be uncle.",
                        uncle_header.id()
                    );
                    self.verify_header(uncle_header, true, &epoch_info)?;
                }
            }
        }

        let txns = {
            let mut t = if is_genesis {
                vec![]
            } else {
                let parent_header = self.get_header(header.parent_hash())?;
                verify_block!(
                    VerifyBlockField::Header,
                    parent_header.is_some(),
                    "invalid block: parent block header {} is none.",
                    header.parent_hash()
                );
                let block_metadata =
                    block.to_metadata(parent_header.expect("header is none.").gas_used());
                vec![Transaction::BlockMetadata(block_metadata)]
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

        let executed_data = if execute {
            starcoin_executor::block_execute(&self.chain_state, txns.clone(), gas_limit)?
        } else {
            self.verify_txns(block_id, txns.as_slice())?
        };
        let state_root = executed_data.state_root;
        let vec_transaction_info = &executed_data.txn_infos;
        verify_block!(
            VerifyBlockField::Header,
            state_root == header.state_root(),
            "verify block:{:?} state_root fail",
            header.id(),
        );
        let block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc + i.gas_used());
        verify_block!(
            VerifyBlockField::Header,
            block_gas_used == header.gas_used(),
            "invalid block: gas_used is not match"
        );

        verify_block!(
            VerifyBlockField::Body,
            vec_transaction_info.len() == txns.len(),
            "invalid txn num in the block"
        );

        // txn accumulator verify.
        let executed_accumulator_root = {
            let included_txn_info_hashes: Vec<_> =
                vec_transaction_info.iter().map(|info| info.id()).collect();
            self.txn_accumulator.append(&included_txn_info_hashes)?
        };

        verify_block!(
            VerifyBlockField::Header,
            executed_accumulator_root == header.accumulator_root(),
            "verify block: txn accumulator root mismatch"
        );

        // If chain state is matched, and accumulator is matched,
        // then, we save flush states, and save block data.
        self.txn_accumulator
            .flush()
            .map_err(|_err| BlockExecutorError::BlockAccumulatorFlushErr)?;
        self.chain_state
            .flush()
            .map_err(BlockExecutorError::BlockChainStateErr)?;

        let total_difficulty = {
            if is_genesis {
                header.difficulty()
            } else {
                let pre_total_difficulty = self
                    .get_block_info_inner(header.parent_hash())?
                    .total_difficulty;
                pre_total_difficulty + header.difficulty()
            }
        };

        self.block_accumulator.append(&[block.id()])?;
        self.block_accumulator.flush()?;
        let txn_accumulator_info: AccumulatorInfo = self.txn_accumulator.get_info();
        let block_accumulator_info: AccumulatorInfo = self.block_accumulator.get_info();
        let block_info = BlockInfo::new_with_accumulator_info(
            header.id(),
            txn_accumulator_info,
            block_accumulator_info,
            total_difficulty,
        );
        // save block's transaction relationship and save transaction
        self.save(
            header.id(),
            txns,
            //TODO refactor this, there some weird
            if execute {
                Some((executed_data.txn_infos, executed_data.txn_events))
            } else {
                None
            },
        )?;
        let block_state = if execute {
            BlockState::Executed
        } else {
            BlockState::Verified
        };
        let uncles = block.uncles();
        self.commit(block.clone(), block_info, block_state)?;

        // update cache
        if switch_epoch {
            self.update_epoch_and_uncle_cache()?;
        } else if let Some(block_uncles) = uncles {
            block_uncles.iter().for_each(|header| {
                self.uncles.insert(header.id());
            });
        }
        Ok(())
    }

    pub fn update_chain_head(&mut self, block: Block) -> Result<()> {
        let block_info = self
            .storage
            .get_block_info(block.id())?
            .ok_or_else(|| format_err!("Can not find block info by hash {:?}", block.id()))?;
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
        self.chain_state =
            ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));
        if self.epoch.is_some()
            && self
                .epoch
                .as_ref()
                .expect("epoch resource is none.")
                .end_block_number()
                == block.header().number()
        {
            self.head = Some(block);
            self.update_epoch_and_uncle_cache()?;
        } else {
            if let Some(block_uncles) = block.uncles() {
                block_uncles.iter().for_each(|header| {
                    self.uncles.insert(header.id());
                });
            }
            self.head = Some(block);
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn verify_txns_for_test(
        &self,
        block_id: HashValue,
        txns: &[Transaction],
    ) -> Result<BlockExecutedData> {
        self.verify_txns(block_id, txns)
    }

    fn verify_txns(&self, block_id: HashValue, txns: &[Transaction]) -> Result<BlockExecutedData> {
        let mut block_executed_data = BlockExecutedData::default();

        let txn_infos = self.storage.get_block_transaction_infos(block_id)?;

        verify_block!(
            VerifyBlockField::Body,
            txn_infos.len() == txns.len(),
            "txn infos len ({:?}) is not equals txns len ({:?}).",
            txn_infos.len(),
            txns.len()
        );
        let mut state_root = None;
        for i in 0..txns.len() {
            let id = txns[i].id();
            verify_block!(
                VerifyBlockField::Body,
                id == txn_infos[i].transaction_hash(),
                "txn {:?} is not match with txn_info ({:?}).",
                id,
                txn_infos[i]
            );
            state_root = Some(txn_infos[i].state_root_hash());
        }
        block_executed_data.state_root =
            state_root.expect("txn infos is not empty, state root must not been None");
        block_executed_data.txn_infos = txn_infos;
        //TODO event?
        Ok(block_executed_data)
    }

    fn commit(
        &mut self,
        block: Block,
        block_info: BlockInfo,
        block_state: BlockState,
    ) -> Result<()> {
        let block_id = block.id();
        self.storage.commit_block(block.clone(), block_state)?;
        self.storage.save_block_info(block_info)?;
        self.head = Some(block);
        self.chain_state = ChainStateDB::new(
            self.storage.clone().into_super_arc(),
            Some(self.head_block().header().state_root()),
        );
        debug!("commit block {:?} success.", block_id);
        Ok(())
    }
}

impl ChainWriter for BlockChain {
    fn apply(&mut self, block: Block) -> Result<()> {
        if let Some(uncles) = block.uncles() {
            self.verify_uncles(uncles, &block.header)?;
        }
        self.apply_inner(block, true, None)
    }

    fn apply_without_execute(
        &mut self,
        block: Block,
        remote_chain_state: &dyn ChainStateReader,
    ) -> Result<()> {
        if let Some(uncles) = block.uncles() {
            self.verify_uncles(uncles, &block.header)?;
        }
        self.apply_inner(block, false, Some(remote_chain_state))
    }

    fn chain_state(&mut self) -> &dyn ChainState {
        &self.chain_state
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
