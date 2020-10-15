// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use chain::BlockChain;
use consensus::Consensus;
use crypto::hash::HashValue;
use futures::executor::block_on;
use logger::prelude::*;
use starcoin_account_api::{AccountAsyncService, AccountInfo};
use starcoin_account_service::AccountService;
use starcoin_config::NodeConfig;
use starcoin_open_block::OpenedBlock;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_vm_types::genesis_config::{ChainNetwork, ConsensusStrategy};
use std::cmp::min;
use std::{collections::HashMap, sync::Arc};
use traits::ChainReader;
use types::{
    block::{Block, BlockHeader, BlockTemplate},
    system_events::{NewBranch, NewHeadBlock},
};

#[cfg(test)]
mod test_create_block_template;

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

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
    inner: Inner,
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
            startup_info.master,
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
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewBranch>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for CreateBlockTemplateService {
    fn handle_event(
        &mut self,
        msg: NewHeadBlock,
        _ctx: &mut ServiceContext<CreateBlockTemplateService>,
    ) {
        if let Err(e) = self.inner.update_chain(msg.0.get_block().clone()) {
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
        msg.0.iter().for_each(|uncle| {
            self.inner.insert_uncle(uncle.clone());
        });
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

pub struct Inner {
    consensus: ConsensusStrategy,
    storage: Arc<dyn Store>,
    chain: BlockChain,
    txpool: TxPoolService,
    parent_uncle: HashMap<HashValue, Vec<HashValue>>,
    uncles: HashMap<HashValue, BlockHeader>,
    local_block_gas_limit: Option<u64>,
    miner_account: AccountInfo,
}

impl Inner {
    pub fn insert_uncle(&mut self, uncle: BlockHeader) {
        self.parent_uncle
            .entry(uncle.parent_hash())
            .or_insert_with(Vec::new)
            .push(uncle.id());
        self.uncles.insert(uncle.id(), uncle);
    }

    pub fn new(
        net: &ChainNetwork,
        storage: Arc<dyn Store>,
        block_id: HashValue,
        txpool: TxPoolService,
        local_block_gas_limit: Option<u64>,
        miner_account: AccountInfo,
    ) -> Result<Self> {
        let chain = BlockChain::new(net.consensus(), block_id, storage.clone())?;

        Ok(Inner {
            consensus: net.consensus(),
            storage,
            chain,
            txpool,
            parent_uncle: HashMap::new(),
            uncles: HashMap::new(),
            local_block_gas_limit,
            miner_account,
        })
    }

    pub fn update_chain(&mut self, block: Block) -> Result<()> {
        if block.header().parent_hash() != self.chain.current_header().id() {
            self.chain = BlockChain::new(self.consensus, block.id(), self.storage.clone())?;
        } else {
            self.chain.update_chain_head(block)?;
        }
        Ok(())
    }

    pub fn do_uncles(&self) -> Vec<BlockHeader> {
        let mut new_uncle = Vec::new();
        if let Ok(epoch) = self.chain.epoch_info() {
            if epoch.end_number() != (self.chain.current_header().number() + 1) {
                for maybe_uncle in self.uncles.values() {
                    if new_uncle.len() >= MAX_UNCLE_COUNT_PER_BLOCK {
                        break;
                    }
                    if self.chain.can_be_uncle(maybe_uncle) {
                        new_uncle.push(maybe_uncle.clone())
                    }
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
                if epoch.end_number() == (self.chain.current_header().number() + 2) {
                    self.uncles.clear();
                }
            }
        }
    }

    pub fn create_block_template(&self) -> Result<BlockTemplate> {
        let on_chain_block_gas_limit = self.chain.get_on_chain_block_gas_limit()?;
        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        //TODO use a GasConstant value to replace 600.
        // block_gas_limit / min_gas_per_txn
        let max_txns = block_gas_limit / 600;

        let txns = self.txpool.get_pending_txns(Some(max_txns), None);

        let chain_state = self.chain.chain_state_reader();
        let author = *self.miner_account.address();
        let author_public_key = if chain_state.exist_account(self.miner_account.address())? {
            None
        } else {
            Some(self.miner_account.public_key.clone())
        };

        let previous_header = self.chain.current_header();
        let uncles = self.do_uncles();

        debug!(
            "CreateBlockTemplate, previous_header: {:?}, block_gas_limit: {}, max_txns: {}, txn len: {}",
            previous_header,
            block_gas_limit,
            max_txns,
            txns.len()
        );

        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            block_gas_limit,
            author,
            author_public_key,
            self.consensus.now_millis(),
            uncles,
        )?;
        let excluded_txns = opened_block.push_txns(txns)?;
        let template = opened_block.finalize()?;
        for invalid_txn in excluded_txns.discarded_txns {
            let _ = self.txpool.remove_txn(invalid_txn.id(), true);
        }
        Ok(template)
    }
}
