// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use actix::prelude::*;
use anyhow::{format_err, Error, Result};
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use network::{get_unix_ts, NetworkAsyncService};
use network_api::NetworkService;
use parking_lot::RwLock;
use starcoin_statedb::ChainStateDB;
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool_api::TxPoolAsyncService;
use std::sync::Arc;
use storage::Store;
use traits::Consensus;
use traits::{is_ok, ChainReader, ChainService, ChainWriter, ConnectBlockError, ConnectResult};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    system_events::SystemEvents,
    transaction::{SignedUserTransaction, TransactionInfo},
};

pub struct BlockChainCollection<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    startup_info: RwLock<StartupInfo>,
    master: RwLock<Option<Arc<BlockChain<C, S, P>>>>,
    storage: Arc<S>,
}

impl<C, S, P> Drop for BlockChainCollection<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    fn drop(&mut self) {
        debug!("drop BlockChainCollection");
    }
}

impl<C, S, P> BlockChainCollection<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    pub fn new(startup_info: StartupInfo, storage: Arc<S>) -> Self {
        BlockChainCollection {
            startup_info: RwLock::new(startup_info),
            master: RwLock::new(None),
            storage,
        }
    }

    pub fn init_master(&self, new_master: BlockChain<C, S, P>) {
        assert!(self.master.read().is_none());
        assert_eq!(self.startup_info.read().master, new_master.get_chain_info());
        self.update_master(new_master)
    }

    pub fn update_master(&self, new_master: BlockChain<C, S, P>) {
        let chain_info = new_master.get_chain_info();
        *self.master.write() = Some(Arc::new(new_master));
        self.startup_info.write().update_master(chain_info);
    }

    pub fn insert_branch(&self, branch: BlockChain<C, S, P>) {
        self.startup_info
            .write()
            .insert_branch(branch.get_chain_info());
    }

    pub fn remove_branch(&self, branch_id: &HashValue) {
        self.startup_info.write().remove_branch(branch_id.clone());
    }

    pub fn get_branch_id(&self, branch_id: &HashValue, number: BlockNumber) -> Option<HashValue> {
        let master = self.get_master_chain_info();
        let chain_info = if &master.branch_id() == branch_id {
            Some(master)
        } else {
            self.startup_info.read().get_branch(branch_id.clone())
        };

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

    pub fn fork(&self, block_header: &BlockHeader) -> Option<ChainInfo> {
        let chain_info = self.get_master().fork(block_header);
        if chain_info.is_none() {
            if let Ok(Some(branch_id)) = self.storage.get_branch(block_header.parent_hash()) {
                if let Some(chain_info) = self.startup_info.read().get_branch(branch_id) {
                    return if chain_info.get_head() == block_header.parent_hash() {
                        Some(chain_info)
                    } else {
                        Some(ChainInfo::new(
                            Some(chain_info.branch_id()),
                            block_header.parent_hash(),
                            block_header,
                        ))
                    };
                }
            }
        }

        chain_info
    }

    pub fn block_exist(&self, block_id: HashValue) -> bool {
        if let Ok(branch_id) = self.storage.get_branch(block_id) {
            return branch_id.is_some();
        }
        false
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        block_id: HashValue,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        if self.get_master().exist_block(block_id) {
            self.get_master().create_block_template(
                author,
                auth_key_prefix,
                Some(block_id),
                user_txns,
            )
        } else {
            // just for test
            let mut tmp = None;
            if let Some(branch_id) = self.storage.get_branch(block_id)? {
                if let Some(branch) = self.startup_info.read().get_branch(branch_id) {
                    let chain = self.get_master().new_chain(branch)?;
                    tmp = Some(chain.create_block_template(
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
        self.startup_info.read().clone()
    }

    pub fn get_master(&self) -> Arc<BlockChain<C, S, P>> {
        self.master.read().as_ref().unwrap().clone()
    }

    pub fn get_master_chain_info(&self) -> ChainInfo {
        self.get_master().get_chain_info()
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

    pub fn find_or_fork(
        &mut self,
        header: &BlockHeader,
    ) -> Result<(bool, Option<BlockChain<C, S, P>>)> {
        debug!("{:?}:{:?}", header.parent_hash(), header.id());
        let chain_info = self.collection.fork(header);
        if chain_info.is_some() {
            let block_exist = self.collection.block_exist(header.id());
            let branch = BlockChain::new(
                self.config.clone(),
                chain_info.unwrap(),
                self.storage.clone(),
                self.txpool.clone(),
                Arc::downgrade(&self.collection),
            )?;
            Ok((block_exist, Some(branch)))
        } else {
            Ok((false, None))
        }
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<C, S, P>) -> Result<()> {
        let branch_id = new_branch.get_chain_info().branch_id();
        let block = new_branch.head_block();
        let block_id = block.header().id();
        let total_difficulty = new_branch.get_total_difficulty()?;
        if total_difficulty > self.collection.get_master().get_total_difficulty()? {
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            let mut retracted = Vec::new();
            let mut rollback = false;
            if new_branch.get_chain_info().branch_id()
                == self.collection.get_master_chain_info().branch_id()
            {
                enacted.append(&mut block.transactions().clone().to_vec());
            } else {
                debug!("rollback branch.");
                self.collection.insert_branch(BlockChain::new(
                    self.config.clone(),
                    self.collection.get_master_chain_info(),
                    self.storage.clone(),
                    self.txpool.clone(),
                    Arc::downgrade(&self.collection),
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
                Arc::downgrade(&self.collection),
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

        self.storage.save_branch(branch_id, block_id)?;
        self.save_startup()
    }

    fn save_startup(&self) -> Result<()> {
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
        let block_retracted = &self.collection.get_master().current_header().id();

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
                .get_master()
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
        bus.do_send(Broadcast {
            msg: SystemEvents::NewHeadBlock(Arc::new(block)),
        });
    }

    pub fn broadcast_2_network(&self, block: BlockDetail) {
        if let Some(network) = self.network.clone() {
            Arbiter::spawn(async move {
                let id = block.header().id();
                let is_ok = network
                    .broadcast_system_event(SystemEvents::NewHeadBlock(Arc::new(block)))
                    .await
                    .is_ok();
                debug!("broadcast system event : {:?}, is_ok:{}", id, is_ok);
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
            if !self.sync_metadata.state_syncing()
                || (pivot_sync && self.sync_metadata.state_done())
            {
                let (block_exist, fork) = self.find_or_fork(block.header())?;
                if block_exist {
                    Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                } else {
                    if let Some(mut branch) = fork {
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
                            self.collection.get_master().latest_blocks(1);
                            Ok(ConnectResult::Ok(()))
                        }
                    } else {
                        Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                    }
                }
            } else {
                Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
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
            let pivot = self.sync_metadata.get_pivot()?;
            let latest_sync_number = self.sync_metadata.get_latest();
            if pivot.is_some() && latest_sync_number.is_some() {
                let pivot_number = pivot.unwrap();
                let latest_number = latest_sync_number.unwrap();
                let current_block_number = block.header().number();
                if pivot_number >= current_block_number {
                    //todo:1. verify block header / verify accumulator / total difficulty
                    let (block_exist, fork) = self.find_or_fork(block.header())?;
                    if block_exist {
                        Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                    } else {
                        if let Some(mut branch) = fork {
                            if let Ok(_) =
                                C::verify_header(self.config.clone(), &branch, block.header())
                            {
                                // 2. commit block
                                branch.commit(block, block_info)?;
                                self.select_head(branch)?;
                                Ok(ConnectResult::Ok(()))
                            } else {
                                Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                            }
                        } else {
                            Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                        }
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
            self.try_connect(block, false)
        }
    }

    fn master_head_block(&self) -> Block {
        self.collection.get_master().head_block()
    }

    fn master_head_header(&self) -> BlockHeader {
        self.collection.get_master().current_header()
    }

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn master_block_by_number(&self, number: u64) -> Result<Option<Block>> {
        self.collection.get_master().get_block_by_number(number)
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
            None => self.collection.get_master().current_header().id(),
        };

        if let Ok(Some(_)) = self.get_block_by_hash(block_id) {
            self.collection
                .create_block_template(author, auth_key_prefix, block_id, user_txns)
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }

    fn gen_tx(&self) -> Result<()> {
        self.collection.get_master().gen_tx()
    }

    fn master_startup_info(&self) -> StartupInfo {
        self.collection.to_startup_info()
    }

    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>> {
        self.collection
            .get_master()
            .get_blocks_by_number(number, count)
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        self.collection.get_master().get_transaction_info(hash)
    }

    fn get_block_txn_ids(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        self.collection
            .get_master()
            .get_block_transactions(block_id)
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
    let master_chain_info = startup_info.master.clone();
    let collection = Arc::new(BlockChainCollection::new(startup_info, storage.clone()));
    let master = BlockChain::new(
        config,
        master_chain_info,
        storage,
        txpool,
        Arc::downgrade(&collection),
    )?;
    collection.init_master(master);

    Ok(collection)
}
