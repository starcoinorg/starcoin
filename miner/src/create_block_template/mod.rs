// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::create_block_template::metrics::BlockBuilderMetrics;
use anyhow::{format_err, Result};
use futures::executor::block_on;
use new_header_service::NewHeaderChannel;
use starcoin_account_api::{AccountAsyncService, AccountInfo, DefaultAccountChangeEvent};
use starcoin_account_service::AccountService;
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_consensus::Consensus;
use starcoin_dag::blockdag::{BlockDAG, MineNewDagBlockInfo};
use std::sync::RwLock;

use starcoin_config::NodeConfig;
use starcoin_crypto::hash::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_sync::block_connector::MinerResponse;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::{Block, BlockHeader, BlockTemplate, Version};
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::cmp::min;
use std::sync::Arc;
mod metrics;
pub mod new_header_service;
//#[cfg(test)]
//mod test_create_block_template;

#[derive(Debug)]
pub struct BlockTemplateRequest;

impl ServiceRequest for BlockTemplateRequest {
    type Response = Result<BlockTemplateResponse>;
}

#[derive(Debug, Clone)]
pub struct BlockTemplateResponse {
    pub parent: BlockHeader,
    pub template: BlockTemplate,
}

pub struct BlockBuilderService {
    inner: Inner<TxPoolService>,
    new_header_channel: NewHeaderChannel,
}

impl BlockBuilderService {
    fn receive_header(&mut self) {
        info!("receive header in block builder service");
        match self.new_header_channel.new_header_receiver.try_recv() {
            Ok(new_header) => {
                match self
                    .inner
                    .set_current_block_header(new_header.as_ref().clone())
                {
                    Ok(()) => (),
                    Err(e) => error!(
                        "Failed to set current block header: {:?} in BlockBuilderService",
                        e
                    ),
                }
            }
            Err(e) => match e {
                crossbeam::channel::TryRecvError::Empty => (),
                crossbeam::channel::TryRecvError::Disconnected => {
                    error!("the new headerchannel is disconnected")
                }
            },
        }
    }
}

impl ServiceFactory<Self> for BlockBuilderService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let header_id = storage
            .get_startup_info()?
            .ok_or_else(|| {
                format_err!("failed to get the starup info when creating block builder service.")
            })?
            .main;
        let current_block_header =
            storage
                .get_block_header_by_hash(header_id)?
                .ok_or_else(|| {
                    format_err!(
                        "failed to get the block header: {:?} when creating block builder service.",
                        header_id
                    )
                })?;
        //TODO support get service ref by AsyncAPI;
        let account_service = ctx.service_ref::<AccountService>()?;
        let miner_account = block_on(async { account_service.get_default_account().await })?
            .ok_or_else(|| {
                format_err!("Default account should exist when BlockBuilderService start.")
            })?;
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| BlockBuilderMetrics::register(registry).ok());

        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;

        let inner = Inner::new(
            current_block_header,
            storage,
            txpool,
            config.miner.block_gas_limit,
            miner_account,
            dag,
            config,
            metrics,
            vm_metrics,
        )?;
        let new_header_channel = ctx.get_shared::<NewHeaderChannel>()?;
        Ok(Self {
            inner,
            new_header_channel,
        })
    }
}

impl ActorService for BlockBuilderService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<DefaultAccountChangeEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<DefaultAccountChangeEvent>();
        Ok(())
    }
}

impl EventHandler<Self, DefaultAccountChangeEvent> for BlockBuilderService {
    fn handle_event(&mut self, msg: DefaultAccountChangeEvent, _ctx: &mut ServiceContext<Self>) {
        info!("Miner account change to {}", msg.new_account.address);

        if let Ok(mut account) = self.inner.miner_account.write() {
            *account = msg.new_account;
        } else {
            warn!("Failed to acquire write lock for miner_account");
        }
    }
}

impl ServiceHandler<Self, BlockTemplateRequest> for BlockBuilderService {
    fn handle(
        &mut self,
        _msg: BlockTemplateRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> <BlockTemplateRequest as ServiceRequest>::Response {
        let header_version = self
            .inner
            .config
            .net()
            .genesis_config()
            .block_header_version;
        self.receive_header();
        self.inner.create_block_template(header_version)
    }
}

pub trait TemplateTxProvider {
    fn get_txns(&self, max: u64) -> Vec<SignedUserTransaction>;
    fn remove_invalid_txn(&self, txn_hash: HashValue);
}

pub struct EmptyProvider;

impl TemplateTxProvider for EmptyProvider {
    fn get_txns(&self, _max: u64) -> Vec<SignedUserTransaction> {
        vec![]
    }

    fn remove_invalid_txn(&self, _txn_hash: HashValue) {}
}

impl TemplateTxProvider for TxPoolService {
    fn get_txns(&self, max: u64) -> Vec<SignedUserTransaction> {
        self.get_pending_txns(Some(max), None)
    }

    fn remove_invalid_txn(&self, txn_hash: HashValue) {
        self.remove_txn(txn_hash, true);
    }
}

pub struct Inner<P> {
    storage: Arc<dyn Store>,
    tx_provider: P,
    local_block_gas_limit: Option<u64>,
    miner_account: RwLock<AccountInfo>,
    main: BlockChain,
    config: Arc<NodeConfig>,
    #[allow(unused)]
    metrics: Option<BlockBuilderMetrics>,
    vm_metrics: Option<VMMetrics>,
}

impl<P> Inner<P>
where
    P: TemplateTxProvider + TxPoolSyncService,
{
    pub fn new(
        header: BlockHeader,
        storage: Arc<dyn Store>,
        tx_provider: P,
        local_block_gas_limit: Option<u64>,
        miner_account: AccountInfo,
        dag: BlockDAG,
        config: Arc<NodeConfig>,
        metrics: Option<BlockBuilderMetrics>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        Ok(Self {
            storage: storage.clone(),
            tx_provider,
            local_block_gas_limit,
            miner_account: RwLock::new(miner_account),
            main: BlockChain::new(
                config.net().time_service(),
                header.id(),
                storage,
                vm_metrics.clone(),
                dag,
            )?,
            config,
            metrics,
            vm_metrics,
        })
    }

    fn resolve_block_parents(&mut self) -> Result<MinerResponse> {
        let MineNewDagBlockInfo {
            tips,
            ghostdata,
            pruning_point,
        } = {
            info!("block template main is {:?}", self.main.current_header());
            let pruning_point = if self.main.current_header().pruning_point() == HashValue::zero() {
                self.main.get_genesis_hash()
            } else {
                self.main.current_header().pruning_point()
            };

            let MineNewDagBlockInfo {
                tips,
                ghostdata,
                pruning_point,
            } = self.main.dag().calc_mergeset_and_tips(
                pruning_point,
                self.config.miner.maximum_parents_count(),
                self.main.get_genesis_hash(),
            )?;
            info!("after calculate the ghostdata, tips are: {:?}, ghostdata is: {:?}, pruning point is: {:?}", tips, ghostdata, pruning_point);

            self.update_main_chain(ghostdata.selected_parent)?;

            let merge_bound_hash = self.main.get_merge_bound_hash(ghostdata.selected_parent)?;

            let (tips, ghostdata) = self.main.dag().remove_bounded_merge_breaking_parents(
                tips,
                ghostdata,
                pruning_point,
                merge_bound_hash,
            )?;
            info!("after remove the bounded merge breaking parents, tips are: {:?}, ghostdata is: {:?}, pruning point is: {:?}, merge bound hash is: {:?}", tips, ghostdata, pruning_point, merge_bound_hash);

            self.update_main_chain(ghostdata.selected_parent)?;

            MineNewDagBlockInfo {
                tips,
                ghostdata,
                pruning_point,
            }
        };

        let selected_parent = ghostdata.selected_parent;

        let time_service = self.config.net().time_service();
        let storage = self.storage.clone();
        let vm_metrics = self.vm_metrics.clone();
        let main = BlockChain::new(
            time_service,
            selected_parent,
            storage,
            vm_metrics,
            self.main.dag(),
        )?;

        let epoch = main.epoch().clone();
        let strategy = epoch.strategy();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let previous_header = main
            .get_storage()
            .get_block_header_by_hash(selected_parent)?
            .ok_or_else(|| format_err!("BlockHeader should exist by hash: {}", selected_parent))?;
        let next_difficulty = epoch.strategy().calculate_next_difficulty(&main)?;
        let now_milliseconds = main.time_service().now_millis();

        Ok(MinerResponse {
            previous_header,
            on_chain_block_gas_limit,
            tips_hash: tips,
            blue_blocks_hash: ghostdata.mergeset_blues.as_ref()[1..].to_vec(),
            strategy,
            next_difficulty,
            now_milliseconds,
            pruning_point,
        })
    }

    pub fn create_block_template(&mut self, version: Version) -> Result<BlockTemplateResponse> {
        let MinerResponse {
            previous_header,
            tips_hash,
            blue_blocks_hash: blues,
            strategy,
            on_chain_block_gas_limit,
            next_difficulty: difficulty,
            now_milliseconds: mut now_millis,
            pruning_point,
        } = self.resolve_block_parents()?;

        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        //TODO use a GasConstant value to replace 200.
        // block_gas_limit / min_gas_per_txn
        let max_txns = (block_gas_limit / 200) * 2;

        let txns = self.tx_provider.get_txns(max_txns);
        let author = *self.miner_account.read().unwrap().address();

        if now_millis <= previous_header.timestamp() {
            info!(
                "Adjust new block timestamp by parent timestamp, parent.timestamp: {}, now: {}, gap: {}",
                previous_header.timestamp(), now_millis, previous_header.timestamp() - now_millis,
            );
            now_millis = previous_header.timestamp() + 1;
        }

        let blue_blocks = blues
            .into_iter()
            .map(|hash| self.storage.get_block_by_hash(hash))
            .collect::<Result<Vec<Option<Block>>>>()?
            .into_iter()
            .map(|op_block_header| {
                op_block_header.ok_or_else(|| format_err!("uncle block header not found."))
            })
            .collect::<Result<Vec<Block>>>()?;

        let uncles = blue_blocks
            .iter()
            .map(|block| block.header().clone())
            .collect::<Vec<_>>();

        info!(
            "[CreateBlockTemplate] previous_header: {:?}, block_gas_limit: {}, max_txns: {}, txn len: {}, uncles len: {}, timestamp: {}",
            previous_header,
            block_gas_limit,
            max_txns,
            txns.len(),
            uncles.len(),
            now_millis,
        );

        let header_version =
            if BlockHeader::check_upgrade(previous_header.number() + 1, previous_header.chain_id())
            {
                version
            } else {
                0
            };

        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header.clone(),
            block_gas_limit,
            author,
            now_millis,
            uncles,
            difficulty,
            strategy,
            self.vm_metrics.clone(),
            tips_hash,
            blue_blocks,
            header_version,
            pruning_point,
        )?;

        let excluded_txns = opened_block.push_txns(txns)?;

        let template = opened_block.finalize()?;
        for invalid_txn in excluded_txns.discarded_txns {
            self.tx_provider.remove_invalid_txn(invalid_txn.id());
        }

        Ok(BlockTemplateResponse {
            parent: previous_header,
            template,
        })
    }

    pub fn set_current_block_header(&mut self, header: BlockHeader) -> Result<()> {
        if self.main.current_header().id() == header.id() {
            return Ok(());
        }
        self.main = BlockChain::new(
            self.config.net().time_service(),
            header.id(),
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.main.dag(),
        )?;
        Ok(())
    }

    fn update_main_chain(&mut self, selected_parent: HashValue) -> Result<()> {
        if self.main.head_block().header().id() != selected_parent {
            self.main = BlockChain::new(
                self.config.net().time_service(),
                selected_parent,
                self.storage.clone(),
                self.vm_metrics.clone(),
                self.main.dag(),
            )?;
        }
        Ok(())
    }
}
