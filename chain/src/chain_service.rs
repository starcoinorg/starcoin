// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use actix::prelude::*;
use anyhow::{format_err, Result};
use atomic_refcell::AtomicRefCell;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use consensus::Consensus;
use crypto::HashValue;
use executor::TransactionExecutor;
use logger::prelude::*;
use network::network::NetworkAsyncService;
use parking_lot::RwLock;
use starcoin_statedb::ChainStateDB;
use starcoin_txpool_api::TxPoolAsyncService;
use std::collections::HashMap;
use std::sync::Arc;
use storage::Store;
use traits::{ChainReader, ChainService, ChainWriter};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
    U256,
};

struct StateSyncMetadata {
    syncing: bool,
    pivot: Option<BlockNumber>,
}

impl StateSyncMetadata {
    pub fn new(state_sync: bool) -> Self {
        StateSyncMetadata {
            syncing: state_sync,
            pivot: None,
        }
    }

    pub fn _update_pivot(&mut self, pivot: BlockNumber) {
        assert!(self.syncing, "chain is not in fast sync mode.");
        self.pivot = Some(pivot);
    }

    pub fn _change_2_full(&mut self) {
        self.syncing = false;
        self.pivot = None;
    }

    pub fn is_state_sync(&self) -> bool {
        self.syncing
    }

    pub fn get_pivot(&self) -> Option<BlockNumber> {
        self.pivot.clone()
    }
}

pub struct BlockChainCollection<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    master: AtomicRefCell<Vec<BlockChain<E, C, S, P>>>,
    branches: RwLock<HashMap<HashValue, BlockChain<E, C, S, P>>>,
}

impl<E, C, S, P> BlockChainCollection<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    pub fn new() -> Self {
        BlockChainCollection {
            master: AtomicRefCell::new(Vec::new()),
            branches: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_branch(&self, branch: BlockChain<E, C, S, P>) {
        self.branches
            .write()
            .insert(branch.get_chain_info().branch_id(), branch);
    }

    pub fn update_master(&self, new_master: BlockChain<E, C, S, P>) {
        self.master.borrow_mut().insert(0, new_master)
    }

    pub fn get_master(&self) -> &AtomicRefCell<Vec<BlockChain<E, C, S, P>>> {
        &self.master
    }

    pub fn get_branch_id(&self, branch_id: &HashValue, number: BlockNumber) -> Option<HashValue> {
        let mut chain_info = None;

        let master = self
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .get_chain_info();
        if master.branch_id() == branch_id.clone() {
            chain_info = Some(master)
        } else {
            for branch in self.branches.read().values() {
                if branch.get_chain_info().branch_id() == branch_id.clone() {
                    chain_info = Some(branch.get_chain_info());
                    break;
                }
            }
        }

        if let Some(tmp) = chain_info {
            if number >= tmp.start_number() {
                return Some(tmp.branch_id());
            } else {
                if let Some(parent_branch) = tmp.parent_branch() {
                    return self.get_branch_id(&parent_branch, number);
                }
            }
        }

        return None;
    }

    pub fn remove_branch(&self, branch_id: &HashValue) {
        self.branches.write().remove(branch_id);
    }

    pub fn fork(&self, block_header: &BlockHeader) -> Option<ChainInfo> {
        let mut chain_info = self
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .fork(block_header);
        if chain_info.is_none() {
            for branch in self.branches.read().values() {
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
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        block_id: HashValue,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        if self
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .exist_block(block_id)
        {
            self.get_master()
                .borrow()
                .get(0)
                .expect("master is none.")
                .create_block_template(
                    author,
                    auth_key_prefix,
                    Some(block_id),
                    difficulty,
                    user_txns,
                )
        } else {
            // just for test
            let mut tmp = None;
            for branch in self.branches.read().values() {
                if branch.exist_block(block_id) {
                    tmp = Some(branch.create_block_template(
                        author,
                        auth_key_prefix.clone(),
                        Some(block_id),
                        difficulty,
                        user_txns.clone(),
                    ));
                }
            }

            Ok(tmp.unwrap().unwrap())
        }
    }

    pub fn to_startup_info(&self) -> StartupInfo {
        let head = self
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .get_chain_info();
        let mut branches = Vec::new();
        for branch in self.branches.read().values() {
            branches.push(branch.get_chain_info());
        }
        StartupInfo::new(head, branches)
    }
}

pub struct ChainServiceImpl<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    config: Arc<NodeConfig>,
    collection: Arc<BlockChainCollection<E, C, S, P>>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService>,
    txpool: P,
    bus: Addr<BusActor>,
    sync: RwLock<StateSyncMetadata>,
    _future_blocks: RwLock<HashMap<HashValue, (Block, Option<BlockInfo>)>>, //todo
}

impl<E, C, S, P> ChainServiceImpl<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<S>,
        network: Option<NetworkAsyncService>,
        txpool: P,
        bus: Addr<BusActor>,
    ) -> Result<Self> {
        let collection = to_block_chain_collection(
            config.clone(),
            startup_info,
            storage.clone(),
            txpool.clone(),
        )?;
        let state_sync_flag = config.sync.is_state_sync();
        let future_blocks: RwLock<HashMap<HashValue, (Block, Option<BlockInfo>)>> =
            RwLock::new(HashMap::new());
        Ok(Self {
            config,
            collection,
            storage,
            network,
            txpool,
            bus,
            sync: RwLock::new(StateSyncMetadata::new(state_sync_flag)),
            _future_blocks: future_blocks,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Option<BlockChain<E, C, S, P>> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let chain_info = self.collection.fork(header);

        if chain_info.is_some() {
            return Some(
                BlockChain::new(
                    self.config.clone(),
                    chain_info.unwrap(),
                    self.storage.clone(),
                    self.txpool.clone(),
                    Arc::clone(&self.collection),
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
        let total_difficulty = new_branch.get_total_difficulty();
        if total_difficulty
            > self
                .collection
                .get_master()
                .borrow()
                .get(0)
                .expect("master is none.")
                .get_total_difficulty()
        {
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            let mut retracted = Vec::new();
            if new_branch.get_chain_info().branch_id()
                == self
                    .collection
                    .get_master()
                    .borrow()
                    .get(0)
                    .expect("master is none.")
                    .get_chain_info()
                    .branch_id()
            {
                enacted.append(&mut block.transactions().clone().to_vec());
            } else {
                debug!("rollback branch.");
                let (mut enacted_tmp, mut retracted_tmp) = self.find_ancestors(&new_branch);
                enacted.append(&mut enacted_tmp);
                retracted.append(&mut retracted_tmp);

                self.collection.insert_branch(
                    BlockChain::new(
                        self.config.clone(),
                        self.collection
                            .get_master()
                            .borrow()
                            .get(0)
                            .expect("master is none.")
                            .get_chain_info(),
                        self.storage.clone(),
                        self.txpool.clone(),
                        Arc::clone(&self.collection),
                    )
                    .unwrap(),
                );
            }

            let _ = self
                .collection
                .remove_branch(&new_branch.get_chain_info().branch_id());
            self.collection.update_master(new_branch);
            self.commit_2_txpool(enacted, retracted);

            let block_detail = BlockDetail::new(block, total_difficulty);
            self.broadcast_2_bus(block_detail.clone());

            self.broadcast_2_network(block_detail);
        } else {
            self.collection.insert_branch(new_branch);
        }

        let startup_info = self.collection.to_startup_info();
        debug!("save startup info : {:?}", startup_info);
        if let Err(e) = self.storage.save_startup_info(startup_info) {
            warn!("err: {:?}", e);
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
        let block_retracted = &self
            .collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .current_header()
            .id();

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
                .get_master()
                .borrow()
                .get(0)
                .expect("master is none.")
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

    pub fn broadcast_2_bus(&self, block: BlockDetail) {
        let bus = self.bus.clone();
        Arbiter::spawn(async move {
            let _ = bus
                .send(Broadcast {
                    msg: SystemEvents::NewHeadBlock(block),
                })
                .await;
        });
    }

    pub fn broadcast_2_network(&self, block: BlockDetail) {
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

impl<E, C, S, P> ChainService for ChainServiceImpl<E, C, S, P>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService,
    S: Store,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block) -> Result<()> {
        if !self.sync.read().is_state_sync() {
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
                self.collection
                    .get_master()
                    .borrow()
                    .get(0)
                    .expect("master is none.")
                    .latest_blocks(10);
            }
        }
        Ok(())
    }

    fn try_connect_with_block_info(&mut self, block: Block, block_info: BlockInfo) -> Result<()> {
        if self.sync.read().is_state_sync() {
            let pivot = self.sync.read().get_pivot();
            if pivot.is_some() && pivot.unwrap() >= block.header().number() {
                //todo:1. verify block header / verify accumulator / total difficulty
                let mut block_chain = self.collection.get_master().borrow_mut();
                let master = block_chain.get_mut(0).expect("master is none.");
                let block_header = block.header().clone();
                if let Ok(_) = C::verify_header(self.config.clone(), master, &block_header) {
                    // 2. save block
                    let _ = master.commit(block, block_info);
                    // 3. update master
                    self.collection
                        .get_master()
                        .borrow()
                        .get(0)
                        .expect("master is none.")
                        .latest_blocks(1);
                }
            }
        }

        Ok(())
    }

    fn master_head_block(&self) -> Block {
        self.collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .head_block()
    }

    fn master_head_header(&self) -> BlockHeader {
        self.collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .current_header()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn master_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .get_block_by_number(number)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        parent_hash: Option<HashValue>,
        difficulty: U256,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_id = match parent_hash {
            Some(hash) => hash,
            None => self
                .collection
                .get_master()
                .borrow()
                .get(0)
                .expect("master is none.")
                .current_header()
                .id(),
        };

        if let Ok(Some(_)) = self.get_block_by_hash(block_id) {
            self.collection.create_block_template(
                author,
                auth_key_prefix,
                block_id,
                difficulty,
                user_txns,
            )
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }

    fn gen_tx(&self) -> Result<()> {
        self.collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .gen_tx()
    }

    fn master_startup_info(&self) -> StartupInfo {
        self.collection.to_startup_info()
    }
}

pub fn to_block_chain_collection<E, C, S, P>(
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    storage: Arc<S>,
    txpool: P,
) -> Result<Arc<BlockChainCollection<E, C, S, P>>>
where
    E: TransactionExecutor,
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    let collection = Arc::new(BlockChainCollection::new());
    let master = BlockChain::new(
        config.clone(),
        startup_info.head,
        storage.clone(),
        txpool.clone(),
        Arc::clone(&collection),
    )?;

    collection.update_master(master);

    for branch_info in startup_info.branches {
        collection.insert_branch(BlockChain::new(
            config.clone(),
            branch_info,
            storage.clone(),
            txpool.clone(),
            Arc::clone(&collection),
        )?);
    }

    Ok(collection)
}
