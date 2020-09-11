// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use consensus::Consensus;
use crypto::ed25519::Ed25519PublicKey;
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, Accumulator,
    AccumulatorTreeStore, MerkleAccumulator,
};
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateReader, ChainStateWriter};
use starcoin_statedb::ChainStateDB;
use starcoin_traits::{
    verify_block, ChainReader, ChainWriter, ConnectBlockError, ExcludedTxns, VerifyBlockField,
};
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
use starcoin_vm_types::on_chain_config::{
    Consensus as ConsensusConfig, EpochDataResource, EpochInfo, EpochResource, GlobalTimeOnChain,
    VMConfig,
};
use std::cmp::min;
use std::iter::Extend;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashSet, convert::TryInto, sync::Arc};
use storage::Store;

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

pub struct BlockChain {
    consensus: ConsensusStrategy,
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    head: Option<Block>,
    chain_state: ChainStateDB,
    storage: Arc<dyn Store>,
    uncles: HashSet<HashValue>,
    epoch: Option<EpochResource>,
}

impl BlockChain {
    pub fn new(
        consensus: ConsensusStrategy,
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
            consensus,
            txn_accumulator: info_2_accumulator(
                txn_accumulator_info.clone(),
                AccumulatorStoreType::Transaction,
                storage.clone().into_super_arc(),
            )?,
            block_accumulator: info_2_accumulator(
                block_accumulator_info.clone(),
                AccumulatorStoreType::Block,
                storage.clone().into_super_arc(),
            )?,
            head: Some(head),
            chain_state: ChainStateDB::new(storage.clone().into_super_arc(), Some(state_root)),
            storage,
            uncles: HashSet::new(),
            epoch: None,
        };
        chain.update_epoch_and_uncle_cache()?;
        Ok(chain)
    }

    pub fn init_empty_chain(consensus: ConsensusStrategy, storage: Arc<dyn Store>) -> Result<Self> {
        let txn_accumulator = MerkleAccumulator::new_empty(
            AccumulatorStoreType::Transaction,
            storage.clone().into_super_arc(),
        )?;
        let block_accumulator = MerkleAccumulator::new_empty(
            AccumulatorStoreType::Block,
            storage.clone().into_super_arc(),
        )?;
        let chain = Self {
            consensus,
            txn_accumulator,
            block_accumulator,
            head: None,
            chain_state: ChainStateDB::new(storage.clone().into_super_arc(), None),
            storage,
            uncles: HashSet::new(),
            epoch: None,
        };
        Ok(chain)
    }

    pub fn new_chain(&self, head_block_hash: HashValue) -> Result<Self> {
        let mut chain = Self::new(self.consensus, head_block_hash, self.storage.clone())?;
        chain.update_epoch_and_uncle_cache()?;
        Ok(chain)
    }

    pub fn current_epoch_uncles_size(&self) -> usize {
        self.uncles.len()
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        self.consensus
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
        epoch.start_number() <= block_header.number()
            && epoch.end_number() > block_header.number()
            && self.exist_block(block_header.parent_hash())
            && !self.uncles.contains(&block_header.id())
    }

    fn epoch_uncles(&self, epoch_resource: &EpochResource) -> Result<Vec<HashValue>> {
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

            if epoch_resource.start_number() > number || epoch_resource.end_number() <= number {
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

    fn create_block_template_inner(
        &self,
        author: AccountAddress,
        author_public_key: Option<Ed25519PublicKey>,
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
            author_public_key,
            self.consensus.now(),
            uncles,
        )?;
        let excluded_txns = opened_block.push_txns(user_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns))
    }

    pub fn get_on_chain_block_gas_limit(&self) -> Result<u64> {
        let account_state_reader = AccountStateReader::new(&self.chain_state);
        let vm_config = account_state_reader.get_on_chain_config::<VMConfig>()?;
        vm_config
            .map(|vm_config| vm_config.block_gas_limit)
            .ok_or_else(|| format_err!("read on chain config fail."))
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

    fn get_epoch_resource_by_number(&self, number: Option<BlockNumber>) -> Result<EpochResource> {
        if let Some(block) = self.block_with_number(number)? {
            let chain_state = ChainStateDB::new(
                self.storage.clone().into_super_arc(),
                Some(block.header().state_root()),
            );
            let account_reader = AccountStateReader::new(&chain_state);
            let epoch = account_reader
                .get_resource::<EpochResource>(genesis_address())?
                .ok_or_else(|| format_err!("Epoch is none."))?;
            Ok(epoch)
        } else {
            Err(format_err!("Block is none when query epoch resource."))
        }
    }
}

impl ChainReader for BlockChain {
    fn head_block(&self) -> Block {
        self.head.clone().expect("head block is none.")
    }

    fn current_header(&self) -> BlockHeader {
        self.head_block().header().clone()
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

    fn create_block_template(
        &self,
        author: AccountAddress,
        author_public_key: Option<Ed25519PublicKey>,
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
            author_public_key,
            previous_header,
            user_txns,
            uncles,
            block_gas_limit,
        )
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
        let block_info = self.storage.get_block_info(self.head_block().id())?;
        Ok(block_info.map_or(U256::zero(), |info| info.total_difficulty))
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
            let epoch = account_reader
                .get_resource::<EpochResource>(genesis_address())?
                .ok_or_else(|| format_err!("Epoch is none."))?;

            let epoch_data = account_reader
                .get_resource::<EpochDataResource>(genesis_address())?
                .ok_or_else(|| format_err!("Epoch is none."))?;

            let consensus_conf = account_reader
                .get_on_chain_config::<ConsensusConfig>()?
                .ok_or_else(|| format_err!("ConsensusConfig is none."))?;

            Ok(EpochInfo::new(&epoch, epoch_data, &consensus_conf))
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
}

impl BlockChain {
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
            verify_block!(
                VerifyBlockField::Header,
                self.head_block().id() == parent_hash,
                "Invalid block: Parent id mismatch."
            );
        }
        // do not check genesis block timestamp check
        if !header.is_genesis() {
            let pre_block = match self.get_block(parent_hash)? {
                Some(block) => block,
                None => {
                    return Err(ConnectBlockError::ParentNotExist(Box::new(header.clone())).into());
                }
            };

            verify_block!(
                VerifyBlockField::Header,
                pre_block.header().timestamp() < header.timestamp(),
                "Invalid block: block timestamp too old"
            );
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            verify_block!(
                VerifyBlockField::Header,
                header.timestamp() <= ALLOWED_FUTURE_BLOCKTIME + now,
                "Invalid block: block timestamp too new"
            );
        }

        verify_block!(
            VerifyBlockField::Header,
            header.gas_used <= header.gas_limit,
            "gas used {} in transaction is bigger than gas limit {}",
            header.gas_used,
            header.gas_limit
        );

        // TODO 最小值是否需要
        if let Err(err) = if is_uncle {
            let uncle_branch =
                BlockChain::new(self.consensus(), parent_hash, self.storage.clone())?;
            self.consensus.verify(&uncle_branch, epoch, header)
        } else {
            self.consensus.verify(self, epoch, header)
        } {
            return Err(
                ConnectBlockError::VerifyBlockFailed(VerifyBlockField::Consensus, err).into(),
            );
        };
        Ok(())
    }

    fn blocks_since(&self, epoch_start_number: BlockNumber) -> Result<HashSet<HashValue>> {
        let mut hashs = HashSet::new();
        let latest_number = self.current_header().number();

        let mut number = epoch_start_number;
        loop {
            if let Some(id) = self.block_accumulator.get_leaf(number)? {
                hashs.insert(id);
            } else {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!("Block accumulator leaf {:?} is none.", number),
                )
                .into());
            }

            number += 1;
            if number > latest_number {
                break;
            }
        }

        Ok(hashs)
    }

    fn check_common_ancestor(
        &self,
        header_id: HashValue,
        epoch_start_number: BlockNumber,
        blocks: &HashSet<HashValue>,
    ) -> Result<bool> {
        let mut result = false;
        let block_header = self.storage.get_block_header_by_hash(header_id)?;

        if let Some(block_header) = block_header {
            if blocks.contains(&header_id) && block_header.number >= epoch_start_number {
                result = true;
            }
        } else {
            return Err(ConnectBlockError::VerifyBlockFailed(
                VerifyBlockField::Uncle,
                format_err!("Uncle parent {:?} is none.", header_id),
            )
            .into());
        }
        Ok(result)
    }

    fn verify_uncles(&self, uncles: &[BlockHeader], header: &BlockHeader) -> Result<()> {
        verify_block!(
            VerifyBlockField::Uncle,
            uncles.len() <= MAX_UNCLE_COUNT_PER_BLOCK,
            "too many uncles {} in block {}",
            uncles.len(),
            header.id()
        );
        for uncle in uncles {
            verify_block!(
                VerifyBlockField::Uncle,
                uncle.number < header.number ,
               "uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}", uncle.number, header.number
            );
        }

        match header.uncle_hash {
            Some(uncle_hash) => {
                let calculated_hash = HashValue::sha3_256_of(&uncles.to_vec().encode()?);
                verify_block!(
                    VerifyBlockField::Uncle,
                    calculated_hash.eq(&uncle_hash),
                    "uncle hash in header is {},uncle hash calculated is {}",
                    uncle_hash,
                    calculated_hash
                );
            }
            None => {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!("Unexpect uncles, header's uncle hash is None"),
                )
                .into());
            }
        }

        let epoch_start_number = if let Some(epoch) = &self.epoch {
            if header.number() >= epoch.end_number() {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!(
                        "block number is {:?}, epoch end number is {:?}",
                        header.number(),
                        epoch.end_number(),
                    ),
                )
                .into());
            }
            epoch.start_number()
        } else {
            header.number()
        };

        for uncle in uncles {
            if self.uncles.contains(&uncle.id()) {
                debug!("uncle block exists in master,uncle id is {:?}", uncle.id(),);
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!("uncle block exists in master,uncle id is {:?}", uncle.id()),
                )
                .into());
            }
        }

        let blocks = self.blocks_since(epoch_start_number)?;
        for uncle in uncles {
            if !self.check_common_ancestor(uncle.parent_hash(), epoch_start_number, &blocks)? {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Uncle,
                    format_err!(
                        "can't find ancestor in master uncle id is {:?},epoch start number is {:?}",
                        uncle.id(),
                        epoch_start_number
                    ),
                )
                .into());
            }
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
        verify_block!(
            VerifyBlockField::Header,
            header.gas_used() <= header.gas_limit(),
            "invalid block: gas_used should not greater than gas_limit"
        );

        let mut switch_epoch = false;
        if !is_genesis {
            let account_reader = match state_reader {
                Some(state_reader) => AccountStateReader::new(state_reader),
                None => AccountStateReader::new(&self.chain_state),
            };
            let epoch_info = account_reader.epoch()?;
            self.verify_header(&header, false, &epoch_info)?;

            if header.number() == epoch_info.end_number() {
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
                let block_metadata = block.clone().into_metadata();
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
            executor::block_execute(&self.chain_state, txns.clone(), header.gas_limit())?
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
            let (accumulator_root, _first_leaf_idx) =
                self.txn_accumulator.append(&included_txn_info_hashes)?;
            accumulator_root
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
        let txn_accumulator_info: AccumulatorInfo = (&self.txn_accumulator).try_into()?;
        let block_accumulator_info: AccumulatorInfo = (&self.block_accumulator).try_into()?;
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
            self.storage.clone().into_super_arc(),
        )?;
        self.block_accumulator = info_2_accumulator(
            block_accumulator_info.clone(),
            AccumulatorStoreType::Block,
            self.storage.clone().into_super_arc(),
        )?;
        self.chain_state =
            ChainStateDB::new(self.storage.clone().into_super_arc(), Some(state_root));
        if self.epoch.is_some()
            && self
                .epoch
                .as_ref()
                .expect("epoch resource is none.")
                .end_number()
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

    fn verify_txns(
        &self,
        block_id: HashValue,
        txns: &[Transaction],
    ) -> Result<executor::BlockExecutedData> {
        let mut block_executed_data = executor::BlockExecutedData::default();

        let txn_infos = self.storage.get_block_transaction_infos(block_id)?;

        if txn_infos.len() != txns.len() {
            return Err(ConnectBlockError::VerifyBlockFailed(
                VerifyBlockField::Body,
                format_err!(
                    "txn infos len ({:?}) is not equals txns len ({:?}).",
                    txn_infos.len(),
                    txns.len()
                ),
            )
            .into());
        }
        let mut state_root = None;
        for i in 0..txns.len() {
            let id = txns[i].id();
            if id != txn_infos[i].transaction_hash() {
                return Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Body,
                    format_err!(
                        "txn {:?} is not match with txn_info ({:?}).",
                        id,
                        txn_infos[i]
                    ),
                )
                .into());
            }
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
    node_store: Arc<dyn AccumulatorTreeStore>,
) -> Result<MerkleAccumulator> {
    MerkleAccumulator::new_with_info(accumulator_info, store_type, node_store)
}
