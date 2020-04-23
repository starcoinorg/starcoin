// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_service::BlockChainCollection;
use actix::prelude::*;
use anyhow::{format_err, Error, Result};
use config::NodeConfig;
use crypto::HashValue;
use executor::block_executor::BlockExecutor;
use executor::executor::mock_create_account_txn;
use logger::prelude::*;
use network::get_unix_ts;
use once_cell::sync::Lazy;
use starcoin_accumulator::node::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_state_api::{ChainState, ChainStateReader};
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::TxPoolAsyncService;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use storage::Store;
use traits::Consensus;
use traits::{ChainReader, ChainWriter};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate, BLOCK_INFO_DEFAULT_ID},
    block_metadata::BlockMetadata,
    startup_info::ChainInfo,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    U512,
};

pub static DEFAULT_BLOCK_INFO: Lazy<BlockInfo> = Lazy::new(|| {
    BlockInfo::new(
        *BLOCK_INFO_DEFAULT_ID,
        *ACCUMULATOR_PLACEHOLDER_HASH,
        vec![],
        0,
        0,
        U512::zero(),
    )
});

pub struct BlockChain<C, S, P>
where
    C: Consensus,
    S: Store + 'static,
    P: TxPoolAsyncService + 'static,
{
    pub config: Arc<NodeConfig>,
    accumulator: MerkleAccumulator,
    head: Block,
    chain_state: ChainStateDB,
    phantom_c: PhantomData<C>,
    pub storage: Arc<S>,
    pub txpool: P,
    chain_info: ChainInfo,
    pub block_chain_collection: Arc<BlockChainCollection<C, S, P>>,
}

impl<C, S, P> BlockChain<C, S, P>
where
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    pub fn new(
        config: Arc<NodeConfig>,
        chain_info: ChainInfo,
        storage: Arc<S>,
        txpool: P,
        block_chain_collection: Arc<BlockChainCollection<C, S, P>>,
    ) -> Result<Self> {
        let head_block_hash = chain_info.get_head();
        let head = storage
            .get_block_by_hash(head_block_hash)?
            .ok_or(format_err!(
                "Can not find block by hash {}",
                head_block_hash
            ))?;
        // let block_info = match storage.clone().get_block_info(head_block_hash) {
        //     Ok(Some(block_info_1)) => block_info_1,
        //     Err(e) => {
        //         warn!("err : {:?}", e);
        //         DEFAULT_BLOCK_INFO.clone()
        //     }
        //     _ => DEFAULT_BLOCK_INFO.clone(),
        // };

        let state_root = head.header().state_root();
        let chain = Self {
            config: config.clone(),
            accumulator: MerkleAccumulator::new(
                chain_info.branch_id(),
                *ACCUMULATOR_PLACEHOLDER_HASH,
                vec![],
                0,
                0,
                storage.clone(),
            )
            .unwrap(),
            head,
            chain_state: ChainStateDB::new(storage.clone(), Some(state_root)),
            phantom_c: PhantomData,
            storage,
            txpool,
            chain_info,
            block_chain_collection,
        };
        Ok(chain)
    }

    pub fn save_block(&self, block: &Block) {
        if let Err(e) = self
            .storage
            .commit_branch_block(self.get_chain_info().branch_id(), block.clone())
        {
            warn!("err : {:?}", e);
        }
        debug!("commit block : {:?}", block.header().id());
    }

    fn get_block_info(&self, block_id: HashValue) -> BlockInfo {
        let block_info = match self.storage.get_block_info(block_id) {
            Ok(Some(block_info_1)) => block_info_1,
            Err(e) => {
                warn!("err : {:?}", e);
                DEFAULT_BLOCK_INFO.clone()
            }
            _ => DEFAULT_BLOCK_INFO.clone(),
        };
        block_info
    }
    pub fn save_block_info(&self, block_info: BlockInfo) {
        if let Err(e) = self.storage.save_block_info(block_info) {
            warn!("err : {:?}", e);
        }
    }

    fn gen_tx_for_test(&self) {
        let tx = mock_create_account_txn();
        let txpool = self.txpool.clone();
        Arbiter::spawn(async move {
            debug!("gen_tx_for_test call txpool.");
            txpool.add(tx.try_into().unwrap()).await.unwrap();
        });
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
            count = count + 1;
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
        let mut txns = user_txns
            .iter()
            .cloned()
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        txns.push(Transaction::BlockMetadata(BlockMetadata::new(
            previous_header.id(),
            timestamp,
            author,
            auth_key_prefix.clone(),
        )));
        let chain_state =
            ChainStateDB::new(self.storage.clone(), Some(previous_header.state_root()));
        // let block_info = self.get_block_info(previous_header.id());
        let accumulator = MerkleAccumulator::new(
            self.chain_info.branch_id(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            self.storage.clone(),
        )?;

        let (accumulator_root, state_root, _) =
            BlockExecutor::block_execute(&chain_state, &accumulator, txns, true)?;

        Ok(BlockTemplate::new(
            previous_header.id(),
            timestamp,
            previous_header.number() + 1,
            author,
            auth_key_prefix,
            accumulator_root,
            state_root,
            0,
            0,
            user_txns.into(),
        ))
    }

    pub fn fork(&self, block_header: &BlockHeader) -> Option<ChainInfo> {
        if self.exist_block(block_header.parent_hash()) {
            Some(if self.head.header().id() == block_header.parent_hash() {
                self.get_chain_info()
            } else {
                ChainInfo::new(
                    Some(self.get_chain_info().branch_id()),
                    block_header.parent_hash(),
                    block_header,
                )
            })
        } else {
            None
        }
    }

    pub fn get_branch_id(&self, number: BlockNumber) -> Option<HashValue> {
        self.block_chain_collection
            .get_branch_id(&self.chain_info.branch_id(), number)
    }

    pub fn update_head(&mut self, latest_block: BlockHeader) {
        self.chain_info.update_head(latest_block)
    }
}

impl<C, S, P> ChainReader for BlockChain<C, S, P>
where
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    fn head_block(&self) -> Block {
        self.head.clone()
    }

    fn current_header(&self) -> BlockHeader {
        self.head.header().clone()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        assert!(self.exist_block(hash));
        Ok(Some(
            self.get_block(hash).unwrap().unwrap().header().clone(),
        ))
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        if let Some(branch_id) = self.get_branch_id(number) {
            self.storage.get_header_by_branch_number(branch_id, number)
        } else {
            Ok(None)
        }
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        if let Some(branch_id) = self.get_branch_id(number) {
            self.storage.get_block_by_branch_number(branch_id, number)
        } else {
            warn!("branch id not found.");
            Ok(None)
        }
    }

    fn get_blocks_by_number(&self, number: BlockNumber, count: u64) -> Result<Vec<Block>, Error> {
        let mut block_vec = vec![];
        let mut temp_number = number;
        if number == 0 as u64 {
            temp_number = self.current_header().number();
        }
        if let Some(branch_id) = self.get_branch_id(temp_number) {
            let mut tmp_count = count;
            let mut current_num = temp_number;

            loop {
                match self
                    .storage
                    .get_block_by_branch_number(branch_id, current_num)
                {
                    Ok(block) => {
                        if block.is_some() {
                            block_vec.push(block.unwrap());
                        }
                    }
                    Err(_e) => {
                        error!(
                            "get block by branch {:?} number{:?} err.",
                            branch_id, current_num
                        );
                    }
                }
                if current_num == 0 || tmp_count == 1 {
                    break;
                }
                current_num = current_num - 1;
                tmp_count = tmp_count - 1;
            }
        } else {
            warn!("branch id not found.");
        }
        Ok(block_vec)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        let block = self.storage.get_block_by_hash(hash);
        match block {
            Ok(tmp) => match tmp {
                Some(b) => {
                    if let Ok(Some(block_header)) = self.get_header_by_number(b.header().number()) {
                        if block_header.id() == b.header().id() {
                            return Ok(Some(b));
                        } else {
                            warn!("block is miss match {:?} : {:?}", hash, block_header.id());
                        }
                    }
                }
                None => {
                    warn!("Get block from storage return none.");
                }
            },
            Err(e) => {
                warn!("err:{:?}", e);
            }
        }

        return Ok(None);
    }

    fn get_block_transactions(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        let mut txn_vec = vec![];
        match self.storage.get_block_transactions(block_id) {
            Ok(vec_hash) => {
                for hash in vec_hash {
                    match self.get_transaction_info(hash) {
                        Ok(Some(transaction_info)) => txn_vec.push(transaction_info),
                        _ => error!("get transaction info error: {:?}", hash),
                    }
                }
            }
            _ => {}
        }
        Ok(txn_vec)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
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
        let previous_header = self.get_header(block_id)?.unwrap();
        self.create_block_template_inner(author, auth_key_prefix, previous_header, user_txns)
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        &self.chain_state
    }

    fn gen_tx(&self) -> Result<()> {
        self.gen_tx_for_test();
        Ok(())
    }

    fn get_chain_info(&self) -> ChainInfo {
        self.chain_info.clone()
    }

    fn get_block_info(&self, block_id: Option<HashValue>) -> Result<Option<BlockInfo>> {
        let id = match block_id {
            Some(hash) => hash,
            None => self.current_header().id(),
        };
        assert!(self.exist_block(id));
        self.storage.get_block_info(id)
    }

    fn get_total_difficulty(&self) -> Result<U512> {
        let block_info = self.storage.get_block_info(self.head.header().id())?;
        Ok(block_info.map_or(U512::zero(), |info| info.total_difficulty))
    }

    fn exist_block(&self, block_id: HashValue) -> bool {
        if let Ok(Some(_)) = self.get_block(block_id) {
            true
        } else {
            false
        }
    }
}

impl<C, S, P> ChainWriter for BlockChain<C, S, P>
where
    C: Consensus,
    S: Store,
    P: TxPoolAsyncService,
{
    fn apply(&mut self, block: Block) -> Result<bool> {
        let header = block.header();
        debug!(
            "Apply block {:?} to {:?}",
            block.header(),
            self.head.header()
        );
        //TODO custom verify macro
        assert_eq!(self.head.header().id(), block.header().parent_hash());

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
            .map(|user_txn| Transaction::UserTransaction(user_txn))
            .collect::<Vec<Transaction>>();
        let block_metadata = header.clone().into_metadata();

        txns.push(Transaction::BlockMetadata(block_metadata));

        let exe_begin_time = get_unix_ts();
        let (accumulator_root, state_root, vec_transaction_info) =
            BlockExecutor::block_execute(chain_state, &self.accumulator, txns.clone(), false)?;
        let exe_end_time = get_unix_ts();
        debug!("exe used time: {}", (exe_end_time - exe_begin_time));
        assert_eq!(
            block.header().state_root(),
            state_root,
            "verify block:{:?} state_root fail.",
            block.header().id()
        );

        let total_difficulty = {
            let pre_total_difficulty = self
                .get_block_info(block.header().parent_hash())
                .total_difficulty;
            pre_total_difficulty + header.difficult().into()
        };

        let block_info = BlockInfo::new(
            header.id().clone(),
            accumulator_root,
            self.accumulator.get_frozen_subtree_roots()?,
            self.accumulator.num_leaves(),
            self.accumulator.num_nodes(),
            total_difficulty,
        );
        // save block's transaction relationship and save transaction
        self.save(header.id().clone(), txns.clone())?;
        self.storage.save_transaction_infos(vec_transaction_info)?;
        let commit_begin_time = get_unix_ts();
        self.commit(block.clone(), block_info)?;
        let commit_end_time = get_unix_ts();
        debug!(
            "commit used time: {}",
            (commit_end_time - commit_begin_time)
        );
        Ok(true)
    }

    fn commit(&mut self, block: Block, block_info: BlockInfo) -> Result<()> {
        let block_id = block.header().id();
        self.save_block(&block);
        self.chain_info.update_head(block.header().clone());
        self.head = block;
        self.accumulator = MerkleAccumulator::new(
            self.chain_info.branch_id(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            self.storage.clone(),
        )
        .unwrap();
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
        self.storage
            .save_block_transactions(block_id, txn_id_vec)
            .unwrap();
        // save transactions
        self.storage.save_transaction_batch(transactions).unwrap();
        Ok(())
    }

    fn chain_state(&mut self) -> &dyn ChainState {
        &self.chain_state
    }
}
