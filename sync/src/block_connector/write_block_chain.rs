// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_connector::metrics::ChainMetrics;
use anyhow::{format_err, Ok, Result};
use itertools::Itertools;
use starcoin_chain::chain_common_func::has_dag_block;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, WriteableChainService};
use starcoin_config::NodeConfig;
#[cfg(test)]
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_dag::consensusdb::consensus_state::DagState;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{ServiceContext, ServiceRef};
use starcoin_storage::Store;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockHeader, ExecutedBlock},
    startup_info::StartupInfo,
    system_events::{NewBranch, NewHeadBlock},
};
#[cfg(test)]
use starcoin_vm_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
// use std::collections::HashSet;
use std::{fmt::Formatter, sync::Arc};

use super::BlockConnectorService;

const MAX_ROLL_BACK_BLOCK: usize = 10;

pub struct WriteBlockChainService<P>
where
    P: TxPoolSyncService,
{
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    main_header: BlockHeader,
    storage: Arc<dyn Store>,
    txpool: P,
    bus: ServiceRef<BusService>,
    metrics: Option<ChainMetrics>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
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
            Self::Duplicate => "Duplicate",
            Self::ExeConnectMain => "ExeConnectMain",
            Self::ExeConnectBranch => "ExeConnectBranch",
            Self::Connect => "Connect",
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

        let result = self.connect_dag_block_from_peer(block);

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
        dag: BlockDAG,
    ) -> Result<Self> {
        let main_header = storage
            .get_block_header_by_hash(startup_info.main)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block header by hash in new WriteBlockChainService {:?}",
                    startup_info.main
                )
            })?;

        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| ChainMetrics::register(registry).ok());

        Ok(Self {
            config,
            startup_info,
            main_header,
            storage,
            txpool,
            bus,
            metrics,
            vm_metrics,
            dag,
        })
    }

    #[cfg(test)]
    pub fn new_with_dag_fork_number(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TransactionPoolServiceT,
        bus: ServiceRef<BusService>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> Result<Self> {
        let this: Self = Self::new(
            config.clone(),
            startup_info,
            storage,
            txpool,
            bus,
            vm_metrics,
            dag,
        )?;
        Ok(this)
    }

    pub fn switch_header(
        &mut self,
        executed_block: &ExecutedBlock,
        ctx: &mut ServiceContext<BlockConnectorService<TransactionPoolServiceT>>,
    ) -> Result<BlockHeader> {
        info!("jacktest: switch header before");
        let current_block_info = self
            .storage
            .get_block_info(self.main_header.id())?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block info by hash in switch_header {:?}.",
                    self.main_header.id()
                )
            })?;
        if executed_block.total_difficulty() > current_block_info.total_difficulty {
            self.main_header = executed_block.block.header().clone();
            self.update_startup_info(executed_block.block().header())?;
            self.config
                .net()
                .time_service()
                .adjust(executed_block.header().timestamp());
            ctx.broadcast(NewHeadBlock {
                executed_block: Arc::new(executed_block.clone()),
            });
        }
        Ok(self.main_header.clone())
    }

    // todo: remove this function
    // this function is only used in testing case, fix them and remove this function
    pub fn get_main(&self) -> BlockChain {
        let time_service = self.config.net().time_service();
        let chain = BlockChain::new(
            time_service,
            self.main_header.id(),
            self.storage.clone(),
            None,
            self.dag.clone(),
        );
        chain.expect(
            "Failed to create writeable block chain, this function should be remove in the future",
        )
    }

    pub fn get_main_header(&self) -> &BlockHeader {
        &self.main_header
    }

    pub fn get_bus(&self) -> ServiceRef<BusService> {
        self.bus.clone()
    }

    //todo: return a reference
    pub fn get_dag(&self) -> BlockDAG {
        self.dag.clone()
    }

    #[cfg(test)]
    pub fn create_block(
        &self,
        author: AccountAddress,
        parent_hash: Option<HashValue>,
        user_txns: Vec<SignedUserTransaction>,
        uncles: Vec<BlockHeader>,
        block_gas_limit: Option<u64>,
        tips: Vec<HashValue>,
    ) -> Result<Block> {
        let main = self.get_main();
        let (block_template, _transactions) = main.create_block_template(
            author,
            parent_hash,
            user_txns,
            uncles,
            block_gas_limit,
            tips,
            HashValue::zero(),
        )?;
        let time_service = self.config.net().time_service();
        Ok(main
            .consensus()
            .create_block(block_template, time_service.as_ref())
            .unwrap())
    }

    #[cfg(test)]
    pub fn time_sleep(&self, millis: u64) {
        self.config.net().time_service().sleep(millis);
    }

    #[cfg(test)]
    pub fn apply_failed(&mut self, block: Block) -> Result<()> {
        use anyhow::bail;
        use starcoin_chain::verifier::FullVerifier;
        let mut main = self.get_main();
        let verified_block = main.verify_with_verifier::<FullVerifier>(block)?;
        let _executed_block = main.execute(verified_block)?;
        bail!("In test case, return a failure intentionally to force sync to reconnect the block");
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
        let new_block = self
            .storage
            .get_block_by_hash(new_head_block)?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block by hash in switch_new_main {:?}.",
                    new_head_block
                )
            })?;
        let new_head_block_info =
            self.storage
                .get_block_info(new_head_block)?
                .ok_or_else(|| {
                    format_err!(
                        "Cannot find block info by hash in switch_new_main {:?}.",
                        new_head_block
                    )
                })?;

        let main_block_info = self
            .storage
            .get_block_info(self.main_header.id())?
            .ok_or_else(|| {
                format_err!(
                    "Cannot find block info by hash in switch_new_main {:?}.",
                    self.main_header.id()
                )
            })?;

        let main_total_difficulty = main_block_info.total_difficulty;
        let branch_total_difficulty = new_head_block_info.total_difficulty;
        if branch_total_difficulty > main_total_difficulty {
            self.main_header = new_block.header().clone();
            self.update_startup_info(new_block.header())?;
            ctx.broadcast(NewHeadBlock {
                executed_block: Arc::new(ExecutedBlock::new(new_block, new_head_block_info)),
            });
            Ok(())
        } else {
            Ok(())
        }
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
        info!("jacktest: do new head begin");
        self.update_startup_info(executed_block.block().header())?;
        if retracted_count > 0 {
            if let Some(metrics) = self.metrics.as_ref() {
                metrics.chain_rollback_block_total.inc_by(retracted_count);
            }
        }
        info!("jacktest: do new head 2");
        self.commit_2_txpool(enacted_blocks, retracted_blocks);
        info!("jacktest: do new head 3");
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

        self.broadcast_new_head(executed_block);

        Ok(())
    }

    /// Reset the node to `block_id`, and replay blocks after the block
    pub fn reset(&mut self, block_id: HashValue) -> Result<()> {
        let new_head_block = self
            .storage
            .get_block_by_hash(block_id)?
            .ok_or_else(|| format_err!("Can not find block {} in main chain", block_id,))?;
        let new_branch = BlockChain::new(
            self.config.net().time_service(),
            block_id,
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )?;

        if new_head_block.header().pruning_point() == HashValue::zero() {
            let genesis = self
                .storage
                .get_genesis()?
                .ok_or_else(|| format_err!("Cannot get the genesis in storage!"))?;
            self.dag.save_dag_state_directly(
                genesis,
                DagState {
                    tips: vec![new_head_block.header().id()],
                },
            )?;
            self.dag.save_dag_state_directly(
                HashValue::zero(),
                DagState {
                    tips: vec![new_head_block.header().id()],
                },
            )?;
        } else {
            self.dag.save_dag_state_directly(
                new_head_block.header().pruning_point(),
                DagState {
                    tips: vec![new_head_block.header().id()],
                },
            )?;
        }

        let executed_block = new_branch.head_block();

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
        let mut chain = BlockChain::new(
            self.config.net().time_service(),
            block.header().parent_hash(),
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
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

    fn find_red_blocks(
        &self,
        selected_header: HashValue,
        deselected_header: HashValue,
    ) -> Result<Vec<Block>> {
        info!(
            "find_red_blocks selected_header:{}, deselected_header:{}",
            selected_header, deselected_header
        );
        let ghostdata = self.dag.ghostdata(&[selected_header, deselected_header])?;
        info!(
            "find_red_blocks red block count: {:?}",
            ghostdata.mergeset_reds.len()
        );
        let red_blocks = ghostdata
        .mergeset_reds
        .iter()
        .map(|id| Ok(self.storage.get_block_by_hash(*id)?.ok_or_else(|| format_err!("cannot find the block by id {:?} in find_red_blocks for selecting new head", id))?))
        .collect::<Result<Vec<Block>>>()?
        .into_iter().sorted_by(|a, b| {
            match a.header().number().cmp(&b.header().number()) {
                std::cmp::Ordering::Equal => a.header().id().cmp(&b.header().id()),
                other => other,
            }
        }).collect();
        Ok(red_blocks)
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

    pub fn broadcast_new_head(&self, block: ExecutedBlock) {
        if let Some(metrics) = self.metrics.as_ref() {
            metrics
                .chain_select_head_total
                .with_label_values(&["new_head"])
                .inc()
        }

        if let Err(e) = self.bus.broadcast(NewHeadBlock {
            executed_block: Arc::new(block),
        }) {
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

    fn connect_dag_block_from_peer(&mut self, block: Block) -> Result<ConnectOk> {
        let block_id = block.id();
        if self.main_header.id() == block_id {
            debug!("Repeat connect, current header is {} already.", block_id);
            return Ok(ConnectOk::Duplicate);
        }

        if has_dag_block(block_id, self.storage.clone(), &self.dag).unwrap_or(false) {
            return Ok(ConnectOk::Duplicate);
        }

        if !block.header().parents_hash().iter().all(|parent_hash| {
            has_dag_block(*parent_hash, self.storage.clone(), &self.dag).unwrap_or(false)
        }) {
            debug!(
                "block: {:?} is a future dag block, trigger sync to pull other dag blocks",
                block_id
            );
            return Err(ConnectBlockError::FutureBlock(Box::new(block)).into());
        }

        let mut main = BlockChain::new(
            self.config.net().time_service(),
            block.header().parent_hash(),
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )?;

        let executed_block = main.apply(block)?;
        let current_block_info = self.storage.get_block_info(block_id)?.ok_or_else(|| {
            format_err!(
                "Cannot find block info by id in connect_dag_block_from_peer {:?}.",
                block_id
            )
        })?;
        if executed_block.block_info().total_difficulty > current_block_info.total_difficulty {
            self.main_header = executed_block.block().header().clone();
            self.update_startup_info(executed_block.block().header())?;
            Ok(ConnectOk::ExeConnectBranch)
        } else {
            Ok(ConnectOk::ExeConnectMain)
        }
    }
}
