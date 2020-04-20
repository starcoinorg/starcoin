// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use actix::prelude::*;
use anyhow::{format_err, Result};
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use network::{get_unix_ts, network::NetworkAsyncService};
use parking_lot::RwLock;
use starcoin_statedb::ChainStateDB;
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool_api::TxPoolAsyncService;
use std::collections::HashMap;
use std::sync::Arc;
use storage::Store;
use traits::Consensus;
use traits::{is_ok, ChainReader, ChainService, ChainWriter, ConnectBlockError, ConnectResult};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::SystemEvents,
    transaction::SignedUserTransaction,
};

pub struct BlockChainCollection<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    master: RwLock<Vec<BlockChain<C, S, P>>>,
    branches: RwLock<HashMap<HashValue, BlockChain<C, S, P>>>,
}

impl<C, S, P> BlockChainCollection<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    pub fn new() -> Self {
        BlockChainCollection {
            master: RwLock::new(Vec::new()),
            branches: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert_branch(&self, branch: BlockChain<C, S, P>) {
        self.branches
            .write()
            .insert(branch.get_chain_info().branch_id(), branch);
    }

    pub fn update_master(&self, new_master: BlockChain<C, S, P>) {
        self.master.write().insert(0, new_master)
    }

    pub fn get_branch_id(&self, branch_id: &HashValue, number: BlockNumber) -> Option<HashValue> {
        let mut chain_info = None;

        let master = self
            .master
            .read()
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
            .master
            .read()
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
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        if self
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .exist_block(block_id)
        {
            self.master
                .read()
                .get(0)
                .expect("master is none.")
                .create_block_template(author, auth_key_prefix, Some(block_id), user_txns)
        } else {
            // just for test
            let mut tmp = None;
            for branch in self.branches.read().values() {
                if branch.exist_block(block_id) {
                    tmp = Some(branch.create_block_template(
                        author,
                        auth_key_prefix.clone(),
                        Some(block_id),
                        user_txns.clone(),
                    ));
                }
            }

            Ok(tmp.unwrap().unwrap())
        }
    }

    pub fn to_startup_info(&self) -> StartupInfo {
        let head = self
            .master
            .read()
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

pub struct ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    config: Arc<NodeConfig>,
    collection: Arc<BlockChainCollection<C, S, P>>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService>,
    txpool: P,
    bus: Addr<BusActor>,
    sync_metadata: SyncMetadata,
}

impl<C, S, P> ChainServiceImpl<C, S, P>
where
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
        sync_metadata: SyncMetadata,
    ) -> Result<Self> {
        let collection = to_block_chain_collection(
            config.clone(),
            startup_info,
            storage.clone(),
            txpool.clone(),
        )?;
        Ok(Self {
            config,
            collection,
            storage,
            network,
            txpool,
            bus,
            sync_metadata,
        })
    }

    pub fn find_or_fork(&mut self, header: &BlockHeader) -> Result<BlockChain<C, S, P>> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let chain_info = self.collection.fork(header);
        if chain_info.is_some() {
            Ok(BlockChain::new(
                self.config.clone(),
                chain_info.unwrap(),
                self.storage.clone(),
                self.txpool.clone(),
                Arc::clone(&self.collection),
            )?)
        } else {
            Err(format_err!("{:?}", "chain info is none."))
        }
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<C, S, P>) -> Result<()> {
        let block = new_branch.head_block();
        let total_difficulty = new_branch.get_total_difficulty()?;
        if total_difficulty
            > self
                .collection
                .master
                .read()
                .get(0)
                .expect("master is none.")
                .get_total_difficulty()?
        {
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            let mut retracted = Vec::new();
            let mut rollback = false;
            if new_branch.get_chain_info().branch_id()
                == self
                    .collection
                    .master
                    .read()
                    .get(0)
                    .expect("master is none.")
                    .get_chain_info()
                    .branch_id()
            {
                enacted.append(&mut block.transactions().clone().to_vec());
            } else {
                debug!("rollback branch.");
                self.collection.insert_branch(BlockChain::new(
                    self.config.clone(),
                    self.collection
                        .master
                        .read()
                        .get(0)
                        .expect("master is none.")
                        .get_chain_info(),
                    self.storage.clone(),
                    self.txpool.clone(),
                    Arc::clone(&self.collection),
                )?);

                rollback = true;
            }

            let _ = self
                .collection
                .remove_branch(&new_branch.get_chain_info().branch_id());
            self.collection.update_master(BlockChain::new(
                self.config.clone(),
                new_branch.get_chain_info(),
                self.storage.clone(),
                self.txpool.clone(),
                Arc::clone(&self.collection),
            )?);
            if rollback {
                let (mut enacted_tmp, mut retracted_tmp) = self.find_ancestors(&new_branch)?;
                enacted.append(&mut enacted_tmp);
                retracted.append(&mut retracted_tmp);
            }

            self.commit_2_txpool(enacted, retracted);
            if self.sync_metadata.is_sync_done() {
                let block_detail = BlockDetail::new(block, total_difficulty);
                self.broadcast_2_bus(block_detail.clone());

                self.broadcast_2_network(block_detail);
            }
        } else {
            self.collection.insert_branch(new_branch);
        }

        let startup_info = self.collection.to_startup_info();
        debug!("save startup info : {:?}", startup_info);
        self.storage.save_startup_info(startup_info)
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
        new_branch: &BlockChain<C, S, P>,
    ) -> Result<(Vec<SignedUserTransaction>, Vec<SignedUserTransaction>)> {
        let mut enacted: Vec<Block> = Vec::new();
        let mut retracted: Vec<Block> = Vec::new();

        let block_enacted = &new_branch.current_header().id();
        let block_retracted = &self
            .collection
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .current_header()
            .id();

        let ancestor = self
            .storage
            .get_common_ancestor(block_enacted.clone(), block_retracted.clone())?
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
                .master
                .read()
                .get(0)
                .expect("master is none.")
                .get_block(block_retracted_tmp)?
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
        Ok((tx_enacted, tx_retracted))
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
                debug!("broadcast system event : {:?}", block.header().id());
                network
                    .clone()
                    .broadcast_system_event(SystemEvents::NewHeadBlock(block))
                    .await
                    .expect("broadcast new head block failed.");
            });
        };
    }
}

impl<C, S, P> ChainService for ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService,
    S: Store,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block, pivot_sync: bool) -> Result<ConnectResult<()>> {
        let connect_begin_time = get_unix_ts();
        if !self.sync_metadata.state_syncing() || pivot_sync {
            if self
                .storage
                .get_block_by_hash(block.header().id())?
                .is_none()
            {
                if self
                    .storage
                    .get_block_by_hash(block.header().parent_hash())?
                    .is_some()
                    && (!self.sync_metadata.state_syncing()
                        || (pivot_sync && self.sync_metadata.state_done()))
                {
                    let header = block.header();
                    let mut branch = self.find_or_fork(&header)?;
                    let fork_end_time = get_unix_ts();
                    debug!("fork used time: {}", (fork_end_time - connect_begin_time));

                    let connected = branch.apply(block.clone())?;
                    let apply_end_time = get_unix_ts();
                    debug!("apply used time: {}", (apply_end_time - fork_end_time));
                    if !connected {
                        Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                    } else {
                        self.select_head(branch)?;
                        let select_head_end_time = get_unix_ts();
                        debug!(
                            "select head used time: {}",
                            (select_head_end_time - apply_end_time)
                        );
                        self.collection
                            .master
                            .read()
                            .get(0)
                            .expect("master is none.")
                            .latest_blocks(10);
                        Ok(ConnectResult::Ok(()))
                    }
                } else {
                    Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                }
            } else {
                Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
            }
        } else {
            Ok(ConnectResult::Err(ConnectBlockError::Other(
                "error connect type.".to_string(),
            )))
        }
    }

    fn try_connect_with_block_info(
        &mut self,
        block: Block,
        block_info: BlockInfo,
    ) -> Result<ConnectResult<()>> {
        if self.sync_metadata.state_syncing() {
            if self
                .storage
                .get_block_by_hash(block.header().id())?
                .is_none()
            {
                if self
                    .storage
                    .get_block_by_hash(block.header().parent_hash())?
                    .is_some()
                {
                    let pivot = self.sync_metadata.get_pivot()?;
                    let latest_sync_number = self.sync_metadata.get_latest();
                    if pivot.is_some() && latest_sync_number.is_some() {
                        let pivot_number = pivot.unwrap();
                        let latest_number = latest_sync_number.unwrap();
                        let current_block_number = block.header().number();
                        if pivot_number >= current_block_number {
                            //todo:1. verify block header / verify accumulator / total difficulty
                            let mut block_chain = self.collection.master.write();
                            let master = block_chain.get_mut(0).expect("master is none.");
                            let block_header = block.header().clone();
                            if let Ok(_) =
                                C::verify_header(self.config.clone(), master, &block_header)
                            {
                                // 2. commit block
                                let _ = master.commit(block, block_info)?;
                                Ok(ConnectResult::Ok(()))
                            } else {
                                Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                            }
                        } else if latest_number >= current_block_number {
                            let connect_result = self.try_connect(block, true)?;
                            // 3. update sync metadata
                            info!(
                                "connect block : {}, {}, {:?}",
                                latest_number, current_block_number, connect_result
                            );
                            if latest_number == current_block_number && is_ok(&connect_result) {
                                if let Err(err) = self.sync_metadata.block_sync_done() {
                                    warn!("err:{:?}", err);
                                }
                            }
                            Ok(connect_result)
                        } else {
                            Ok(ConnectResult::Err(ConnectBlockError::Other(
                                "block number > pivot.".to_string(),
                            )))
                        }
                    } else {
                        Ok(ConnectResult::Err(ConnectBlockError::Other(
                            "pivot is none.".to_string(),
                        )))
                    }
                } else {
                    Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                }
            } else {
                Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
            }
        } else {
            self.try_connect(block, false)
        }
    }

    fn master_head_block(&self) -> Block {
        self.collection
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .head_block()
    }

    fn master_head_header(&self) -> BlockHeader {
        self.collection
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .current_header()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn master_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.collection
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .get_block_by_number(number)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>> {
        self.storage.get_block_info(hash)
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
            None => self
                .collection
                .master
                .read()
                .get(0)
                .expect("master is none.")
                .current_header()
                .id(),
        };

        if let Ok(Some(_)) = self.get_block_by_hash(block_id) {
            self.collection
                .create_block_template(author, auth_key_prefix, block_id, user_txns)
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }

    fn gen_tx(&self) -> Result<()> {
        self.collection
            .master
            .read()
            .get(0)
            .expect("master is none.")
            .gen_tx()
    }

    fn master_startup_info(&self) -> StartupInfo {
        self.collection.to_startup_info()
    }

    fn master_blocks_by_number(&self, number: u64, count: u64) -> Result<Vec<Block>> {
        self.collection
            .get_master()
            .borrow()
            .get(0)
            .expect("master is none.")
            .get_blocks_by_number(number, count)
    }
}

pub fn to_block_chain_collection<C, S, P>(
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    storage: Arc<S>,
    txpool: P,
) -> Result<Arc<BlockChainCollection<C, S, P>>>
where
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
