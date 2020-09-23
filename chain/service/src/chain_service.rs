// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use starcoin_chain::BlockChain;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_api::{ChainReader, ReadableChainService};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};
use std::sync::Arc;

/// A Chain reader service to provider Reader API.
pub struct ChainReaderService {
    inner: ChainReaderServiceInner,
}

impl ChainReaderService {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
    ) -> Result<Self> {
        Ok(Self {
            inner: ChainReaderServiceInner::new(config, startup_info, storage)?,
        })
    }
}

impl ServiceFactory<Self> for ChainReaderService {
    fn create(ctx: &mut ServiceContext<ChainReaderService>) -> Result<ChainReaderService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("StartupInfo should exist at service init."))?;
        Self::new(config, startup_info, storage)
    }
}

impl ActorService for ChainReaderService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainReaderService {
    fn handle_event(&mut self, event: NewHeadBlock, _ctx: &mut ServiceContext<ChainReaderService>) {
        let new_head = event.0.get_block().header();
        let old_head = self.inner.get_master().current_header().id();
        if let Err(e) = if new_head.parent_hash() == old_head {
            self.inner.update_chain_head(event.0.get_block().clone())
        } else {
            self.inner.switch_master(new_head.id())
        } {
            warn!("err: {:?}", e);
        }
    }
}

impl ServiceHandler<Self, ChainRequest> for ChainReaderService {
    fn handle(
        &mut self,
        msg: ChainRequest,
        _ctx: &mut ServiceContext<ChainReaderService>,
    ) -> Result<ChainResponse> {
        match msg {
            ChainRequest::CurrentHeader() => Ok(ChainResponse::BlockHeader(Box::new(Some(
                self.inner.master_head_header(),
            )))),
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeader(Box::new(
                self.inner.get_header_by_hash(hash)?,
            ))),
            ChainRequest::HeadBlock() => Ok(ChainResponse::Block(Box::new(
                self.inner.master_head_block(),
            ))),
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::Block(Box::new(
                self.inner.master_block_by_number(number)?.ok_or_else(|| {
                    format_err!("Can not find block from master by number {:?}", number)
                })?,
            ))),
            ChainRequest::GetBlockHeaderByNumber(number) => {
                Ok(ChainResponse::BlockHeader(Box::new(Some(
                    self.inner
                        .master_block_header_by_number(number)?
                        .ok_or_else(|| {
                            format_err!(
                                "Can not find block header from master by number {:?}",
                                number
                            )
                        })?,
                ))))
            }
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::OptionBlock(
                if let Some(block) = self.inner.get_block_by_hash(hash)? {
                    Some(Box::new(block))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockByUncle(uncle_id) => Ok(ChainResponse::OptionBlock(
                if let Some(block) = self.inner.master_block_by_uncle(uncle_id)? {
                    Some(Box::new(block))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockStateByHash(hash) => Ok(ChainResponse::BlockState(
                if let Some(block_state) = self.inner.get_block_state_by_hash(hash)? {
                    Some(Box::new(block_state))
                } else {
                    None
                },
            )),
            ChainRequest::GetBlockInfoByHash(hash) => Ok(ChainResponse::OptionBlockInfo(Box::new(
                self.inner.get_block_info_by_hash(hash)?,
            ))),
            ChainRequest::GetStartupInfo() => {
                Ok(ChainResponse::StartupInfo(self.inner.master_startup_info()))
            }
            ChainRequest::GetHeadChainInfo() => Ok(ChainResponse::ChainInfo(ChainInfo::new(
                *self.inner.master_startup_info().get_master(),
            ))),
            ChainRequest::GetTransaction(hash) => Ok(ChainResponse::Transaction(Box::new(
                self.inner
                    .get_transaction(hash)?
                    .ok_or_else(|| format_err!("Can not find transaction by hash {:?}", hash))?,
            ))),
            ChainRequest::GetTransactionInfo(hash) => Ok(ChainResponse::TransactionInfo(
                self.inner.get_transaction_info(hash)?,
            )),
            ChainRequest::GetBlocksByNumber(number, count) => Ok(ChainResponse::VecBlock(
                self.inner.master_blocks_by_number(number, count)?,
            )),
            ChainRequest::GetBlockTransactionInfos(block_id) => Ok(
                ChainResponse::TransactionInfos(self.inner.get_block_txn_infos(block_id)?),
            ),
            ChainRequest::GetTransactionInfoByBlockAndIndex { block_id, txn_idx } => {
                Ok(ChainResponse::TransactionInfo(
                    self.inner
                        .get_txn_info_by_block_and_index(block_id, txn_idx)?,
                ))
            }
            ChainRequest::GetEventsByTxnInfoId { txn_info_id } => Ok(ChainResponse::Events(
                self.inner.get_events_by_txn_info_id(txn_info_id)?,
            )),
            ChainRequest::GetEpochInfo() => Ok(ChainResponse::EpochInfo(self.inner.epoch_info()?)),
            ChainRequest::GetEpochInfoByNumber(number) => Ok(ChainResponse::EpochInfo(
                self.inner.get_epoch_info_by_number(number)?,
            )),
            ChainRequest::GetGlobalTimeByNumber(number) => Ok(ChainResponse::GlobalTime(
                self.inner.get_global_time_by_number(number)?,
            )),
        }
    }
}

pub struct ChainReaderServiceInner {
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    master: BlockChain,
    storage: Arc<dyn Store>,
}

impl ChainReaderServiceInner {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
    ) -> Result<Self> {
        let master = BlockChain::new(
            config.net().consensus(),
            startup_info.master,
            storage.clone(),
        )?;
        Ok(Self {
            config,
            startup_info,
            master,
            storage,
        })
    }

    pub fn get_master(&self) -> &BlockChain {
        &self.master
    }

    pub fn update_chain_head(&mut self, block: Block) -> Result<()> {
        self.master.update_chain_head(block)
    }

    pub fn switch_master(&mut self, new_head_id: HashValue) -> Result<()> {
        let old_head_id = self.get_master().current_header().id();
        if old_head_id != new_head_id {
            let old_difficulty = self
                .storage
                .get_block_info(old_head_id)?
                .ok_or_else(|| {
                    format_err!("block info not exist by old block id {:?}.", old_head_id)
                })?
                .get_total_difficulty();
            let new_difficulty = self
                .storage
                .get_block_info(new_head_id)?
                .ok_or_else(|| {
                    format_err!("block info not exist by new block id {:?}.", new_head_id)
                })?
                .get_total_difficulty();
            assert!(new_difficulty > old_difficulty);
            self.master = BlockChain::new(
                self.config.net().consensus(),
                new_head_id,
                self.storage.clone(),
            )?;
        }
        Ok(())
    }
}

impl ReadableChainService for ChainReaderServiceInner {
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

    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>, Error> {
        self.get_master().get_transaction_info(txn_hash)
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
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        self.storage.get_contract_events(txn_info_id)
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

    fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>> {
        self.get_master().get_latest_block_by_uncle(uncle_id, 500)
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

    fn epoch_info(&self) -> Result<EpochInfo> {
        self.get_master().epoch_info()
    }

    fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo> {
        self.get_master().get_epoch_info_by_number(Some(number))
    }

    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain> {
        self.get_master().get_global_time_by_number(number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_chain_api::ChainAsyncService;
    use starcoin_config::NodeConfig;
    use starcoin_service_registry::{RegistryAsyncService, RegistryService};

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let (storage, startup_info, _) = test_helper::Genesis::init_storage_for_test(config.net())?;
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        registry.put_shared(storage).await?;
        let service_ref = registry.register::<ChainReaderService>().await?;
        let chain_info = service_ref.master_head().await?;
        assert_eq!(*chain_info.get_head(), startup_info.master);
        Ok(())
    }
}
