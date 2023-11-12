// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::metrics::ChainMetrics;
use anyhow::{bail, format_err, Ok, Result};
use parking_lot::Mutex;
use starcoin_chain::{BlockChain, ChainStatusWithBlock};
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, WriteableChainService};
use starcoin_config::NodeConfig;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{ServiceContext, ServiceRef};
use starcoin_storage::Store;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::BlockInfo;
use starcoin_types::consensus_header::DagHeader;
use starcoin_types::{
    block::{Block, BlockHeader, ExecutedBlock},
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
};
use std::{collections::HashMap, fmt::Formatter, sync::Arc};

use super::BlockConnectorService;

struct DagBranch {
    selected_parent: HashValue,
    status: ChainStatusWithBlock,
}

impl DagBranch {
    pub fn new(head: Block, selected_parent: HashValue, dag_tips: Vec<HashValue>) -> Self {
        todo!()
        //Self {
        //    selected_parent,
        //    status: ChainStatusWithBlock::new(block, dag_tips),
        //}
    }

    pub fn selected_parent(&self) -> HashValue {
        self.selected_parent
    }

    pub fn dag_tips(&self) -> Vec<HashValue> {
        self.status
            .dag_tips()
            .clone()
            .expect("invalid tips_hash for a dag branch")
    }

    pub fn appendable(&self, other: &DagBranch) -> Option<usize> {
        if self.selected_parent == other.selected_parent {
            match (self.status.dag_tips(), other.status.dag_tips()) {
                (None, _) | (_, None) => None,
                (Some(me), Some(other)) => {
                    todo!()
                }
            }
        } else {
            None
        }
    }
}

#[derive(Default)]
struct DagBranchesStatus {
    branches: HashMap<HashValue, DagBranch>,
}

const MAX_ROLL_BACK_BLOCK: usize = 10;

pub struct WriteBlockChainService<P>
where
    P: TxPoolSyncService,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    main: BlockChain,
    dag_branches: DagBranchesStatus,
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
    // the dag block waiting for the time window end
    DagPending,
    DagConnectMissingBlock,
}

impl ConnectOk {
    pub fn block(&self) -> Option<ExecutedBlock> {
        match self {
            ConnectOk::Duplicate(block) => Some(block.clone()),
            ConnectOk::ExeConnectMain(block) => Some(block.clone()),
            ConnectOk::ExeConnectBranch(block) => Some(block.clone()),
            ConnectOk::Connect(block) => Some(block.clone()),
            ConnectOk::DagConnected
            | ConnectOk::MainDuplicate
            | ConnectOk::DagPending
            | ConnectOk::DagConnectMissingBlock => None,
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
            ConnectOk::DagPending => "DagPending",
            ConnectOk::DagConnectMissingBlock => "DagConnectMissingBlock",
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

impl<TransactionPoolServiceT> WriteBlockChainService<TransactionPoolServiceT>
where
    TransactionPoolServiceT: TxPoolSyncService + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TransactionPoolServiceT,
        bus: ServiceRef<BusService>,
        vm_metrics: Option<VMMetrics>,
        dag: Arc<Mutex<BlockDAG>>,
    ) -> Result<Self> {
        let net = config.net();
        let main = BlockChain::new(
            net.time_service(),
            startup_info.main,
            storage.clone(),
            config.net().id().clone(),
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
            dag_branches: Default::default(),
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
                    net.id().clone(),
                    self.vm_metrics.clone(),
                )?)
            }
        } else if self.block_exist(header.parent_hash())? {
            let net = self.config.net();
            Some(BlockChain::new(
                net.time_service(),
                header.parent_hash(),
                self.storage.clone(),
                net.id().clone(),
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

    #[cfg(test)]
    pub fn time_sleep(&self, sec: u64) {
        self.config.net().time_service().sleep(sec * 1000000);
    }

    #[cfg(test)]
    pub fn apply_failed(&mut self, block: Block) -> Result<()> {
        use anyhow::bail;
        use starcoin_chain::verifier::FullVerifier;

        // apply but no connection
        let verified_block = self.main.verify_with_verifier::<FullVerifier>(block)?;
        let _executed_block = self.main.execute(verified_block)?;

        bail!("failed to apply for tesing the connection later!");
    }

    // for sync task to connect to its chain, if chain's total difficulties is larger than the main
    // switch by:
    // 1, update the startup info
    // 2, broadcast the new header
    pub fn switch_new_main(
        &mut self,
        new_head_block: HashValue,
        ctx: &mut ServiceContext<BlockConnectorService<TransactionPoolServiceT>>,
    ) -> Result<()>
    where
        TransactionPoolServiceT: TxPoolSyncService,
    {
        let new_branch = BlockChain::new(
            self.config.net().time_service(),
            new_head_block,
            self.storage.clone(),
            self.config.net().id().clone(),
            self.vm_metrics.clone(),
        )?;

        let main_total_difficulty = self.main.get_total_difficulty()?;
        let branch_total_difficulty = new_branch.get_total_difficulty()?;
        if branch_total_difficulty > main_total_difficulty {
            self.update_startup_info(new_branch.head_block().header())?;

            let dag_parents = self.dag.lock().get_parents(new_head_block)?;
            ctx.broadcast(NewHeadBlock(
                Arc::new(new_branch.head_block()),
                Some(dag_parents),
            ));

            Ok(())
        } else {
            bail!("no need to switch");
        }
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
        if enacted_blocks.is_empty() {
            error!("enacted_blocks is empty.");
            bail!("enacted_blocks is empty.");
        }
        if enacted_blocks.last().unwrap().header != executed_block.block().header {
            error!("enacted_blocks.last().unwrap().header: {:?}, executed_block.block().header: {:?} are different!", 
                    enacted_blocks.last().unwrap().header, executed_block.block().header);
            bail!("enacted_blocks.last().unwrap().header: {:?}, executed_block.block().header: {:?} are different!", 
                    enacted_blocks.last().unwrap().header, executed_block.block().header);
        }
        debug_assert!(!enacted_blocks.is_empty());
        debug_assert_eq!(
            enacted_blocks.last().unwrap().header,
            executed_block.block().header
        );
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

    pub fn do_new_head_with_broadcast() {}

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
            self.config.net().id().clone(),
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

        let next_tips = self
            .storage
            .get_tips_by_block_id(executed_block.block.header().id())
            .ok();
        self.broadcast_new_head(executed_block, dag_block_parents, next_tips);

        Ok(())
    }

    ///Directly execute the block and save result, do not try to connect.
    pub fn execute(&mut self, block: Block) -> Result<ExecutedBlock> {
        let chain = BlockChain::new(
            self.config.net().time_service(),
            block.header().parent_hash(),
            self.storage.clone(),
            self.config.net().id().clone(),
            self.vm_metrics.clone(),
        )?;
        let verify_block = chain.verify(block)?;
        chain.execute(verify_block)
    }

    fn is_main_head(&self, parent_id: &HashValue) -> bool {
        if parent_id == &self.startup_info.main {
            return true;
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

    fn find_ancestors_from_dag_accumulator(
        &self,
        new_branch: &BlockChain,
    ) -> Result<(u64, Vec<Block>, u64, Vec<Block>)> {
        let mut min_leaf_index = std::cmp::min(
            self.main.get_dag_current_leaf_number()?,
            new_branch.get_dag_current_leaf_number()?,
        ) - 1;

        let mut retracted = vec![];
        let mut enacted = vec![];

        let snapshot =
            new_branch.get_dag_accumulator_snapshot(new_branch.head_block().header().id())?;
        let mut children = snapshot.child_hashes.clone();
        children.sort();
        for child in children {
            match self.storage.get_block(child)? {
                Some(block) => enacted.push(block),
                None => bail!(
                    "the block{} dose not exist in new branch, ignore",
                    child.clone()
                ),
            }
        }
        enacted.reverse();

        loop {
            if min_leaf_index == 0 {
                break;
            }
            let main_snapshot = self
                .main
                .get_dag_accumulator_snapshot_by_index(min_leaf_index)?;
            let new_branch_snapshot =
                new_branch.get_dag_accumulator_snapshot_by_index(min_leaf_index)?;

            if main_snapshot.accumulator_info.get_accumulator_root()
                == new_branch_snapshot.accumulator_info.get_accumulator_root()
            {
                break;
            }

            let mut temp_retracted = vec![];
            temp_retracted.extend(
                main_snapshot
                    .child_hashes
                    .iter()
                    .try_fold(Vec::<Block>::new(), |mut rollback_blocks, child| {
                        let block = self.storage.get_block(child.clone());
                        if let anyhow::Result::Ok(Some(block)) = block {
                            rollback_blocks.push(block);
                        } else {
                            bail!(
                                "the block{} dose not exist in main branch, ignore",
                                child.clone()
                            );
                        }
                        return Ok(rollback_blocks);
                    })?
                    .into_iter(),
            );
            temp_retracted.sort_by(|a, b| b.header().id().cmp(&a.header().id()));
            retracted.extend(temp_retracted.into_iter());

            let mut temp_enacted = vec![];
            temp_enacted.extend(
                new_branch_snapshot
                    .child_hashes
                    .iter()
                    .try_fold(Vec::<Block>::new(), |mut rollback_blocks, child| {
                        let block = self.storage.get_block(child.clone());
                        if let anyhow::Result::Ok(Some(block)) = block {
                            rollback_blocks.push(block);
                        } else {
                            bail!(
                                "the block{} dose not exist in new branch, ignore",
                                child.clone()
                            );
                        }
                        return Ok(rollback_blocks);
                    })?
                    .into_iter(),
            );
            temp_enacted.sort_by(|a, b| b.header().id().cmp(&a.header().id()));
            enacted.extend(temp_enacted.into_iter());

            min_leaf_index = min_leaf_index.saturating_sub(1);
        }
        enacted.reverse();
        retracted.reverse();
        Ok((
            enacted.len() as u64,
            enacted,
            retracted.len() as u64,
            retracted,
        ))
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
            .broadcast(NewHeadBlock(Arc::new(block), dag_parents))
        {
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
        let dag_parents = block.block.header().clone().parents_hash();
        if let Err(e) = self.bus.broadcast(NewBranch(Arc::new(block), dag_parents)) {
            error!("Broadcast NewBranch error: {:?}", e);
        }
    }

    fn connect_dag_inner(&mut self, block: Block) -> Result<ConnectOk> {
        let block_id = block.id();

        self.dag.lock().commit(DagHeader::new(block.header))?;

        let (selected_parent, mine_tips) = block
            .uncles()
            .and_then(|u| u.split_first())
            .expect("uncles must full filled");

        // todo:
        // 1. keep tracking received blocks in a multi-queue
        // 2. for a block, check if [selected_parent, ... mergeset_blues] is one queue
        // 2a. if there is a existing queue, append the block to it.
        // 2b. if there is a shorter queue, extend it.
        // 2c. if not, create a new queue, and append the block to it
        // 3. select the longest queue as main

        // This block has been processed before, should we update here?
        if let Some(branch) = self.dag_branches.branches.get(&block_id) {
            if branch.selected_parent != selected_parent.id() {
                // inconsistency found
                todo!()
            }
            let mut branch_tips_iter = branch.status.dag_tips().unwrap().iter();
            let mut mine_tips_iter = mine_tips.iter();
            loop {
                match (branch_tips_iter.next(), mine_tips_iter.next()) {
                    (Some(b), Some(m)) if b == m => { /* nothing to do*/ }
                    (None, None) => {
                        // we're done iterating. everything is good.
                        break;
                    }
                    _ => {
                        // inconsistency found
                        todo!()
                    }
                }
            }
        }

        // find the longest-same-prefix branch
        let mut longest = (None, 0);
        for branch in &self.dag_branches.branches {
            let mut branch_tips_iter = branch.1.status.dag_tips().unwrap().iter();
            let mut mine_tips_iter = mine_tips.iter();
            let mut index = 1usize;

            let res = loop {
                match (branch_tips_iter.next(), mine_tips_iter.next()) {
                    (Some(b), Some(m)) if b == m => index += 1,
                    _ => break (branch.0, index - 1),
                }
            };
            if res.1 > longest.1 {
                longest = (Some(res.0), res.1)
            }
        }

        let mut chain = match longest {
            (None, _) => self.main.fork(selected_parent.id())?,
            (Some(v), index) => {
                let _parent_hash = self
                    .dag_branches
                    .branches
                    .get(v)
                    .and_then(|branch| branch.status.dag_tips())
                    .and_then(|tips| tips.get(index))
                    .expect("must exist")
                    .clone();
                // fork chain
                //self.main.fork(parent_hash);
                todo!()
            }
        };

        //self.broadcast_new_head();
        Ok(ConnectOk::DagConnected)
    }

    fn connect_inner(&mut self, block: Block) -> Result<ConnectOk> {
        let block_id = block.id();
        if block_id == *starcoin_storage::BARNARD_HARD_FORK_HASH
            && block.header().number() == starcoin_storage::BARNARD_HARD_FORK_HEIGHT
        {
            debug!("barnard hard fork {}", block_id);
            return Err(ConnectBlockError::BarnardHardFork(Box::new(block)).into());
        }

        if block.is_dag() {
            return self.connect_dag_inner(block);
        }

        if self.main.current_header().id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(ConnectOk::MainDuplicate);
        }

        if self.main.current_header().id() == block.header().parent_hash()
            && !self.block_exist(block_id)?
        {
            let executed_block = self.main.apply(block)?;
            let enacted_blocks = vec![executed_block.block().clone()];
            self.do_new_head(executed_block.clone(), 1, enacted_blocks, 0, vec![])?;
            return Ok(ConnectOk::ExeConnectMain(executed_block));
        }
        let (block_info, fork) = self.find_or_fork(block.header())?;
        match (block_info, fork) {
            //block has been processed in some branch, so just trigger a head selection.
            (Some(block_info), Some(branch)) => {
                debug!(
                    "Block {} has been processed, trigger head selection, total_difficulty: {}",
                    block_id,
                    branch.get_total_difficulty()?
                );
                self.select_head(branch)?;
                Ok(ConnectOk::Duplicate(ExecutedBlock { block, block_info }))
            }
            //block has been processed, and its parent is main chain, so just connect it to main chain.
            (Some(block_info), None) => {
                let executed_block = self.main.connect(ExecutedBlock {
                    block: block.clone(),
                    block_info,
                })?;
                info!(
                    "Block {} main has been processed, trigger head selection",
                    block_id
                );
                self.do_new_head(executed_block.clone(), 1, vec![block], 0, vec![])?;
                Ok(ConnectOk::Connect(executed_block))
            }
            (None, Some(mut branch)) => {
                let executed_block = branch.apply(block)?;
                self.select_head(branch)?;
                Ok(ConnectOk::ExeConnectBranch(executed_block))
            }
            (None, None) => Err(ConnectBlockError::FutureBlock(Box::new(block)).into()),
        }
    }

    #[cfg(test)]
    pub fn execute_dag_block_pool(&mut self) -> Result<ConnectOk> {
        let mut dag_blocks = self.dag_block_pool.lock().unwrap().clone();
        self.dag_block_pool.lock().unwrap().clear();
        return self.execute_dag_block_in_pool(
            dag_blocks,
            self.main
                .status()
                .tips_hash
                .expect("dag block must has current tips")
                .clone(),
        );
    }

    // pub fn execute_dag_block_in_pool(
    //     &mut self,
    //     mut dag_blocks: Vec<(Block, Vec<HashValue>)>,
    //     current_tips: Vec<HashValue>,
    // ) -> Result<ConnectOk> {
    //     // 3, process the blocks that are got from the pool
    //     // sort by id
    //     dag_blocks.sort_by_key(|(block, _)| block.header().id());

    //     let mut dag_block_next_parent = current_tips
    //         .iter()
    //         .max()
    //         .expect("tips must be larger than 0")
    //         .clone();
    //     let mut next_tips = Some(vec![]);
    //     let mut executed_blocks = vec![];
    //     // connect the block one by one
    //     dag_blocks
    //         .into_iter()
    //         .try_fold((), |_, (block, dag_block_parents)| {
    //             let next_transaction_parent = block.header().id();
    //             let result = self.connect_to_main(
    //                 block,
    //                 Some(dag_block_parents.clone()),
    //                 Some(dag_block_next_parent),
    //                 &mut next_tips,
    //             );
    //             match result {
    //                 std::result::Result::Ok(connect_ok) => {
    //                     executed_blocks.push((connect_ok.block().clone(), dag_block_parents));
    //                     dag_block_next_parent = next_transaction_parent;
    //                     Ok(())
    //                 }
    //                 Err(error) => {
    //                     bail!("apply_and_select_head failed, error: {}", error.to_string())
    //                 }
    //             }
    //         })?;

    //     match next_tips {
    //         Some(new_tips) => {
    //             if new_tips.is_empty() {
    //                 bail!("no new block has been executed successfully!");
    //             }

    //             let mut connected = self.main.is_head_of_dag_accumulator(new_tips.clone())?;
    //             if self.main.dag_parents_in_tips(new_tips.clone())? {
    //                 // 1, write to disc
    //                 if !connected {
    //                     self.main.append_dag_accumulator_leaf(new_tips.clone())?;
    //                     connected = true;
    //                 }
    //             }

    //             if connected {
    //                 // 2, broadcast the blocks sorted by their id
    //                 executed_blocks
    //                     .iter()
    //                     .for_each(|(exe_block, dag_block_parents)| {
    //                         if let Some(block) = exe_block {
    //                             self.broadcast_new_head(
    //                                 block.clone(),
    //                                 Some(dag_block_parents.clone()),
    //                                 Some(new_tips.clone()),
    //                             );
    //                         }
    //                     });
    //             }

    //             return executed_blocks
    //                 .last()
    //                 .map(|(exe_block, _)| {
    //                     if connected {
    //                         ConnectOk::ExeConnectMain(
    //                             exe_block
    //                                 .as_ref()
    //                                 .expect("exe block should not be None!")
    //                                 .clone(),
    //                         )
    //                     } else {
    //                         ConnectOk::ExeConnectBranch(
    //                             exe_block
    //                                 .as_ref()
    //                                 .expect("exe block should not be None!")
    //                                 .clone(),
    //                         )
    //                     }
    //                 })
    //                 .ok_or_else(|| format_err!("no block has been executed successfully!"));
    //         }
    //         None => {
    //             unreachable!("next tips should not be None");
    //         }
    //     };
    // }
}
