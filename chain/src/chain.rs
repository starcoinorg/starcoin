// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Error, Result};
use config::NodeConfig;
use crypto::HashValue;
use executor::block_executor::BlockExecutor;
use logger::prelude::*;
use network::get_unix_ts;
use starcoin_accumulator::{Accumulator, AccumulatorTreeStore, MerkleAccumulator};
use starcoin_state_api::{ChainState, ChainStateReader};
use starcoin_statedb::ChainStateDB;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::Store;
use traits::Consensus;
use traits::{ChainReader, ChainWriter};
use types::{
    account_address::AccountAddress,
    accumulator_info::AccumulatorInfo,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    block_metadata::BlockMetadata,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    U512,
};

pub struct BlockChain<C, S>
where
    C: Consensus,
    S: Store + 'static,
{
    config: Arc<NodeConfig>,
    txn_accumulator: MerkleAccumulator,
    block_accumulator: MerkleAccumulator,
    head: Block,
    chain_state: ChainStateDB,
    storage: Arc<S>,
    phantom: PhantomData<C>,
}

impl<C, S> BlockChain<C, S>
where
    C: Consensus,
    S: Store,
{
    pub fn new(
        config: Arc<NodeConfig>,
        head_block_hash: HashValue,
        storage: Arc<S>,
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
        let chain = Self {
            config,
            txn_accumulator: info_2_accumulator(txn_accumulator_info, storage.clone())?,
            block_accumulator: info_2_accumulator(block_accumulator_info.clone(), storage.clone())?,
            head,
            chain_state: ChainStateDB::new(storage.clone(), Some(state_root)),
            storage,
            phantom: PhantomData,
        };
        Ok(chain)
    }

    pub fn new_chain(&self, head_block_hash: HashValue) -> Result<Self> {
        Self::new(self.config.clone(), head_block_hash, self.storage.clone())
    }

    pub fn save_block(&self, block: &Block) {
        if let Err(e) = self.storage.commit_block(block.clone()) {
            warn!("err : {:?}", e);
        }
        debug!("commit block : {:?}", block.header().id());
    }

    fn get_block_info(&self, block_id: HashValue) -> Result<BlockInfo> {
        Ok(self
            .storage
            .get_block_info(block_id)?
            .ok_or_else(|| format_err!("Can not find block info by hash {}", block_id))?)
    }
    pub fn save_block_info(&self, block_info: BlockInfo) {
        if let Err(e) = self.storage.save_block_info(block_info) {
            warn!("err : {:?}", e);
        }
    }

    pub fn latest_blocks(&self, size: u64) {
        let mut count = 0;
        let mut last = self.head.header().clone();
        loop {
            info!(
                "block chain :: number : {} , block_id : {:?}",
                last.number(),
                last.id()
            );
            if last.number() == 0 || count >= size {
                break;
            }
            last = self
                .get_header(last.parent_hash())
                .unwrap()
                .unwrap()
                .clone();
            count += 1;
        }
    }

    pub fn create_block_template_inner(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        previous_header: BlockHeader,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        //TODO calculate gas limit etc.
        let txns = user_txns
            .iter()
            .cloned()
            .map(Transaction::UserTransaction)
            .collect::<Vec<Transaction>>();

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let chain_state =
            ChainStateDB::new(self.storage.clone(), Some(previous_header.state_root()));
        let block_info = self.get_block_info(previous_header.id())?;
        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new(
            *txn_accumulator_info.get_accumulator_root(),
            txn_accumulator_info.get_frozen_subtree_roots().clone(),
            txn_accumulator_info.get_num_leaves(),
            txn_accumulator_info.get_num_nodes(),
            self.storage.clone(),
        )?;
        let block_gas_limit = self.config.miner.block_gas_limit;
        let (accumulator_root, state_root, txn_infos) = BlockExecutor::block_execute(
            &chain_state,
            &txn_accumulator,
            txns,
            BlockMetadata::new(
                previous_header.id(),
                timestamp,
                author,
                auth_key_prefix.clone(),
            ),
            block_gas_limit,
            true,
        )?;
        let block_gas_used = txn_infos.iter().fold(0u64, |acc, i| acc + i.gas_used());
        let included_user_txns = user_txns
            .into_iter()
            .take(txn_infos.len() - 1)
            .collect::<Vec<_>>();
        Ok(BlockTemplate::new(
            previous_header.id(),
            *block_info
                .get_block_accumulator_info()
                .get_accumulator_root(),
            timestamp,
            previous_header.number() + 1,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            block_gas_used,
            block_gas_limit,
            included_user_txns.into(),
        ))
    }

    fn find_block_by_number(&self, number: u64) -> Result<HashValue> {
        self.block_accumulator
            .get_leaf(number)?
            .ok_or_else(|| format_err!("Can not find block by number {}", number))
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
                    "block is miss match {:?} : {:?}",
                    block_id,
                    block_header.id()
                );
            }
        }

        Ok(false)
    }

    pub fn append_pivot(
        &mut self,
        block_id: HashValue,
        block_accumulator_info: AccumulatorInfo,
    ) -> Result<()> {
        self.block_accumulator.append(&[block_id])?;
        self.block_accumulator.flush()?;

        let pivot_block_accumulator_info: AccumulatorInfo = (&self.block_accumulator).try_into()?;
        assert_eq!(block_accumulator_info, pivot_block_accumulator_info);
        debug!("save pivot {:?} succ.", block_id);

        Ok(())
    }

    pub fn get_storage(&self) -> Arc<S> {
        self.storage.clone()
    }
}

impl<C, S> ChainReader for BlockChain<C, S>
where
    C: Consensus,
    S: Store,
{
    fn head_block(&self) -> Block {
        self.head.clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.head.header().clone()
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
                debug!("Get block from storage return none.");
            }
        }

        Ok(None)
    }

    fn get_block_transactions(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        let mut txn_vec = vec![];
        let vec_hash = self.storage.get_block_transactions(block_id)?;
        for hash in vec_hash {
            if let Some(transaction_info) = self.get_transaction_info(hash)? {
                txn_vec.push(transaction_info);
            }
        }
        Ok(txn_vec)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>> {
        self.storage.get_transaction_info(hash)
    }

    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        assert!(self.exist_block(block_id));
        let previous_header = self
            .get_header(block_id)?
            .ok_or_else(|| format_err!("Can find block header by {:?}", block_id))?;
        self.create_block_template_inner(author, auth_key_prefix, previous_header, user_txns)
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

    fn get_total_difficulty(&self) -> Result<U512> {
        let block_info = self.storage.get_block_info(self.head.header().id())?;
        Ok(block_info.map_or(U512::zero(), |info| info.total_difficulty))
    }

    fn exist_block(&self, block_id: HashValue) -> bool {
        if let Ok(Some(header)) = self.storage.get_block_header_by_hash(block_id) {
            if let Ok(exist) = self.block_exist_by_number(block_id, header.number()) {
                return exist;
            }
        }
        false
    }
}

impl<C, S> ChainWriter for BlockChain<C, S>
where
    C: Consensus,
    S: Store,
{
    fn apply(&mut self, block: Block) -> Result<bool> {
        let header = block.header();
        debug!("Apply block {:?} to {:?}", header, self.head.header());
        //TODO custom verify macro
        assert_eq!(self.head.header().id(), header.parent_hash());

        ensure!(
            block.header().gas_used() <= block.header().gas_limit(),
            "invalid block: gas_used should not greater than gas_limit"
        );
        let apply_begin_time = get_unix_ts();
        if let Err(e) = C::verify_header(self.config.clone(), self, header) {
            error!("err: {:?}", e);
            return Ok(false);
        }
        let verify_end_time = get_unix_ts();
        debug!("verify used time: {}", (verify_end_time - apply_begin_time));

        let chain_state = &self.chain_state;
        let mut txns = block
            .transactions()
            .iter()
            .cloned()
            .map(Transaction::UserTransaction)
            .collect::<Vec<Transaction>>();
        let block_metadata = header.clone().into_metadata();

        // txns.push(Transaction::BlockMetadata(block_metadata));

        let exe_begin_time = get_unix_ts();

        let (_, state_root, vec_transaction_info) = BlockExecutor::block_execute(
            chain_state,
            &self.txn_accumulator,
            txns.clone(),
            block_metadata.clone(),
            block.header().gas_limit(),
            false,
        )?;

        let exe_end_time = get_unix_ts();
        debug!("exe used time: {}", (exe_end_time - exe_begin_time));
        assert_eq!(
            block.header().state_root(),
            state_root,
            "verify block:{:?} state_root fail.",
            block.header().id()
        );

        let block_gas_used = vec_transaction_info
            .iter()
            .fold(0u64, |acc, i| acc + i.gas_used());
        ensure!(
            block_gas_used == block.header().gas_used(),
            "invalid block: gas_used is not match"
        );

        // +1 because block_meta_data is not included in block.
        ensure!(
            vec_transaction_info.len() == txns.len() + 1,
            "invalid txn num in the block"
        );
        // push the extra meta txn to save.
        txns.push(Transaction::BlockMetadata(block_metadata));

        let total_difficulty = {
            let pre_total_difficulty = self
                .get_block_info(block.header().parent_hash())?
                .total_difficulty;
            pre_total_difficulty + header.difficulty().into()
        };

        let new_block_info_begin_time = get_unix_ts();

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
        let new_block_info_end_time = get_unix_ts();
        debug!(
            "new block info used time: {}",
            (new_block_info_end_time - new_block_info_begin_time)
        );
        // save block's transaction relationship and save transaction
        self.save(header.id(), txns)?;
        let save_block_end_time = get_unix_ts();
        debug!(
            "save block used time: {}",
            (save_block_end_time - new_block_info_end_time)
        );
        self.storage.save_transaction_infos(vec_transaction_info)?;
        let commit_begin_time = get_unix_ts();
        debug!(
            "new transaction info used time: {}",
            (commit_begin_time - save_block_end_time)
        );
        self.commit(block.clone(), block_info)?;
        let commit_end_time = get_unix_ts();
        debug!(
            "commit used time: {}",
            (commit_end_time - commit_begin_time)
        );
        Ok(true)
    }

    fn commit(&mut self, block: Block, block_info: BlockInfo) -> Result<()> {
        let block_id = block.id();
        self.save_block(&block);
        self.head = block;
        self.save_block_info(block_info);
        self.chain_state =
            ChainStateDB::new(self.storage.clone(), Some(self.head.header().state_root()));
        debug!("save block {:?} succ.", block_id);
        Ok(())
    }

    fn save(&mut self, block_id: HashValue, transactions: Vec<Transaction>) -> Result<()> {
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

    fn chain_state(&mut self) -> &dyn ChainState {
        &self.chain_state
    }
}

pub(crate) fn info_2_accumulator(
    accumulator_info: AccumulatorInfo,
    node_store: Arc<dyn AccumulatorTreeStore>,
) -> Result<MerkleAccumulator> {
    MerkleAccumulator::new(
        *accumulator_info.get_accumulator_root(),
        accumulator_info.get_frozen_subtree_roots().clone(),
        accumulator_info.get_num_leaves(),
        accumulator_info.get_num_nodes(),
        node_store,
    )
}
