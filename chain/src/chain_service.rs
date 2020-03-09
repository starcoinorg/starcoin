// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use crate::message::{ChainRequest, ChainResponse};
use actix::prelude::*;
use anyhow::{Error, Result};
use config::NodeConfig;
use consensus::{Consensus, ConsensusHeader};
use crypto::{hash::CryptoHash, HashValue};
use executor::TransactionExecutor;
use futures_locks::RwLock;
use logger::prelude::*;
use network::network::NetworkAsyncService;
use starcoin_statedb::ChainStateDB;
use state_tree::StateNodeStore;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, BlockStorageOp, StarcoinStorage, StarcoinStorageOp};
use traits::{
    ChainAsyncService, ChainReader, ChainService, ChainStateReader, ChainWriter, TxPoolAsyncService,
};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    system_events::SystemEvents,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo, TransactionStatus},
};

pub struct ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: StateNodeStore + BlockStorageOp + 'static,
{
    config: Arc<NodeConfig>,
    head: BlockChain<E, C, S, P>,
    branches: Vec<BlockChain<E, C, S, P>>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService<P>>,
    txpool: P,
}

impl<E, C, P, S> ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: StateNodeStore + BlockStorageOp,
{
    pub fn new(
        config: Arc<NodeConfig>,
        storage: Arc<S>,
        network: Option<NetworkAsyncService<P>>,
        txpool: P,
    ) -> Result<Self> {
        let latest_header = storage.get_latest_block_header()?;
        let head = BlockChain::new(
            config.clone(),
            storage.clone(),
            latest_header.map(|header| header.id()),
            txpool.clone(),
        )?;
        let branches = Vec::new();
        Ok(Self {
            config,
            head,
            branches,
            storage,
            network,
            txpool,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Option<BlockChain<E, C, S, P>> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let block_in_head = self.head.get_block(header.parent_hash()).unwrap();
        match block_in_head {
            Some(block) => {
                return Some(
                    BlockChain::new(
                        self.config.clone(),
                        self.storage.clone(),
                        Some(header.parent_hash()),
                        self.txpool.clone(),
                    )
                    .unwrap(),
                );
            }
            None => {
                for branch in &self.branches {
                    if let Ok(Some(block)) = branch.get_block(header.parent_hash()) {
                        return Some(
                            BlockChain::new(
                                self.config.clone(),
                                self.storage.clone(),
                                Some(header.parent_hash()),
                                self.txpool.clone(),
                            )
                            .unwrap(),
                        );
                    }
                }
            }
        }

        None
    }

    pub fn state_at(&self, root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<E, C, S, P>) {
        let new_branch_parent_hash = new_branch.current_header().parent_hash();
        let mut need_broadcast = false;
        let block = new_branch.head_block();
        if new_branch_parent_hash == self.head.current_header().id() {
            //1. update head branch
            self.head = new_branch;
            need_broadcast = true;

            //delete txpool
            let mut enacted = Vec::new();
            enacted.push(block.header().id());
            let mut retracted = Vec::new();
            self.commit_2_txpool(enacted, retracted);
        } else {
            //2. update branches
            let mut update_branch_flag = false;
            for mut branch in &self.branches {
                if new_branch_parent_hash == branch.current_header().id() {
                    if new_branch.current_header().number() > self.head.current_header().number() {
                        //3. change head
                        //rollback txpool
                        let (enacted, retracted) = Self::find_ancestors(&new_branch, &self.head);

                        branch = &self.head;
                        self.head = BlockChain::new(
                            self.config.clone(),
                            self.storage.clone(),
                            Some(new_branch.current_header().id()),
                            self.txpool.clone(),
                        )
                        .unwrap();

                        self.commit_2_txpool(enacted, retracted);

                        need_broadcast = true;
                    } else {
                        branch = &new_branch;
                    }
                    update_branch_flag = true;
                    break;
                }
            }

            if !update_branch_flag {
                self.branches.push(new_branch);
            }
        }

        if need_broadcast {
            if let Some(network) = self.network.clone() {
                Arbiter::spawn(async move {
                    info!("broadcast system event : {:?}", block.header().id());
                    network
                        .clone()
                        .broadcast_system_event(SystemEvents::NewHeadBlock(block))
                        .await;
                });
            };
        }
    }

    fn commit_2_txpool(&self, enacted: Vec<HashValue>, retracted: Vec<HashValue>) {
        let txpool = self.txpool.clone();
        Arbiter::spawn(async move {
            txpool.chain_new_blocks(enacted, retracted).await.unwrap();
        });
    }

    fn find_ancestors(
        block_chain: &BlockChain<E, C, S, P>,
        head_chain: &BlockChain<E, C, S, P>,
    ) -> (Vec<HashValue>, Vec<HashValue>) {
        let mut enacted: Vec<HashValue> = Vec::new();
        let mut retracted: Vec<HashValue> = Vec::new();
        //todo:from db
        (enacted, retracted)
    }
}

impl<E, C, P, S> ChainService for ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: StateNodeStore + BlockStorageOp,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block) -> Result<()> {
        if self
            .storage
            .get_block_by_hash(block.header().id())?
            .is_none()
            && self
                .storage
                .get_block_by_hash(block.header().parent_hash())?
                .is_some()
        {
            let header = block.header();
            let mut branch = self.find_or_fork(&header).unwrap();
            branch.apply(block.clone())?;
            self.select_head(branch);
        }
        Ok(())
    }

    fn get_head_branch(&self) -> HashValue {
        self.head.current_header().id()
    }
}

impl<E, C, P, S> ChainReader for ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: StateNodeStore + BlockStorageOp,
{
    fn head_block(&self) -> Block {
        self.head.head_block()
    }

    fn current_header(&self) -> BlockHeader {
        self.head.current_header()
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.head.get_header(hash)
    }

    fn get_header_by_number(&self, number: u64) -> Result<Option<BlockHeader>> {
        self.head.get_header_by_number(number)
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.head.get_block_by_number(number)
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        self.head.get_block(hash)
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>> {
        self.head.get_transaction(hash)
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>> {
        self.head.get_transaction_info(hash)
    }

    fn create_block_template(&self, txns: Vec<SignedUserTransaction>) -> Result<BlockTemplate> {
        self.head.create_block_template(txns)
    }

    fn chain_state_reader(&self) -> &dyn ChainStateReader {
        self.head.chain_state_reader()
    }

    fn gen_tx(&self) -> Result<()> {
        self.head.gen_tx()
    }
}
