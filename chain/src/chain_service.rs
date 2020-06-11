// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{chain::BlockChain, chain_metrics::CHAIN_METRICS};
use actix::prelude::*;
use anyhow::{format_err, Error, Result};
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_statedb::ChainStateDB;
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool_api::TxPoolSyncService;
use std::sync::Arc;
use storage::Store;
use traits::{
    is_ok, ChainReader, ChainService, ChainWriter, ConnectBlockError, ConnectResult, Consensus,
};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockDetail, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    startup_info::StartupInfo,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
    BLOCK_PROTOCOL_NAME,
};

pub struct ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolSyncService + 'static,
    S: Store + 'static,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    master: BlockChain<C, S>,
    storage: Arc<S>,
    network: Option<NetworkAsyncService>,
    txpool: P,
    bus: Addr<BusActor>,
    sync_metadata: SyncMetadata,
}

impl<C, S, P> ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolSyncService + 'static,
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
        let master = BlockChain::new(config.clone(), startup_info.master, storage.clone())?;
        Ok(Self {
            config,
            startup_info,
            master,
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

    pub fn get_master(&self) -> &BlockChain<C, S> {
        &self.master
    }

    fn select_head(&mut self, new_branch: BlockChain<C, S>) -> Result<()> {
        let block = new_branch.head_block();
        let block_header = block.header();
        let total_difficulty = new_branch.get_total_difficulty()?;
        if total_difficulty > self.get_master().get_total_difficulty()? {
            let (enacted_blocks, retracted_blocks) =
                if block.header().parent_hash() == self.startup_info.master {
                    (vec![block.clone()], vec![])
                } else {
                    // TODO: After review the impl of find_common_ancestor in storage.
                    // we can just let find_ancestors do it work, no matter whether fork or not.
                    self.find_ancestors(&new_branch)?
                };

            debug_assert!(!enacted_blocks.is_empty());
            debug_assert_eq!(enacted_blocks.last().unwrap(), &block);

            self.update_master(new_branch);
            self.commit_2_txpool(enacted_blocks, retracted_blocks);

            if self.sync_metadata.is_sync_done() {
                CHAIN_METRICS.broadcast_head_count.inc();
                let block_detail = BlockDetail::new(block, total_difficulty);
                self.broadcast_2_bus(block_detail.clone());
                self.broadcast_2_network(block_detail);
            }
        } else {
            self.insert_branch(block_header);
        }

        CHAIN_METRICS
            .branch_total_count
            .set(self.startup_info.branches.len() as i64);
        self.save_startup()
    }

    fn update_master(&mut self, new_master: BlockChain<C, S>) {
        let header = new_master.current_header();
        self.master = new_master;
        self.startup_info.update_master(&header);
    }

    fn insert_branch(&mut self, new_block_header: &BlockHeader) {
        self.startup_info.insert_branch(new_block_header);
    }

    fn save_startup(&self) -> Result<()> {
        let startup_info = self.startup_info.clone();
        self.storage.save_startup_info(startup_info)
    }

    fn commit_2_txpool(&self, enacted: Vec<Block>, retracted: Vec<Block>) {
        if let Err(e) = self.txpool.rollback(enacted, retracted) {
            error!("rollback err : {:?}", e);
        }
    }

    fn find_ancestors(&self, new_branch: &BlockChain<C, S>) -> Result<(Vec<Block>, Vec<Block>)> {
        let block_enacted = new_branch.current_header().id();
        let block_retracted = self.get_master().current_header().id();

        let ancestor = self
            .storage
            .get_common_ancestor(block_enacted, block_retracted)?
            .ok_or_else(|| {
                format_err!(
                    "Can not find ancestor with {:?} and {:?}.",
                    block_enacted,
                    block_retracted
                )
            })?;

        let enacted = self.find_blocks_until(block_enacted, ancestor)?;
        let retracted = self.find_blocks_until(block_retracted, ancestor)?;
        debug!(
            "commit block num:{}, rollback block num:{}",
            enacted.len(),
            retracted.len(),
        );
        Ok((enacted, retracted))
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
                .ok_or_else(|| format_err!("Can not find block {:?}.", tmp))?;
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
                let block_id = block.header().id();
                if let Err(e) = network
                    .broadcast_new_head_block(
                        BLOCK_PROTOCOL_NAME.into(),
                        NewHeadBlock(Arc::new(block)),
                    )
                    .await
                {
                    error!("broadcast new head block {:?} failed : {:?}", block_id, e);
                }
            });
        };
    }
}

impl<C, S, P> ChainService for ChainServiceImpl<C, S, P>
where
    C: Consensus,
    P: TxPoolSyncService,
    S: Store,
{
    //TODO define connect result.
    fn try_connect(&mut self, block: Block, pivot_sync: bool) -> Result<ConnectResult<()>> {
        if !self.sync_metadata.state_syncing() || pivot_sync {
            if !self.sync_metadata.state_syncing()
                || (pivot_sync && self.sync_metadata.state_done())
            {
                let (block_exist, fork) = self.find_or_fork(block.header())?;
                if block_exist {
                    CHAIN_METRICS.duplicate_conn_count.inc();
                    Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                } else if let Some(mut branch) = fork {
                    let timer = CHAIN_METRICS
                        .exe_block_time
                        .with_label_values(&["time"])
                        .start_timer();
                    let connected = branch.apply(block.clone())?;
                    timer.observe_duration();
                    if !connected {
                        debug!("connected failed {:?}", block.header().id());
                        CHAIN_METRICS.verify_fail_count.inc();
                        Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                    } else {
                        self.select_head(branch)?;
                        Ok(ConnectResult::Ok(()))
                    }
                } else {
                    Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                }
            } else {
                Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
            }
        } else {
            Ok(ConnectResult::Err(ConnectBlockError::Other(format!(
                "error connect type. pivot_sync : {}, block header : {:?}, sync metadata : {:?}.",
                pivot_sync,
                block.header(),
                self.sync_metadata
            ))))
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
                        self.sync_metadata.set_pivot_block(block, block_info)?;
                        return Ok(ConnectResult::Err(ConnectBlockError::Other(
                            "pivot block wait state.".to_string(),
                        )));
                    }
                    //todo:1. verify block header / verify accumulator / total difficulty
                    let (block_exist, fork) = self.find_or_fork(block.header())?;
                    if block_exist {
                        CHAIN_METRICS.duplicate_conn_count.inc();
                        Ok(ConnectResult::Err(ConnectBlockError::DuplicateConn))
                    } else if let Some(mut branch) = fork {
                        if C::verify(self.config.clone(), &branch, block.header()).is_ok() {
                            // 2. commit block
                            branch.append(
                                block.id(),
                                block_info.get_block_accumulator_info().clone(),
                            )?;
                            branch.commit(block, block_info, BlockState::Verified)?;
                            self.select_head(branch)?;
                            if pivot_flag {
                                self.sync_metadata.pivot_connected_succ()?;
                            }
                            Ok(ConnectResult::Ok(()))
                        } else {
                            Ok(ConnectResult::Err(ConnectBlockError::VerifyFailed))
                        }
                    } else {
                        Ok(ConnectResult::Err(ConnectBlockError::FutureBlock))
                    }
                } else if latest_number >= current_block_number {
                    if self.sync_metadata.state_done() {
                        let connect_result = self.try_connect(block, true)?;
                        // 3. update sync metadata
                        if latest_number == current_block_number && is_ok(&connect_result) {
                            if let Err(err) = self.sync_metadata.block_sync_done() {
                                error!(
                                    "update block_sync_done in sync_metadata failed : {:?}",
                                    err
                                );
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

    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn get_block_state_by_hash(&self, hash: HashValue) -> Result<Option<BlockState>> {
        self.storage.get_block_state(hash)
    }

    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>> {
        self.storage.get_block_info(hash)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.storage.get_transaction(txn_hash)
    }
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>, Error> {
        self.storage.get_block_transaction_infos(block_id)
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>, Error> {
        self.storage
            .get_transaction_info_by_block_and_index(block_id, idx)
    }

    fn master_head_header(&self) -> BlockHeader {
        self.get_master().current_header()
    }

    fn master_head_block(&self) -> Block {
        self.get_master().head_block()
    }

    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.get_master().get_block_by_number(number)
    }

    fn master_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        self.get_master().get_header_by_number(number)
    }
    fn master_startup_info(&self) -> StartupInfo {
        self.startup_info.clone()
    }

    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>> {
        self.get_master().get_blocks_by_number(number, count)
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
            None => self.get_master().current_header().id(),
        };

        if let Ok(Some(_)) = self.get_block_by_hash(block_id) {
            //TODO ensure is need create a new chain?
            let block_chain = self.get_master().new_chain(block_id)?;
            block_chain
                .create_block_template(author, auth_key_prefix, Some(block_id), user_txns)
                .map(|t| t.0)
        } else {
            Err(format_err!("Block {:?} not exist.", block_id))
        }
    }
}
