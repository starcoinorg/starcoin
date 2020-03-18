// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use actix::prelude::*;
use anyhow::Result;
use config::NodeConfig;
use consensus::Consensus;
use crypto::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::network::NetworkAsyncService;
use starcoin_statedb::ChainStateDB;
use std::sync::Arc;
use storage::BlockChainStore;
use traits::{ChainReader, ChainService, ChainWriter, TxPoolAsyncService};
use types::{
    block::{Block, BlockHeader, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
    U256,
};

pub struct ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: BlockChainStore + 'static,
{
    config: Arc<NodeConfig>,
    master: BlockChain<E, C, S, P>,
    branches: Vec<BlockChain<E, C, S, P>>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService>,
    txpool: P,
}

impl<E, C, P, S> ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: BlockChainStore,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<S>,
        network: Option<NetworkAsyncService>,
        txpool: P,
    ) -> Result<Self> {
        let master = BlockChain::new(
            config.clone(),
            startup_info.head,
            storage.clone(),
            txpool.clone(),
        )?;
        let mut branches = Vec::new();
        for branch_info in startup_info.branches {
            branches.push(BlockChain::new(
                config.clone(),
                branch_info,
                storage.clone(),
                txpool.clone(),
            )?)
        }
        Ok(Self {
            config,
            master,
            branches,
            storage,
            network,
            txpool,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Option<BlockChain<E, C, S, P>> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let exist_in_head = self.master.exist_block(&header.parent_hash());
        if exist_in_head {
            return Some(
                BlockChain::new(
                    self.config.clone(),
                    self.master.fork_chain_info(&header.parent_hash()),
                    self.storage.clone(),
                    self.txpool.clone(),
                )
                .unwrap(),
            );
        } else {
            for branch in &self.branches {
                if branch.exist_block(&header.parent_hash()) {
                    return Some(
                        BlockChain::new(
                            self.config.clone(),
                            branch.fork_chain_info(&header.parent_hash()),
                            self.storage.clone(),
                            self.txpool.clone(),
                        )
                        .unwrap(),
                    );
                }
            }
        }

        None
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<E, C, S, P>) {
        let new_branch_parent_hash = new_branch.current_header().parent_hash();
        let mut need_broadcast = false;
        let block = new_branch.head_block();
        if new_branch_parent_hash == self.master.current_header().id() {
            debug!("head branch.");
            //1. update head branch
            self.master = new_branch;
            need_broadcast = true;

            //delete txpool
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            enacted.append(&mut block.transactions().clone().to_vec());
            let retracted = Vec::new();
            self.commit_2_txpool(enacted, retracted);
        } else {
            //2. update branches
            let mut update_branch_flag = false;
            let mut index = 0;
            for branch in &self.branches {
                index = index + 1;
                if new_branch_parent_hash == branch.current_header().id() {
                    if new_branch.current_header().number() > self.master.current_header().number()
                    {
                        debug!("rollback branch.");
                        //3. change head
                        //rollback txpool
                        let (enacted, retracted) = self.find_ancestors(&new_branch, &self.master);

                        self.branches.insert(
                            index - 1,
                            BlockChain::new(
                                self.config.clone(),
                                self.master.get_chain_info(),
                                self.storage.clone(),
                                self.txpool.clone(),
                            )
                            .unwrap(),
                        );
                        self.master = BlockChain::new(
                            new_branch.config.clone(),
                            new_branch.get_chain_info(),
                            new_branch.storage.clone(),
                            new_branch.txpool.clone(),
                        )
                        .unwrap();

                        self.commit_2_txpool(enacted, retracted);

                        need_broadcast = true;
                    } else {
                        debug!("replace branch.");
                        self.branches.insert(
                            index - 1,
                            BlockChain::new(
                                new_branch.config.clone(),
                                new_branch.get_chain_info(),
                                new_branch.storage.clone(),
                                new_branch.txpool.clone(),
                            )
                            .unwrap(),
                        );
                    }
                    update_branch_flag = true;
                    break;
                }
            }

            if !update_branch_flag {
                debug!("update branch.");
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
                        .await
                        .expect("broadcast new head block failed.");
                });
            };
        }
    }

    fn commit_2_txpool(
        &self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) {
        let txpool = self.txpool.clone();
        Arbiter::spawn(async move {
            if let Err(e) = txpool.rollback(enacted, retracted).await {
                warn!("rollback err : {:?}", e);
            }
        });
    }

    fn find_ancestors_in_memory(
        new_branch: &BlockChain<E, C, S, P>,
        head: &BlockChain<E, C, S, P>,
    ) -> Option<HashValue> {
        let mut begin_number = if new_branch.chain_info.size() > head.chain_info.size() {
            head.chain_info.size() as u64
        } else {
            new_branch.chain_info.size() as u64
        };

        debug!(
            "find_ancestors_in_memory:{}, {} , {}",
            new_branch.chain_info.size(),
            head.chain_info.size(),
            begin_number
        );

        let mut common_ancestor = None;
        loop {
            debug!(
                "number {}, block1 {:?}, block2 {:?}",
                (begin_number - 1),
                new_branch
                    .chain_info
                    .get_hash_by_number(begin_number - 1)
                    .unwrap(),
                head.chain_info
                    .get_hash_by_number(begin_number - 1)
                    .unwrap()
            );
            if new_branch
                .chain_info
                .get_hash_by_number(begin_number - 1)
                .unwrap()
                == head
                    .chain_info
                    .get_hash_by_number(begin_number - 1)
                    .unwrap()
            {
                common_ancestor = Some(
                    new_branch
                        .chain_info
                        .get_hash_by_number(begin_number - 1)
                        .unwrap(),
                );
                break;
            }

            begin_number = begin_number - 1;

            if begin_number == 0 {
                break;
            }
        }

        common_ancestor
    }

    fn find_ancestors(
        &self,
        new_branch: &BlockChain<E, C, S, P>,
        head: &BlockChain<E, C, S, P>,
    ) -> (Vec<SignedUserTransaction>, Vec<SignedUserTransaction>) {
        let mut enacted: Vec<Block> = Vec::new();
        let mut retracted: Vec<Block> = Vec::new();
        let ancestor =
            Self::find_ancestors_in_memory(new_branch, head).expect("common ancestor is none.");
        let block_enacted = &new_branch.current_header().parent_hash();
        let block_retracted = &self.master.current_header().parent_hash();
        let mut block_enacted_tmp = block_enacted.clone();

        debug!("ancestor block is : {:?}", ancestor);
        loop {
            if block_enacted_tmp == ancestor {
                break;
            };
            debug!("get block 1 {:?}.", block_enacted_tmp);
            let block_tmp = self
                .storage
                .get_block(block_enacted_tmp.clone())
                .unwrap()
                .expect("block is none 1.");
            block_enacted_tmp = block_tmp.header().parent_hash();
            enacted.push(block_tmp);
        }

        let mut block_retracted_tmp = block_retracted.clone();
        loop {
            if block_retracted_tmp == ancestor {
                break;
            };
            debug!("get block 2 {:?}.", block_retracted_tmp);
            let block_tmp = self
                .storage
                .get_block(block_retracted_tmp)
                .unwrap()
                .expect("block is none 2.");
            block_retracted_tmp = block_tmp.header().parent_hash();
            retracted.push(block_tmp);
        }
        retracted.reverse();
        enacted.reverse();
        let mut tx_enacted: Vec<SignedUserTransaction> = Vec::new();
        let mut tx_retracted: Vec<SignedUserTransaction> = Vec::new();
        enacted.iter().for_each(|b| {
            tx_enacted.append(&mut b.transactions().clone().to_vec());
        });
        retracted.iter().for_each(|b| {
            tx_retracted.append(&mut b.transactions().clone().to_vec());
        });
        debug!(
            "commit size:{}, rollback size:{}",
            tx_enacted.len(),
            tx_retracted.len()
        );
        (tx_enacted, tx_retracted)
    }
}

impl<E, C, P, S> ChainService for ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: BlockChainStore,
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
            let mut branch = self.find_or_fork(&header).expect("fork branch failed.");
            branch.apply(block.clone())?;
            self.select_head(branch);
            self.master.latest_blocks();
        }
        Ok(())
    }

    fn head_block(&self) -> Block {
        self.master.head_block()
    }

    fn current_header(&self) -> BlockHeader {
        self.master.current_header()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn get_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.master.get_block_by_number(number)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn create_block_template(
        &self,
        parent_hash: Option<HashValue>,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        self.master
            .create_block_template(parent_hash, difficulty, user_txns)
    }

    fn gen_tx(&self) -> Result<()> {
        self.master.gen_tx()
    }

    fn get_chain_info(&self) -> ChainInfo {
        self.master.get_chain_info()
    }
}
