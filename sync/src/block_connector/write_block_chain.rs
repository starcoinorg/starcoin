// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::metrics::ChainMetrics;
use anyhow::{format_err, Result};
use config::NodeConfig;
use executor::VMMetrics;
use logger::prelude::*;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, WriteableChainService};
use starcoin_crypto::HashValue;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::Store;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::BlockInfo;
use starcoin_types::{
    block::{Block, BlockHeader, ExecutedBlock},
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
};
use starcoin_vm_types::on_chain_config::GlobalTimeOnChain;
use std::fmt::Formatter;
use std::sync::Arc;

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
    metrics: Option<ChainMetrics>,
    vm_metrics: Option<VMMetrics>,
}

#[derive(Copy, Clone, Debug)]
pub enum ConnectOk {
    Duplicate,
    //Execute block and connect to main
    ExeConnectMain,
    //Execute block and connect to branch.
    ExeConnectBranch,
    //Block has executed, just connect.
    Connect,
}

impl std::fmt::Display for ConnectOk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConnectOk::Duplicate => "Duplicate",
            ConnectOk::ExeConnectMain => "ExeConnectMain",
            ConnectOk::ExeConnectBranch => "ExeConnectBranch",
            ConnectOk::Connect => "Connect",
        };
        write!(f, "{}", s)
    }
}

impl<P> WriteableChainService for WriteBlockChainService<P>
where
    P: TxPoolSyncService + 'static,
{
    fn try_connect(&mut self, block: Block) -> Result<()> {
        let _timer = self
            .metrics
            .as_ref()
            .map(|metrics| metrics.chain_block_connect_time.start_timer());

        let result = self.connect_inner(block);

        if let Some(metrics) = self.metrics.as_ref() {
            let result_str = match result.as_ref() {
                Ok(connect) => format!("Ok_{}", connect),
                Err(err) => {
                    if let Some(connect_err) = err.downcast_ref::<ConnectBlockError>() {
                        format!("Err_{}", connect_err.reason())
                    } else {
                        "Err_other".to_string()
                    }
                }
            };
            metrics
                .chain_block_connect_total
                .with_label_values(&[result_str.as_str()])
                .inc();
        }
        result.map(|_| ())
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
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let net = config.net();
        let main = BlockChain::new(
            net.time_service(),
            startup_info.main,
            storage.clone(),
            vm_metrics.clone(),
        )?;
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| ChainMetrics::register(registry).ok());

        Ok(Self {
            config,
            startup_info,
            main,
            storage,
            txpool,
            bus,
            metrics,
            vm_metrics,
        })
    }

    fn find_or_fork(
        &self,
        header: &BlockHeader,
    ) -> Result<(Option<BlockInfo>, Option<BlockChain>)> {
        let block_id = header.id();
        let block_info = self.storage.get_block_info(block_id)?;
        let block_chain = if block_info.is_some() {
            if self.is_main_head(&header.parent_hash()) {
                None
            } else {
                let net = self.config.net();
                Some(BlockChain::new(
                    net.time_service(),
                    block_id,
                    self.storage.clone(),
                    self.vm_metrics.clone(),
                )?)
            }
        } else if self.block_exist(header.parent_hash())? {
            let net = self.config.net();
            Some(BlockChain::new(
                net.time_service(),
                header.parent_hash(),
                self.storage.clone(),
                self.vm_metrics.clone(),
            )?)
        } else {
            None
        };
        Ok((block_info, block_chain))
    }

    fn block_exist(&self, block_id: HashValue) -> Result<bool> {
        Ok(matches!(self.storage.get_block_info(block_id)?, Some(_)))
    }

    pub fn get_main(&self) -> &BlockChain {
        &self.main
    }

    pub fn select_head(&mut self, new_branch: BlockChain) -> Result<()> {
        let executed_block = new_branch.head_block();
        let main_total_difficulty = self.main.get_total_difficulty()?;
        let branch_total_difficulty = new_branch.get_total_difficulty()?;
        let parent_is_main_head = self.is_main_head(&executed_block.header().parent_hash());

        if branch_total_difficulty > main_total_difficulty {
            let (enacted_count, enacted_blocks, retracted_count, retracted_blocks) =
                if !parent_is_main_head {
                    self.find_ancestors_from_accumulator(&new_branch)?
                } else {
                    (1, vec![executed_block.block.clone()], 0, vec![])
                };
            self.main = new_branch;

            self.do_new_head(
                executed_block,
                enacted_count,
                enacted_blocks,
                retracted_count,
                retracted_blocks,
            )?;
        } else {
            //send new branch event
            self.broadcast_new_branch(executed_block);
        }
        Ok(())
    }

    fn do_new_head(
        &mut self,
        executed_block: ExecutedBlock,
        enacted_count: u64,
        enacted_blocks: Vec<Block>,
        retracted_count: u64,
        retracted_blocks: Vec<Block>,
    ) -> Result<()> {
        debug_assert!(!enacted_blocks.is_empty());
        debug_assert_eq!(enacted_blocks.last().unwrap(), executed_block.block());
        self.update_startup_info(executed_block.block().header())?;
        if retracted_count > 0 {
            if let Some(metrics) = self.metrics.as_ref() {
                metrics.chain_rollback_block_total.inc_by(retracted_count);
            }
        }
        self.commit_2_txpool(enacted_blocks, retracted_blocks);
        self.config
            .net()
            .time_service()
            .adjust(GlobalTimeOnChain::new(executed_block.header().timestamp()));
        info!("[chain] Select new head, id: {}, number: {}, total_difficulty: {}, enacted_block_count: {}, retracted_block_count: {}", executed_block.header().id(), executed_block.header().number(), executed_block.block_info().total_difficulty, enacted_count, retracted_count);

        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_block_num
                .set(executed_block.block.header().number());

            metrics
                .chain_txn_num
                .set(executed_block.block_info.txn_accumulator_info.num_leaves);
        }

        self.broadcast_new_head(executed_block);

        Ok(())
    }

    /// Reset the node to `block_id`, and replay blocks after the block
    pub fn reset(&mut self, block_id: HashValue) -> Result<()> {
        let new_head_block = self
            .main
            .get_block(block_id)?
            .ok_or_else(|| format_err!("Can not find block {} in main chain", block_id,))?;
        let new_branch = BlockChain::new(
            self.config.net().time_service(),
            block_id,
            self.storage.clone(),
            self.vm_metrics.clone(),
        )?;

        // delete block since from block.number + 1 to latest.
        let start = new_head_block.header().number().saturating_add(1);
        let latest = self.main.status().head.number();
        for block_number in start..latest {
            if let Some(block) = self.main.get_block_by_number(block_number)? {
                info!("Delete block({:?})", block.header);
                self.storage.delete_block(block.id())?;
                self.storage.delete_block_info(block.id())?;
            } else {
                warn!("Can not find block by number:{}", block_number);
            }
        }
        let executed_block = new_branch.head_block();

        self.main = new_branch;

        let (enacted_count, enacted_blocks, retracted_count, retracted_blocks) =
            (1, vec![executed_block.block.clone()], 0, vec![]);
        self.do_new_head(
            executed_block,
            enacted_count,
            enacted_blocks,
            retracted_count,
            retracted_blocks,
        )?;
        Ok(())
    }

    ///Directly execute the block and save result, do not try to connect.
    pub fn execute(&mut self, block: Block) -> Result<ExecutedBlock> {
        let chain = BlockChain::new(
            self.config.net().time_service(),
            block.header().parent_hash(),
            self.storage.clone(),
            self.vm_metrics.clone(),
        )?;
        let verify_block = chain.verify(block)?;
        chain.execute(verify_block)
    }

    fn is_main_head(&self, parent_id: &HashValue) -> bool {
        parent_id == &self.startup_info.main
    }

    fn update_startup_info(&mut self, main_head: &BlockHeader) -> Result<()> {
        self.startup_info.update_main(main_head.id());
        self.storage.save_startup_info(self.startup_info.clone())
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
        let enacted_count = new_branch
            .current_header()
            .number()
            .checked_sub(ancestor_block.header().number())
            .ok_or_else(|| format_err!("current_header number should > ancestor_block number."))?;
        let retracted_count = self
            .main
            .current_header()
            .number()
            .checked_sub(ancestor_block.header().number())
            .ok_or_else(|| format_err!("current_header number should > ancestor_block number."))?;

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

    fn broadcast_new_head(&self, block: ExecutedBlock) {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_select_head_total
                .with_label_values(&["new_head"])
                .inc()
        }

        if let Err(e) = self.bus.broadcast(NewHeadBlock(Arc::new(block))) {
            error!("Broadcast NewHeadBlock error: {:?}", e);
        }
    }

    fn broadcast_new_branch(&self, block: ExecutedBlock) {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_select_head_total
                .with_label_values(&["new_branch"])
                .inc()
        }
        if let Err(e) = self.bus.broadcast(NewBranch(Arc::new(block))) {
            error!("Broadcast NewBranch error: {:?}", e);
        }
    }

    fn connect_inner(&mut self, block: Block) -> Result<ConnectOk> {
        let block_id = block.id();
        if self.main.current_header().id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(ConnectOk::Duplicate);
        }
        if self.main.current_header().id() == block.header().parent_hash()
            && !self.block_exist(block_id)?
        {
            let executed_block = self.main.apply(block)?;
            let enacted_blocks = vec![executed_block.block().clone()];
            self.do_new_head(executed_block, 1, enacted_blocks, 0, vec![])?;
            return Ok(ConnectOk::ExeConnectMain);
        }
        let (block_info, fork) = self.find_or_fork(block.header())?;
        match (block_info, fork) {
            //block has bean processed in some branch, so just trigger a head select.
            (Some(_block_info), Some(branch)) => {
                debug!(
                    "Block {} has bean processed, trigger head select, total_difficulty: {}",
                    block_id,
                    branch.get_total_difficulty()?
                );
                self.select_head(branch)?;
                Ok(ConnectOk::Duplicate)
            }
            //block has bean processed, and it parent is main chain ,so just connect it to main chain.
            (Some(block_info), None) => {
                let executed_block = self.main.connect(ExecutedBlock {
                    block: block.clone(),
                    block_info,
                })?;
                self.do_new_head(executed_block, 1, vec![block], 0, vec![])?;
                Ok(ConnectOk::Connect)
            }
            (None, Some(mut branch)) => {
                let _executed_block = branch.apply(block)?;
                self.select_head(branch)?;
                Ok(ConnectOk::ExeConnectBranch)
            }
            (None, None) => Err(ConnectBlockError::FutureBlock(Box::new(block)).into()),
        }
    }
}
