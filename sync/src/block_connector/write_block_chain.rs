// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::metrics::WRITE_BLOCK_CHAIN_METRICS;
use anyhow::{ensure, format_err, Result};
use chain::BlockChain;
use config::NodeConfig;
use logger::prelude::*;
use starcoin_crypto::HashValue;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::Store;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockDetail, BlockHeader},
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
};
use starcoin_vm_types::on_chain_config::GlobalTimeOnChain;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter, ConnectBlockError, WriteableChainService};

const MAX_ROLL_BACK_BLOCK: usize = 10;

pub struct WriteBlockChainService<P>
where
    P: TxPoolSyncService,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    main: BlockChain,
    storage: Arc<dyn Store>,
    txpool: P,
    bus: ServiceRef<BusService>,
}

impl<P> WriteableChainService for WriteBlockChainService<P>
where
    P: TxPoolSyncService + 'static,
{
    fn try_connect(&mut self, block: Block) -> Result<()> {
        self.connect_inner(block)
    }
}

impl<P> WriteBlockChainService<P>
where
    P: TxPoolSyncService + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: P,
        bus: ServiceRef<BusService>,
    ) -> Result<Self> {
        let net = config.net();
        let main = BlockChain::new(net.time_service(), startup_info.main, storage.clone())?;
        Ok(Self {
            config,
            startup_info,
            main,
            storage,
            txpool,
            bus,
        })
    }

    pub fn find_or_fork(&self, header: &BlockHeader) -> Result<(bool, Option<BlockChain>)> {
        WRITE_BLOCK_CHAIN_METRICS.try_connect_count.inc();
        let block_id = header.id();
        let block_exist = self.block_exist(block_id);
        let block_chain = if block_exist {
            if self.is_main_head(&header.parent_hash()) {
                None
            } else {
                let net = self.config.net();
                Some(BlockChain::new(
                    net.time_service(),
                    block_id,
                    self.storage.clone(),
                )?)
            }
        } else if self.block_exist(header.parent_hash()) {
            let net = self.config.net();
            Some(BlockChain::new(
                net.time_service(),
                header.parent_hash(),
                self.storage.clone(),
            )?)
        } else {
            None
        };
        Ok((block_exist, block_chain))
    }

    fn block_exist(&self, block_id: HashValue) -> bool {
        if let Ok(Some(_)) = self.storage.get_block_info(block_id) {
            true
        } else {
            false
        }
    }

    pub fn get_main(&self) -> &BlockChain {
        &self.main
    }

    pub fn select_head(&mut self, new_branch: BlockChain) -> Result<()> {
        let block = new_branch.head_block();
        let block_header = block.header().clone();
        let main_total_difficulty = self.main.get_total_difficulty()?;
        let branch_total_difficulty = new_branch.get_total_difficulty()?;
        let mut map_be_uncles = Vec::new();
        let parent_is_main_head = self.is_main_head(&block_header.parent_hash());
        if branch_total_difficulty > main_total_difficulty {
            let (enacted_count, enacted_blocks, retracted_count, retracted_blocks) =
                if !parent_is_main_head {
                    self.find_ancestors_from_accumulator(&new_branch)?
                } else {
                    (1, vec![block.clone()], 0, vec![])
                };
            self.main = new_branch;
            self.do_new_head(
                block,
                enacted_count,
                enacted_blocks,
                retracted_count,
                retracted_blocks,
            )?;
        } else {
            //send new branch event
            map_be_uncles.push(block_header);
            self.broadcast_new_branch(map_be_uncles);
        }

        self.save_startup()
    }

    pub fn do_new_head(
        &mut self,
        new_head_block: Block,
        enacted_count: u64,
        enacted_blocks: Vec<Block>,
        retracted_count: u64,
        retracted_blocks: Vec<Block>,
    ) -> Result<()> {
        debug_assert!(!enacted_blocks.is_empty());
        debug_assert_eq!(enacted_blocks.last().unwrap(), &new_head_block);
        self.startup_info.update_main(new_head_block.header());
        if retracted_count > 0 {
            WRITE_BLOCK_CHAIN_METRICS
                .rollback_block_size
                .set(retracted_count as i64);
        }
        self.commit_2_txpool(enacted_blocks, retracted_blocks);
        WRITE_BLOCK_CHAIN_METRICS.broadcast_head_count.inc();
        self.config
            .net()
            .time_service()
            .adjust(GlobalTimeOnChain::new(new_head_block.header().timestamp));
        let block_info = self
            .storage
            .get_block_info(new_head_block.id())?
            .ok_or_else(|| format_err!("Can not find block info by id {}", new_head_block.id()))?;
        info!("[chain] Select new head, id: {}, number: {}, total_difficulty: {}, enacted_block_count: {}, retracted_block_count: {}", new_head_block.id(), new_head_block.header().number, block_info.total_difficulty, enacted_count, retracted_count);

        self.broadcast_new_head(BlockDetail::new(
            new_head_block,
            block_info.total_difficulty,
        ));
        Ok(())
    }

    fn is_main_head(&self, parent_id: &HashValue) -> bool {
        parent_id == &self.startup_info.main
    }

    fn save_startup(&self) -> Result<()> {
        let startup_info = self.startup_info.clone();
        self.storage.save_startup_info(startup_info)
    }

    fn commit_2_txpool(&self, enacted: Vec<Block>, retracted: Vec<Block>) {
        if let Err(e) = self.txpool.chain_new_block(enacted, retracted) {
            error!("rollback err : {:?}", e);
        }
    }

    fn find_ancestors_from_accumulator(
        &self,
        new_branch: &BlockChain,
    ) -> Result<(u64, Vec<Block>, u64, Vec<Block>)> {
        let ancestor = self.main.find_ancestor(new_branch)?.ok_or_else(|| {
            format_err!(
                "Can not find ancestors between main chain: {:?} and branch: {:?}",
                self.main.status(),
                new_branch.status()
            )
        })?;

        let ancestor_block = self
            .main
            .get_block(ancestor.id)?
            .ok_or_else(|| format_err!("Can not find block by id:{}", ancestor.id))?;
        let enacted_count = new_branch.current_header().number() - ancestor_block.header().number();
        let retracted_count =
            self.main.current_header().number() - ancestor_block.header().number();

        let block_enacted = new_branch.current_header().id();
        let block_retracted = self.main.current_header().id();

        let enacted = self.find_blocks_until(block_enacted, ancestor.id, MAX_ROLL_BACK_BLOCK)?;
        let retracted =
            self.find_blocks_until(block_retracted, ancestor.id, MAX_ROLL_BACK_BLOCK)?;

        debug!(
            "Commit block count:{}, rollback block count:{}",
            enacted_count, retracted_count,
        );
        Ok((enacted_count, enacted, retracted_count, retracted))
    }

    fn find_blocks_until(
        &self,
        from: HashValue,
        until: HashValue,
        max_size: usize,
    ) -> Result<Vec<Block>> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut block_id = from;
        loop {
            if block_id == until {
                break;
            }
            if blocks.len() >= max_size {
                break;
            }
            let block = self
                .storage
                .get_block(block_id)?
                .ok_or_else(|| format_err!("Can not find block {:?}.", block_id))?;
            block_id = block.header().parent_hash();
            blocks.push(block);
        }
        blocks.reverse();

        Ok(blocks)
    }

    fn broadcast_new_head(&self, block: BlockDetail) {
        if let Err(e) = self.bus.broadcast(NewHeadBlock(Arc::new(block))) {
            error!("Broadcast NewHeadBlock error: {:?}", e);
        }
    }

    fn broadcast_new_branch(&self, maybe_uncles: Vec<BlockHeader>) {
        if let Err(e) = self.bus.broadcast(NewBranch(Arc::new(maybe_uncles))) {
            error!("Broadcast NewBranch error: {:?}", e);
        }
    }

    fn connect_inner(&mut self, block: Block) -> Result<()> {
        let block_id = block.id();
        if self.main.current_header().id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(());
        }
        if self.main.current_header().id() == block.header().parent_hash()
            && !self.block_exist(block_id)
        {
            let connected = self.main.apply(block.clone());
            if connected.is_err() {
                debug!("connected failed {:?}", block_id);
                WRITE_BLOCK_CHAIN_METRICS.verify_fail_count.inc();
            } else {
                self.do_new_head(block.clone(), 1, vec![block], 0, vec![])?;
            }
            return connected;
        }
        let (block_exist, fork) = self.find_or_fork(block.header())?;
        match (block_exist, fork) {
            //block has bean processed, so just trigger a head select.
            (true, Some(branch)) => {
                debug!(
                    "Block {} has bean processed, trigger head select, total_difficulty: {}",
                    block_id,
                    branch.get_total_difficulty()?
                );
                WRITE_BLOCK_CHAIN_METRICS.duplicate_conn_count.inc();
                self.select_head(branch)?;
                Ok(())
            }
            (true, None) => {
                self.main.update_chain_head(block.clone())?;
                self.do_new_head(block.clone(), 1, vec![block], 0, vec![])?;
                Ok(())
            }
            (false, Some(mut branch)) => {
                let timer = WRITE_BLOCK_CHAIN_METRICS
                    .exe_block_time
                    .with_label_values(&["time"])
                    .start_timer();
                let connected = branch.apply(block);
                timer.observe_duration();
                if connected.is_err() {
                    debug!("connected failed {:?}", block_id);
                    WRITE_BLOCK_CHAIN_METRICS.verify_fail_count.inc();
                } else {
                    self.select_head(branch)?;
                }
                connected
            }
            (false, None) => Err(ConnectBlockError::FutureBlock(Box::new(block)).into()),
        }
    }
}
