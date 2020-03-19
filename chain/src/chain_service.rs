// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use actix::prelude::*;
use anyhow::{format_err, Result};
use config::NodeConfig;
use consensus::Consensus;
use crypto::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::network::NetworkAsyncService;
use parking_lot::RwLock;
use starcoin_statedb::ChainStateDB;
use std::collections::HashMap;
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
// use std::sync::RwLock;

struct BlockChainCollection<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: BlockChainStore + 'static,
{
    master: BlockChain<E, C, S, P>,
    branches: HashMap<HashValue, BlockChain<E, C, S, P>>,
}

impl<E, C, P, S> BlockChainCollection<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: BlockChainStore + 'static,
{
    pub fn new(master: BlockChain<E, C, S, P>) -> Self {
        BlockChainCollection {
            master,
            branches: HashMap::new(),
        }
    }

    pub fn insert_branch(&mut self, branch: BlockChain<E, C, S, P>) {
        self.branches
            .insert(branch.get_chain_info().branch_id(), branch);
    }

    pub fn update_master(&mut self, new_master: BlockChain<E, C, S, P>) {
        self.master = new_master;
    }

    pub fn get_master(&self) -> &BlockChain<E, C, S, P> {
        &self.master
    }

    pub fn get_branch(&self, branch_id: &HashValue) -> Option<&BlockChain<E, C, S, P>> {
        self.branches.get(branch_id)
    }

    pub fn remove_branch(&mut self, branch_id: &HashValue) {
        self.branches.remove(branch_id);
    }

    pub fn fork(&self, block_header: &BlockHeader) -> Option<ChainInfo> {
        let mut chain_info = self.master.fork(block_header);
        if chain_info.is_none() {
            for branch in self.branches.values() {
                chain_info = branch.fork(block_header);
                if chain_info.is_some() {
                    break;
                }
            }
        }

        chain_info
    }

    pub fn create_block_template(
        &self,
        block_id: HashValue,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        if self.master.exist_block(block_id) {
            self.master
                .create_block_template(Some(block_id), difficulty, user_txns)
        } else {
            let mut tmp = None;
            for branch in self.branches.values() {
                if branch.exist_block(block_id) {
                    tmp = Some(branch.create_block_template(
                        Some(block_id),
                        difficulty,
                        user_txns.clone(),
                    ));
                }
            }

            Ok(tmp.unwrap().unwrap())
        }
    }
}

pub struct ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: BlockChainStore + 'static,
{
    config: Arc<NodeConfig>,
    // master: BlockChain<E, C, S, P>,
    // branches: Arc<RwLock<HashMap<HashValue, BlockChain<E, C, S, P>>>>,
    collection: Arc<RwLock<BlockChainCollection<E, C, S, P>>>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService>,
    txpool: P,
}

impl<E, C, P, S> ChainServiceImpl<E, C, P, S>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: BlockChainStore + 'static,
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
        let collection = Arc::new(RwLock::new(BlockChainCollection::new(master)));
        for branch_info in startup_info.branches {
            collection.write().insert(
                branch_info.branch_id(),
                BlockChain::new(config.clone(), branch_info, storage.clone(), txpool.clone())?,
            );
        }
        Ok(Self {
            config,
            // master,
            // branches,
            collection,
            storage,
            network,
            txpool,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Option<BlockChain<E, C, S, P>> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let mut chain_info = self.collection.read().fork(header);

        if chain_info.is_some() {
            return Some(
                BlockChain::new(
                    self.config.clone(),
                    chain_info.unwrap(),
                    self.storage.clone(),
                    self.txpool.clone(),
                )
                .unwrap(),
            );
        } else {
            None
        }
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<E, C, S, P>) {
        let block = new_branch.head_block();
        let _ = self
            .collection
            .write()
            .remove_branch(&new_branch.get_chain_info().branch_id());

        if new_branch.get_total_difficulty()
            > self.collection.read().get_master().get_total_difficulty()
        {
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            let mut retracted = Vec::new();
            if new_branch.get_chain_info().branch_id()
                == self
                    .collection
                    .read()
                    .get_master()
                    .get_chain_info()
                    .branch_id()
            {
                enacted.append(&mut block.transactions().clone().to_vec());
            } else {
                debug!("rollback branch.");
                let (mut enacted_tmp, mut retracted_tmp) = self.find_ancestors(&new_branch);
                enacted.append(&mut enacted_tmp);
                retracted.append(&mut retracted_tmp);

                self.collection.write().insert_branch(
                    BlockChain::new(
                        self.config.clone(),
                        self.collection.read().get_master().get_chain_info(),
                        self.storage.clone(),
                        self.txpool.clone(),
                    )
                    .unwrap(),
                );
            }

            self.collection.write().update_master(new_branch);
            self.commit_2_txpool(enacted, retracted);

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
        } else {
            self.collection.write().insert_branch(new_branch);
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

    fn find_ancestors(
        &self,
        new_branch: &BlockChain<E, C, S, P>,
    ) -> (Vec<SignedUserTransaction>, Vec<SignedUserTransaction>) {
        let mut enacted: Vec<Block> = Vec::new();
        let mut retracted: Vec<Block> = Vec::new();

        let block_enacted = &new_branch.current_header().id();
        let block_retracted = &self.collection.read().get_master().current_header().id();

        let ancestor = self
            .storage
            .get_common_ancestor(block_enacted.clone(), block_retracted.clone())
            .unwrap()
            .unwrap();

        let mut block_enacted_tmp = block_enacted.clone();

        debug!("ancestor block is : {:?}", ancestor);
        loop {
            if block_enacted_tmp == ancestor {
                break;
            };
            debug!("get block 1 {:?}.", block_enacted_tmp);
            let block_tmp = new_branch
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
                .collection
                .read()
                .get_master()
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
            //self.master.latest_blocks();
        }
        Ok(())
    }

    fn master_head_block(&self) -> Block {
        self.collection.read().get_master().head_block()
    }

    fn master_head_header(&self) -> BlockHeader {
        self.collection.read().get_master().current_header()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn master_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.collection
            .read()
            .get_master()
            .get_block_by_number(number)
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
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self.collection.read().get_master().current_header().id(),
        };

        if let Ok(Some(_)) = self.get_block_by_hash(block_id) {
            self.collection
                .create_block_template(block_id, difficulty, user_txns)
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }

    fn gen_tx(&self) -> Result<()> {
        self.collection.read().get_master().gen_tx()
    }

    fn master_chain_info(&self) -> ChainInfo {
        self.collection.read().get_master().get_chain_info()
    }
}
