// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use crate::message::{ChainRequest, ChainResponse};
use crate::starcoin_chain_state::StarcoinChainState;
use actix::prelude::*;
use anyhow::{Error, Result};
use chain_state::ChainState;
use config::NodeConfig;
use consensus::{Consensus, ConsensusHeader};
use crypto::{hash::CryptoHash, HashValue};
use executor::TransactionExecutor;
use futures_locks::RwLock;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use traits::{ChainReader, ChainService, ChainStateReader, ChainWriter};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct ChainServiceImpl<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    config: Arc<NodeConfig>,
    head: BlockChain<E, C>,
    branches: Vec<BlockChain<E, C>>,
    storage: Arc<StarcoinStorage>,
}

impl<E, C> ChainServiceImpl<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    pub fn new(config: Arc<NodeConfig>, storage: Arc<StarcoinStorage>) -> Result<Self> {
        let latest_header = storage.block_store.get_latest_block_header()?;
        let head = BlockChain::new(config.clone(), storage.clone(), latest_header)?;
        let branches = Vec::new();
        Ok(Self {
            config,
            head,
            branches,
            storage,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> BlockChain<E, C> {
        unimplemented!()
    }

    pub fn state_at(&self, root: HashValue) -> StarcoinChainState {
        unimplemented!()
    }

    fn select_head(&mut self) {
        //select head branch;
        todo!()
    }
}

impl<E, C> ChainService for ChainServiceImpl<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block) -> Result<()> {
        let header = block.header();
        let mut branch = self.find_or_fork(&header);
        branch.apply(block)?;
        self.select_head();
        todo!()
    }
}

impl<E, C> ChainReader for ChainServiceImpl<E, C>
where
    E: TransactionExecutor,
    C: Consensus,
{
    fn head_block(&self) -> Block {
        self.head.head_block()
    }

    fn current_header(&self) -> BlockHeader {
        self.head.current_header()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        unimplemented!()
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        unimplemented!()
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        unimplemented!()
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>> {
        unimplemented!()
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>> {
        unimplemented!()
    }

    fn create_block_template(&self) -> Result<BlockTemplate> {
        self.head.create_block_template()
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        unimplemented!()
    }
}
