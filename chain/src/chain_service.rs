// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain::BlockChain;
use crate::chain_metrics::CHAIN_METRICS;
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
    startup_info::StartupInfo,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, TransactionInfo},
};

pub struct BlockChainCollection<C, S>
where
    C: Consensus,
    S: Store + 'static,
{
    startup_info: RwLock<StartupInfo>,
    master: RwLock<Option<Arc<BlockChain<C, S>>>>,
    _storage: Arc<S>,
}

impl<C, S> Drop for BlockChainCollection<C, S>
where
    C: Consensus,
    S: Store + 'static,
{
    fn drop(&mut self) {
        debug!("drop BlockChainCollection");
    }
}

impl<C, S> BlockChainCollection<C, S>
where
    C: Consensus,
    S: Store + 'static,
{
    pub fn new(startup_info: StartupInfo, _storage: Arc<S>) -> Self {
        BlockChainCollection {
            startup_info: RwLock::new(startup_info),
            master: RwLock::new(None),
            _storage,
        }
    }

    pub fn init_master(&self, new_master: BlockChain<C, S>) {
        assert!(self.master.read().is_none());
        assert_eq!(
            self.startup_info.read().get_master(),
            &new_master.current_header().id()
        );
        self.update_master(new_master)
    }

    pub fn update_master(&self, new_master: BlockChain<C, S>) {
        let header = new_master.current_header();
        *self.master.write() = Some(Arc::new(new_master));
        self.startup_info.write().update_master(&header);
    }

    pub fn insert_branch(&self, new_block_header: &BlockHeader) {
        self.startup_info.write().insert_branch(new_block_header);
    }

    pub fn create_block_template(
        &self,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        block_id: HashValue,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<BlockTemplate> {
        let block_chain = self.get_master().new_chain(block_id)?;
        block_chain.create_block_template(author, auth_key_prefix, Some(block_id), user_txns)
    }

    pub fn to_startup_info(&self) -> StartupInfo {
        self.startup_info.read().clone()
    }

    pub fn get_master(&self) -> Arc<BlockChain<C, S>> {
        self.master.read().as_ref().unwrap().clone()
    }

    pub fn get_head(&self) -> HashValue {
        *self.startup_info.read().get_master()
    }
}

pub struct ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolAsyncService + 'static,
    S: Store + 'static,
{
    config: Arc<NodeConfig>,
    collection: Arc<BlockChainCollection<C, S>>,
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
        let collection = to_block_chain_collection(config.clone(), startup_info, storage.clone())?;
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
    ) -> Result<(bool, Option<BlockChain<C, S>>)> {
        CHAIN_METRICS.try_connect_count.inc();
        let block_exist = self.block_exist(header.id());
        let block_chain = if !block_exist {
            if self.block_exist(header.parent_hash()) {
                Some(BlockChain::new(
                    self.config.clone(),
                    header.parent_hash(),
                    self.storage.clone(),
                    Arc::downgrade(&self.collection),
                )?)
            } else {
                None
            }
        } else {
            None
        };
        Ok((block_exist, block_chain))
    }

    pub fn block_exist(&self, block_id: HashValue) -> bool {
        if let Ok(Some(_)) = self.storage.get_block_info(block_id) {
            true
        } else {
            false
        }
    }

    pub fn state_at(&self, _root: HashValue) -> ChainStateDB {
        unimplemented!()
    }

    fn select_head(&mut self, new_branch: BlockChain<C, S>) -> Result<()> {
        let block = new_branch.head_block();
        let block_header = block.header();
        let total_difficulty = new_branch.get_total_difficulty()?;
        if total_difficulty > self.collection.get_master().get_total_difficulty()? {
            let mut enacted: Vec<SignedUserTransaction> = Vec::new();
            let mut retracted = Vec::new();
            if block.header().parent_hash() == self.collection.get_head() {
                enacted.append(&mut block.transactions().to_vec());
            } else {
                CHAIN_METRICS.rollback_count.inc();
                debug!("rollback branch.");

                let (enacted_blocks, mut enacted_tmp, mut retracted_tmp) =
                    self.find_ancestors(&new_branch)?;
                enacted.append(&mut enacted_tmp);
                retracted.append(&mut retracted_tmp);
                if self.sync_metadata.is_sync_done() {
                    enacted_blocks.into_iter().for_each(|enacted_block| {
                        if let Ok(Some(b_i)) =
                            self.storage.get_block_info(enacted_block.header().id())
                        {
                            let enacted_block_detail =
                                BlockDetail::new(enacted_block, b_i.get_total_difficulty());
                            self.broadcast_2_bus(enacted_block_detail);
                        }
                    });
                }
            }

            self.collection.update_master(new_branch);

            self.commit_2_txpool(enacted, retracted);
            if self.sync_metadata.is_sync_done() {
                CHAIN_METRICS.broadcast_head_count.inc();
                let block_detail = BlockDetail::new(block, total_difficulty);
                self.broadcast_2_bus(block_detail.clone());

                self.broadcast_2_network(block_detail);
            }
        } else {
            self.collection.insert_branch(block_header);
        }

        CHAIN_METRICS
            .branch_total_count
            .set(self.collection.startup_info.read().branches.len() as i64);
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
        new_branch: &BlockChain<C, S>,
    ) -> Result<(
        Vec<Block>,
        Vec<SignedUserTransaction>,
        Vec<SignedUserTransaction>,
    )> {
        let block_enacted = &new_branch.current_header().id();
        let block_retracted = &self.collection.get_master().current_header().id();

        let ancestor = self
            .storage
            .get_common_ancestor(*block_enacted, *block_retracted)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find ancestor with {} and {}.",
                    block_enacted,
                    block_retracted
                )
            })?;

        let enacted = self.find_blocks_until(*block_enacted, ancestor)?;
        let retracted = self.find_blocks_until(*block_retracted, ancestor)?;
        let mut tx_enacted: Vec<SignedUserTransaction> = Vec::new();
        let mut tx_retracted: Vec<SignedUserTransaction> = Vec::new();
        enacted.iter().for_each(|b| {
            tx_enacted.append(&mut b.transactions().to_vec());
        });
        retracted.iter().for_each(|b| {
            tx_retracted.append(&mut b.transactions().to_vec());
        });
        debug!(
            "commit size:{}, rollback size:{}",
            tx_enacted.len(),
            tx_retracted.len()
        );
        Ok((enacted, tx_enacted, tx_retracted))
    }

    fn find_blocks_until(&self, from: HashValue, until: HashValue) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut tmp = from;
        loop {
            if tmp == until {
                break;
            };
            let block = self
                .storage
                .get_block(tmp)?
                .ok_or_else(|| format_err!("Can not find block {}.", tmp))?;
            tmp = block.header().parent_hash();
            blocks.push(block);
        }
        blocks.reverse();

        Ok(blocks)
    }

    pub fn broadcast_2_bus(&self, block: BlockDetail) {
        let bus = self.bus.clone();
        bus.do_send(Broadcast {
            msg: NewHeadBlock(Arc::new(block)),
        });
    }

    pub fn broadcast_2_network(&self, block: BlockDetail) {
        if let Some(network) = self.network.clone() {
            Arbiter::spawn(async move {
                let id = block.header().id();
                let is_ok = network
                    .broadcast_new_head_block(NewHeadBlock(Arc::new(block)))
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
                info!(
                    "startup_info branch try_connect : {:?}, {:?}, :{:?}",
                    block.header().parent_hash(),
                    block.header().id(),
                    block_exist
                );
                if block_exist {
                    CHAIN_METRICS.duplicate_conn_count.inc();
                    Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                } else if let Some(mut branch) = fork {
                    let fork_end_time = get_unix_ts();
                    debug!("fork used time: {}", (fork_end_time - connect_begin_time));

                    let timer = CHAIN_METRICS
                        .exe_block_time
                        .with_label_values(&["time"])
                        .start_timer();

                    let connected = branch.apply(block.clone())?;
                    timer.observe_duration();
                    let apply_end_time = get_unix_ts();
                    let apply_total_time = apply_end_time - fork_end_time;
                    debug!("apply used time: {}", apply_total_time);
                    if !connected {
                        debug!("connected failed {:?}", block.header().id());
                        CHAIN_METRICS.verify_fail_count.inc();
                        Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                    } else {
                        self.select_head(branch)?;
                        let select_head_end_time = get_unix_ts();
                        debug!(
                            "select head used time: {}",
                            (select_head_end_time - apply_end_time)
                        );
                        self.collection.get_master().latest_blocks(10);
                        Ok(ConnectResult::Ok(()))
                    }
                } else {
                    debug!("future block 1 {:?}", block.header().id());
                    Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                }
            } else {
                debug!("future block 2 {:?}", block.header().id());
                Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
            }
        } else {
            Ok(ConnectResult::Err(ConnectBlockError::Other(
                format!("error connect type. pivot_sync: {}, state_syncing: {:?}, block_id: {:?}, number : {}, \
                pivot_connected: {}, pivot : {:?}, ", pivot_sync,
                        self.sync_metadata.state_syncing(), block.header().id(), block.header().number(),
                        self.sync_metadata.pivot_connected(), self.sync_metadata.get_pivot()),
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
            if let (Some(pivot_number), Some(latest_number)) = (pivot, latest_sync_number) {
                let current_block_number = block.header().number();
                if pivot_number >= current_block_number {
                    let pivot_flag = pivot_number == current_block_number;
                    if pivot_flag && !self.sync_metadata.state_done() {
                        debug!("block future {:?} for pivot.", block.header().id());
                        self.sync_metadata.set_pivot_block(block, block_info)?;
                        return Ok(ConnectResult::Err(ConnectBlockError::Other(
                            "pivot block wait state.".to_string(),
                        )));
                    }
                    //todo:1. verify block header / verify accumulator / total difficulty
                    let (block_exist, fork) = self.find_or_fork(block.header())?;
                    debug!(
                        "startup_info branch try_connect_with_block_info : {:?}, {:?}, :{:?}",
                        block.header().parent_hash(),
                        block.header().id(),
                        block_exist
                    );
                    if block_exist {
                        CHAIN_METRICS.duplicate_conn_count.inc();
                        Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                    } else if let Some(mut branch) = fork {
                        if C::verify_header(self.config.clone(), &branch, block.header()).is_ok() {
                            // 2. commit block
                            if pivot_flag {
                                branch.append_pivot(
                                    block.id(),
                                    block_info.get_block_accumulator_info().clone(),
                                )?
                            }
                            branch.commit(block, block_info)?;
                            self.select_head(branch)?;
                            if pivot_flag {
                                self.sync_metadata.pivot_connected_succ()?;
                            }
                            let master_header = self.collection.get_master().current_header();
                            info!(
                                "block chain info :: number : {} , block_id : {:?}, parent_id : {:?}",
                                master_header.number(),
                                master_header.id(),
                                master_header.parent_hash()
                            );
                            Ok(ConnectResult::Ok(()))
                        } else {
                            debug!("verify failed {:?}", block.header().id());
                            Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                        }
                    } else {
                        debug!("block future {:?}", block.header().id());
                        Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                    }
                } else if latest_number >= current_block_number {
                    if self.sync_metadata.state_done() {
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
                        Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                    }
                } else {
                    Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
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

pub fn to_block_chain_collection<C, S>(
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    storage: Arc<S>,
) -> Result<Arc<BlockChainCollection<C, S>>>
where
    C: Consensus,
    S: Store + 'static,
{
    let master_chain_info = *startup_info.get_master();
    let collection = Arc::new(BlockChainCollection::new(startup_info, storage.clone()));
    let master = BlockChain::new(
        config,
        master_chain_info,
        storage,
        Arc::downgrade(&collection),
    )?;
    collection.init_master(master);

    Ok(collection)
}
