// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::metrics::ChainMetrics;
use anyhow::{bail, format_err, Ok, Result};
use async_std::stream::StreamExt;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, WriteableChainService};
use starcoin_config::NodeConfig;
use starcoin_consensus::dag::ghostdag::protocol::ColoringOutput;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::Store;
use starcoin_time_service::{DagBlockTimeWindowService, TimeWindowResult};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::BlockInfo;
use starcoin_types::blockhash::BlockHashMap;
use starcoin_types::{
    block::{Block, BlockHeader, ExecutedBlock},
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
};
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};

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
    dag_block_pool: Arc<Mutex<Vec<(Block, Vec<HashValue>)>>>,
    dag: Arc<Mutex<BlockDAG>>,
}

#[derive(Clone, Debug)]
pub enum ConnectOk {
    Duplicate(ExecutedBlock),
    //Execute block and connect to main
    ExeConnectMain(ExecutedBlock),
    //Execute block and connect to branch.
    ExeConnectBranch(ExecutedBlock),
    //Block has executed, just connect.
    Connect(ExecutedBlock),
    
    //Block has executed, just connect.
    DagConnected,
    // the retry block
    MainDuplicate,
}

impl ConnectOk {
    pub fn block(&self) -> Option<ExecutedBlock> {
        match self {
            ConnectOk::Duplicate(block) => Some(block.clone()),
            ConnectOk::ExeConnectMain(block) => Some(block.clone()),
            ConnectOk::ExeConnectBranch(block) => Some(block.clone()),
            ConnectOk::Connect(block) => Some(block.clone()),
            ConnectOk::DagConnected => None,
            ConnectOk::MainDuplicate => None,
        }
    }
}

impl std::fmt::Display for ConnectOk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConnectOk::Duplicate(_) => "Duplicate",
            ConnectOk::ExeConnectMain(_) => "ExeConnectMain",
            ConnectOk::ExeConnectBranch(_) => "ExeConnectBranch",
            ConnectOk::Connect(_) => "Connect",
            ConnectOk::DagConnected => "DagConnect",
            ConnectOk::MainDuplicate => "MainDuplicate",
        };
        write!(f, "{}", s)
    }
}

impl<P> WriteableChainService for WriteBlockChainService<P>
where
    P: TxPoolSyncService + 'static,
{
    fn try_connect(&mut self, block: Block, tips_headers: Option<Vec<HashValue>>) -> Result<()> {
        let _timer = self
            .metrics
            .as_ref()
            .map(|metrics| metrics.chain_block_connect_time.start_timer());

        let result = self.connect_inner(block, tips_headers);

        if let Some(metrics) = self.metrics.as_ref() {
            let result = match result.as_ref() {
                std::result::Result::Ok(connect) => format!("Ok_{}", connect),
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
                .with_label_values(&[result.as_str()])
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
        dag: Arc<Mutex<BlockDAG>>,
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
            dag_block_pool: Arc::new(Mutex::new(vec![])),
            dag,
        })
    }

    fn find_or_fork(
        &self,
        header: &BlockHeader,
        dag_block_parents: Option<Vec<HashValue>>,
    ) -> Result<(Option<BlockInfo>, Option<BlockChain>)> {
        let block_id = header.id();
        let block_info = self.storage.get_block_info(block_id)?;
        let block_chain = if block_info.is_some() {
            if self.is_main_head(&header.parent_hash(), dag_block_parents) {
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
        } else if self.block_exist(header.parent_hash())? || self.blocks_exist(dag_block_parents)? {
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

    fn blocks_exist(&self, block_id: Option<Vec<HashValue>>) -> Result<bool> {
        if let Some(block_id) = block_id {
            if block_id.is_empty() {
                return Ok(false);
            }
            return Ok(matches!(self.storage.get_block_infos(block_id)?, _));
        }
        return Ok(false);
    }

    pub fn get_main(&self) -> &BlockChain {
        &self.main
    }

    pub fn select_head(
        &mut self,
        new_branch: BlockChain,
        dag_block_parents: Option<Vec<HashValue>>,
    ) -> Result<()> {
        let executed_block = new_branch.head_block();
        let main_total_difficulty = self.main.get_total_difficulty()?;
        let branch_total_difficulty = new_branch.get_total_difficulty()?;
        let parent_is_main_head = self.is_main_head(
            &executed_block.header().parent_hash(),
            dag_block_parents.clone(),
        );

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
            self.broadcast_new_branch(executed_block, dag_block_parents);
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
            .adjust(executed_block.header().timestamp());
        info!("[chain] Select new head, id: {}, number: {}, total_difficulty: {}, enacted_block_count: {}, retracted_block_count: {}", executed_block.header().id(), executed_block.header().number(), executed_block.block_info().total_difficulty, enacted_count, retracted_count);

        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_block_num
                .set(executed_block.block.header().number());

            metrics
                .chain_txn_num
                .set(executed_block.block_info.txn_accumulator_info.num_leaves);
        }

        // self.broadcast_new_head(executed_block, dag_parents, next_tips);

        Ok(())
    }

    /// Reset the node to `block_id`, and replay blocks after the block
    pub fn reset(
        &mut self,
        block_id: HashValue,
        dag_block_parents: Option<Vec<HashValue>>,
    ) -> Result<()> {
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
            executed_block.clone(),
            enacted_count,
            enacted_blocks,
            retracted_count,
            retracted_blocks,
        )?;

        let next_tips = self.storage.get_tips_by_block_id(executed_block.block.header().id()).ok();
        self.broadcast_new_head(executed_block, dag_block_parents, next_tips);

        Ok(())
    }

    ///Directly execute the block and save result, do not try to connect.
    pub fn execute(
        &mut self,
        block: Block,
        dag_block_parent: Option<HashValue>,
    ) -> Result<ExecutedBlock> {
        let chain = BlockChain::new(
            self.config.net().time_service(),
            block.header().parent_hash(),
            self.storage.clone(),
            self.vm_metrics.clone(),
        )?;
        let verify_block = chain.verify(block)?;
        chain.execute(verify_block, dag_block_parent)
    }

    fn is_main_head(
        &self,
        parent_id: &HashValue,
        dag_block_parents: Option<Vec<HashValue>>,
    ) -> bool {
        if parent_id == &self.startup_info.main {
            return true;
        }

        if let Some(block_parents) = dag_block_parents {
            if self.main.status().tips_hash.is_some() && !block_parents.is_empty() {
                return block_parents.into_iter().all(|block_header| {
                    self.main
                        .status()
                        .tips_hash
                        .unwrap()
                        .contains(&block_header)
                });
            }
        }

        return false;
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

    fn broadcast_new_head(
        &self,
        block: ExecutedBlock,
        dag_parents: Option<Vec<HashValue>>,
        next_tips: Option<Vec<HashValue>>,
    ) {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_select_head_total
                .with_label_values(&["new_head"])
                .inc()
        }

        if let Err(e) = self
            .bus
            .broadcast(NewHeadBlock(Arc::new(block), dag_parents, next_tips))
        {
            error!("Broadcast NewHeadBlock error: {:?}", e);
        }
    }

    fn broadcast_new_branch(
        &self,
        block: ExecutedBlock,
        dag_block_parents: Option<Vec<HashValue>>,
    ) {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_select_head_total
                .with_label_values(&["new_branch"])
                .inc()
        }
        if let Err(e) = self
            .bus
            .broadcast(NewBranch(Arc::new(block), dag_block_parents))
        {
            error!("Broadcast NewBranch error: {:?}", e);
        }
    }

    fn switch_branch(
        &mut self,
        block: Block,
        dag_block_parents: Option<Vec<HashValue>>,
        dag_block_next_parent: Option<HashValue>,
        next_tips: &mut Option<Vec<HashValue>>,
    ) -> Result<ConnectOk> {
        let (block_info, fork) = self.find_or_fork(block.header(), dag_block_parents.clone())?;
        match (block_info, fork) {
            //block has been processed in some branch, so just trigger a head selection.
            (Some(block_info), Some(branch)) => {
                // both are different, select one
                debug!(
                    "Block {} has been processed, trigger head selection, total_difficulty: {}",
                    block_info.block_id(),
                    branch.get_total_difficulty()?
                );
                let exe_block = branch.head_block();
                self.select_head(branch, dag_block_parents)?;
                Ok(ConnectOk::Duplicate(exe_block))
            }
            //block has been processed, and its parent is main chain, so just connect it to main chain.
            (Some(block_info), None) => {
                // both are identical
                let block_id: HashValue = block_info.block_id().clone();
                let executed_block = self.main.connect(
                    ExecutedBlock {
                        block: block.clone(),
                        block_info,
                        dag_parent: dag_block_next_parent,
                    },
                    next_tips,
                )?;
                info!(
                    "Block {} main has been processed, trigger head selection",
                    block_id,
                );
                self.do_new_head(
                    executed_block.clone(),
                    1,
                    vec![block],
                    0,
                    vec![],
                )?;
                Ok(ConnectOk::Connect(executed_block))
            }
            (None, Some(mut branch)) => {
                // the block is not in the block, but the parent is
                let executed_block = branch.apply(block, dag_block_next_parent, next_tips)?;
                self.select_head(branch, dag_block_parents)?;
                Ok(ConnectOk::ExeConnectBranch(executed_block))
            }
            (None, None) => Err(ConnectBlockError::FutureBlock(Box::new(block)).into()),
        }
    }

    fn connect_to_main(
        &mut self,
        block: Block,
        dag_block_parents: Option<Vec<HashValue>>,
        dag_block_next_parent: Option<HashValue>,
        next_tips: &mut Option<Vec<HashValue>>,
    ) -> Result<ConnectOk> {
        let block_id = block.id();
        if block_id == *starcoin_storage::BARNARD_HARD_FORK_HASH
            && block.header().number() == starcoin_storage::BARNARD_HARD_FORK_HEIGHT
        {
            debug!("barnard hard fork {}", block_id);
            return Err(ConnectBlockError::BarnardHardFork(Box::new(block)).into());
        }
        if self.main.current_header().id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(ConnectOk::MainDuplicate);
        }
        if let Some(parents) = dag_block_parents {
            assert!(parents.len() > 0);
            // let color = self
            //     .dag
            //     .lock()
            //     .as_mut()
            //     .expect("failed to get the mut dag object")
            //     .commit_header(&Header::new(block.header().clone(), parents.clone()))?;
            let color = ColoringOutput::Blue(
                0,
                BlockHashMap::with_capacity(3), // k
            );
            match color {
                ColoringOutput::Blue(_, _) => {
                    if self
                        .main
                        .current_tips_hash()
                        .expect("in dag block, the tips hash must exist")
                        == block.header().parent_hash()
                        && !self.block_exist(block_id)?
                    {
                        return self.apply_and_select_head(
                            block,
                            Some(parents),
                            dag_block_next_parent,
                            next_tips,
                        );
                    }
                    self.switch_branch(block, Some(parents), dag_block_next_parent, next_tips)
                }
                ColoringOutput::Red => {
                    self.switch_branch(block, Some(parents), dag_block_next_parent, next_tips)
                }
            }
        } else {
            if self.main.current_header().id() == block.header().parent_hash()
                && !self.block_exist(block_id)?
            {
                return self.apply_and_select_head(block, None, None, &mut None);
            }
            // todo: should switch dag together
            self.switch_branch(block, None, None, &mut None)
        }
    }

    fn apply_and_select_head(
        &mut self,
        block: Block,
        dag_block_parents: Option<Vec<HashValue>>,
        dag_block_next_parent: Option<HashValue>,
        next_tips: &mut Option<Vec<HashValue>>,
    ) -> Result<ConnectOk> {
        let executed_block = self.main.apply(block, dag_block_next_parent, next_tips)?;
        let enacted_blocks = vec![executed_block.block().clone()];
        self.do_new_head(
            executed_block.clone(),
            1,
            enacted_blocks,
            0,
            vec![],
        )?;
        return Ok(ConnectOk::ExeConnectMain(executed_block));
    }

    fn connect_inner(
        &mut self,
        block: Block,
        tips_headers: Option<Vec<HashValue>>,
    ) -> Result<ConnectOk> {
        let block_id = block.id();
        if block_id == *starcoin_storage::BARNARD_HARD_FORK_HASH
            && block.header().number() == starcoin_storage::BARNARD_HARD_FORK_HEIGHT
        {
            debug!("barnard hard fork {}", block_id);
            return Err(ConnectBlockError::BarnardHardFork(Box::new(block)).into());
        }
        if self.main.current_header().id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(ConnectOk::MainDuplicate);
        }

        // if it received a block with tips, it is a dag block
        if let Some(dag_block_parents) = tips_headers {
            // tips header, check the dag time window to see if it is should apply the blocks
            // checkout if it is time to settle down
            let time_service = DagBlockTimeWindowService::new(15 * 1000);
            match time_service.is_in_time_window(
                block.header().timestamp(),
                self.config.net().time_service().clone(),
            ) {
                // TimeWindowResult::InTimeWindow => {
                //     self.dag_block_pool
                //         .lock()
                //         .unwrap()
                //         .push((block, dag_block_parents));
                //     return Ok(ConnectOk::DagPending);
                // }
                // TimeWindowResult::BeforeTimeWindow => {
                //     return Err(ConnectBlockError::DagBlockBeforeTimeWindow(Box::new(block)).into())
                // }
                // TimeWindowResult::AfterTimeWindow => {
                _ => {
                    // dump the block in the time window pool and put the block into the next time window pool
                    let block_id = block.header.id();
                    self.dag_block_pool
                        .lock()
                        .unwrap()
                        .push((block, dag_block_parents.clone()));
                    let _testing = self.dag.lock().unwrap().push_parent_children(block_id, Arc::new(dag_block_parents.clone()));
                    self.main.status().tips_hash = None; // set the tips to None, and in connect_to_main, the block will be added to the tips
                    let mut dag_blocks = self.dag_block_pool.lock().unwrap().clone();
                    self.dag_block_pool.lock().unwrap().clear();
                    // sort by id
                    dag_blocks.sort_by_key(|(block, _)| block.header().id());

                    let mut dag_block_next_parent = dag_block_parents
                        .iter()
                        .max()
                        .expect("tips must be larger than 0")
                        .clone();
                    let mut next_tips = Some(vec![]);
                    let mut executed_blocks = vec![];
                    // connect the block one by one
                    dag_blocks
                        .into_iter()
                        .for_each(|(block, dag_block_parents)| {
                            let next_transaction_parent = block.header().id();
                            let result = self.connect_to_main(
                                block,
                                Some(dag_block_parents),
                                Some(dag_block_next_parent),
                                &mut next_tips,
                            );
                            match result {
                                std::result::Result::Ok(connect_ok) => {
                                    executed_blocks.push(connect_ok.block().clone());
                                    dag_block_next_parent = next_transaction_parent;
                                }
                                Err(error) => error!(
                                    "apply_and_select_head failed, error: {}",
                                    error.to_string()
                                ),
                            }
                        });
                    
                    match next_tips {
                        Some(new_tips) => {
                            if new_tips.is_empty() {
                                bail!("no new block has been executed successfully!");
                            }

                            // 1, write to disc
                            self.main.append_dag_accumulator_leaf(new_tips.clone()).expect("failed to append new tips to dag accumulator");

                            // 2, broadcast the blocks sorted by their id
                            executed_blocks.into_iter().for_each(|exe_block| {
                                if let Some(block) = exe_block {
                                    self.broadcast_new_head(block.clone(), Some(dag_block_parents.clone()), Some(new_tips.clone()));
                                }
                            });
                        }
                        None => {
                            unreachable!("next tips should not be None");
                        }
                    }

                    return Ok(ConnectOk::DagConnected);
                }
            }
        } else {
            // normal block, just connect to main
            let mut next_tips = Some(vec![]);
            let executed_block = self.connect_to_main(block, None, None, &mut next_tips)?.clone();
            if let Some(block) = executed_block.block() {
                self.broadcast_new_head(block.clone(), None, None);
            }
            return Ok(executed_block);
        }
    }
}
