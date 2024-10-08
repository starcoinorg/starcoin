// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::create_block_template::metrics::BlockBuilderMetrics;
use anyhow::{format_err, Result};
use futures::executor::block_on;
use starcoin_account_api::{AccountAsyncService, AccountInfo, DefaultAccountChangeEvent};
use starcoin_account_service::AccountService;

use starcoin_config::NodeConfig;
use starcoin_crypto::hash::HashValue;

use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};
use starcoin_storage::{Storage, Store};
use starcoin_sync::block_connector::{BlockConnectorService, MinerRequest, MinerResponse};
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::{Block, BlockHeader, BlockTemplate, Version};
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::cmp::min;
use std::sync::Arc;

mod metrics;
//#[cfg(test)]
//mod test_create_block_template;

#[derive(Debug)]
pub struct BlockTemplateRequest;

impl ServiceRequest for BlockTemplateRequest {
    type Response = Result<BlockTemplateResponse>;
}

#[derive(Debug)]
pub struct BlockTemplateResponse {
    pub parent: BlockHeader,
    pub template: BlockTemplate,
}

pub struct BlockBuilderService {
    inner: Inner<TxPoolService, TxPoolService>,
}

impl BlockBuilderService {}

impl ServiceFactory<Self> for BlockBuilderService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let connector_service = ctx
            .service_ref::<BlockConnectorService<TxPoolService>>()?
            .clone();
        //TODO support get service ref by AsyncAPI;
        let account_service = ctx.service_ref::<AccountService>()?;
        let miner_account = block_on(async { account_service.get_default_account().await })?
            .ok_or_else(|| {
                format_err!("Default account should exist when BlockBuilderService start.")
            })?;
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let metrics = config
            .metrics
            .registry()
            .and_then(|registry| BlockBuilderMetrics::register(registry).ok());

        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;

        let inner = Inner::new(
            connector_service,
            storage,
            txpool,
            config.miner.block_gas_limit,
            miner_account,
            metrics,
            vm_metrics,
        )?;
        Ok(Self { inner })
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
        self.inner.miner_account = msg.new_account;
    }
}

impl ServiceHandler<Self, BlockTemplateRequest> for BlockBuilderService {
    fn handle(
        &mut self,
        _msg: BlockTemplateRequest,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<BlockTemplateResponse> {
        let header_version = ctx
            .get_shared::<Arc<NodeConfig>>()?
            .net()
            .genesis_config()
            .block_header_version;

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

pub struct Inner<P, T: TxPoolSyncService + 'static> {
    storage: Arc<dyn Store>,
    block_connector_service: ServiceRef<BlockConnectorService<T>>,
    tx_provider: P,
    local_block_gas_limit: Option<u64>,
    miner_account: AccountInfo,
    #[allow(unused)]
    metrics: Option<BlockBuilderMetrics>,
    vm_metrics: Option<VMMetrics>,
}

impl<P, T> Inner<P, T>
where
    P: TemplateTxProvider,
    T: TxPoolSyncService,
{
    pub fn new(
        block_connector_service: ServiceRef<BlockConnectorService<T>>,
        storage: Arc<dyn Store>,
        tx_provider: P,
        local_block_gas_limit: Option<u64>,
        miner_account: AccountInfo,
        metrics: Option<BlockBuilderMetrics>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            block_connector_service,
            tx_provider,
            local_block_gas_limit,
            miner_account,
            metrics,
            vm_metrics,
        })
    }

    pub fn create_block_template(&self, version: Version) -> Result<BlockTemplateResponse> {
        let MinerResponse {
            previous_header,
            tips_hash,
            blues_hash: blues,
            strategy,
            on_chain_block_gas_limit,
            next_difficulty: difficulty,
            now_milliseconds: mut now_millis,
            pruning_point,
        } = *block_on(self.block_connector_service.send(MinerRequest { version }))??;

        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        //TODO use a GasConstant value to replace 200.
        // block_gas_limit / min_gas_per_txn
        let max_txns = (block_gas_limit / 200) * 2;

        let txns = self.tx_provider.get_txns(max_txns);
        let author = *self.miner_account.address();

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
}
