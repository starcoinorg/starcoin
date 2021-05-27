// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::metrics::MINER_METRICS;
use anyhow::{format_err, Result};
use consensus::Consensus;
use crypto::hash::HashValue;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_account_api::{AccountAsyncService, AccountInfo, DefaultAccountChangeEvent};
use starcoin_account_service::AccountService;
use starcoin_chain::BlockChain;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_config::ChainNetwork;
use starcoin_config::NodeConfig;
use starcoin_open_block::OpenedBlock;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::cmp::min;
use std::{collections::HashMap, sync::Arc};
use types::{
    block::{BlockHeader, BlockTemplate, ExecutedBlock},
    system_events::{NewBranch, NewHeadBlock},
};

#[cfg(test)]
mod test_create_block_template;

#[derive(Debug)]
pub struct GetHeadRequest;

impl ServiceRequest for GetHeadRequest {
    type Response = HashValue;
}

#[derive(Debug)]
pub struct CreateBlockTemplateRequest;

impl ServiceRequest for CreateBlockTemplateRequest {
    type Response = Result<BlockTemplate>;
}

pub struct CreateBlockTemplateService {
    inner: Inner<TxPoolService>,
}

impl CreateBlockTemplateService {}

impl ServiceFactory<Self> for CreateBlockTemplateService {
    fn create(
        ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) -> Result<CreateBlockTemplateService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .expect("Startup info should exist when service start.");
        //TODO support get service ref by AsyncAPI;
        let account_service = ctx.service_ref::<AccountService>()?;
        let miner_account = block_on(async { account_service.get_default_account().await })?
            .ok_or_else(|| {
                format_err!("Default account should exist when CreateBlockTemplateService start.")
            })?;
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let inner = Inner::new(
            config.net(),
            storage,
            startup_info.main,
            txpool,
            config.miner.block_gas_limit,
            miner_account,
        )?;
        Ok(Self { inner })
    }
}

impl ActorService for CreateBlockTemplateService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewBranch>();
        ctx.subscribe::<DefaultAccountChangeEvent>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewBranch>();
        ctx.unsubscribe::<DefaultAccountChangeEvent>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for CreateBlockTemplateService {
    fn handle_event(
        &mut self,
        msg: NewHeadBlock,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) {
        if let Err(e) = self.inner.update_chain(msg.0.as_ref().clone()) {
            error!("err : {:?}", e)
        }
    }
}

impl EventHandler<Self, NewBranch> for CreateBlockTemplateService {
    fn handle_event(
        &mut self,
        msg: NewBranch,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) {
        self.inner.insert_uncle(msg.0.block.header().clone());
    }
}

impl EventHandler<Self, DefaultAccountChangeEvent> for CreateBlockTemplateService {
    fn handle_event(
        &mut self,
        msg: DefaultAccountChangeEvent,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) {
        info!("Miner account change to {}", msg.new_account.address);
        self.inner.miner_account = msg.new_account;
    }
}

impl ServiceHandler<Self, CreateBlockTemplateRequest> for CreateBlockTemplateService {
    fn handle(
        &mut self,
        _msg: CreateBlockTemplateRequest,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) -> Result<BlockTemplate> {
        let template = self.inner.create_block_template();
        self.inner.uncles_prune();
        template
    }
}

impl ServiceHandler<Self, GetHeadRequest> for CreateBlockTemplateService {
    fn handle(
        &mut self,
        _msg: GetHeadRequest,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) -> HashValue {
        self.inner.chain.current_header().id()
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
    chain: BlockChain,
    tx_provider: P,
    parent_uncle: HashMap<HashValue, Vec<HashValue>>,
    uncles: HashMap<HashValue, BlockHeader>,
    local_block_gas_limit: Option<u64>,
    miner_account: AccountInfo,
}

impl<P> Inner<P>
where
    P: TemplateTxProvider,
{
    pub fn new(
        net: &ChainNetwork,
        storage: Arc<dyn Store>,
        block_id: HashValue,
        tx_provider: P,
        local_block_gas_limit: Option<u64>,
        miner_account: AccountInfo,
    ) -> Result<Self> {
        let chain = BlockChain::new(net.time_service(), block_id, storage.clone())?;

        Ok(Inner {
            storage,
            chain,
            tx_provider,
            parent_uncle: HashMap::new(),
            uncles: HashMap::new(),
            local_block_gas_limit,
            miner_account,
        })
    }

    pub fn insert_uncle(&mut self, uncle: BlockHeader) {
        self.parent_uncle
            .entry(uncle.parent_hash())
            .or_insert_with(Vec::new)
            .push(uncle.id());
        self.uncles.insert(uncle.id(), uncle);
        MINER_METRICS.maybe_uncle_count.inc();
    }

    pub fn update_chain(&mut self, block: ExecutedBlock) -> Result<()> {
        let current_header = self.chain.current_header();
        let current_id = current_header.id();
        if self.chain.can_connect(&block) {
            self.chain.connect(block)?;
        } else {
            self.chain = BlockChain::new(
                self.chain.time_service(),
                block.header().id(),
                self.storage.clone(),
            )?;
            //current block possible bean uncle.
            self.uncles.insert(current_id, current_header);
            MINER_METRICS.maybe_uncle_count.inc();
        }
        Ok(())
    }

    pub fn find_uncles(&self) -> Vec<BlockHeader> {
        let mut new_uncle = Vec::new();
        let epoch = self.chain.epoch();
        if epoch.end_block_number() != (self.chain.current_header().number() + 1) {
            for maybe_uncle in self.uncles.values() {
                if new_uncle.len() as u64 >= epoch.max_uncles_per_block() {
                    break;
                }
                if self.chain.can_be_uncle(maybe_uncle).unwrap_or_default() {
                    new_uncle.push(maybe_uncle.clone())
                }
            }
        }

        new_uncle
    }

    fn uncles_prune(&mut self) {
        if !self.uncles.is_empty() {
            if let Ok(epoch) = self.chain.epoch_info() {
                // epoch的end_number是开区间，当前块已经生成但还没有apply，所以应该在epoch（最终状态）
                // 的倒数第二块处理时清理uncles
                if epoch.end_block_number() == (self.chain.current_header().number() + 2) {
                    self.uncles.clear();
                }
            }
        }
    }

    pub fn create_block_template(&self) -> Result<BlockTemplate> {
        let on_chain_block_gas_limit = self.chain.epoch().block_gas_limit();
        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        //TODO use a GasConstant value to replace 200.
        // block_gas_limit / min_gas_per_txn
        let max_txns = (block_gas_limit / 200) * 2;

        let txns = self.tx_provider.get_txns(max_txns);

        let chain_state = self.chain.chain_state_reader();
        let author = *self.miner_account.address();
        let author_auth_key = if chain_state.exist_account(self.miner_account.address())? {
            None
        } else {
            Some(self.miner_account.public_key.authentication_key())
        };

        let previous_header = self.chain.current_header();
        let uncles = self.find_uncles();
        let mut now_millis = self.chain.time_service().now_millis();
        if now_millis <= previous_header.timestamp() {
            info!(
                "Adjust new block timestamp by parent timestamp, parent.timestamp: {}, now: {}, gap: {}",
                previous_header.timestamp(), now_millis, previous_header.timestamp() - now_millis,
            );
            now_millis = previous_header.timestamp() + 1;
        }
        info!(
            "[CreateBlockTemplate] previous_header: {:?}, block_gas_limit: {}, max_txns: {}, txn len: {}, uncles len: {}, timestamp: {}",
            previous_header,
            block_gas_limit,
            max_txns,
            txns.len(),
            uncles.len(),
            now_millis,
        );

        let epoch = self.chain.epoch();
        let strategy = epoch.strategy();
        let difficulty = strategy.calculate_next_difficulty(&self.chain)?;

        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            block_gas_limit,
            author,
            author_auth_key,
            now_millis,
            uncles,
            difficulty,
            strategy,
        )?;
        let excluded_txns = opened_block.push_txns(txns)?;
        let template = opened_block.finalize()?;
        for invalid_txn in excluded_txns.discarded_txns {
            let _ = self.tx_provider.remove_invalid_txn(invalid_txn.id());
        }
        Ok(template)
    }
}
