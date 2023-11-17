// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Error, Ok, Result};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_api::{
    ChainReader, ChainWriter, ReadableChainService, TransactionInfoWithProof,
};
use starcoin_config::NodeConfig;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_flexidag::flexidag_service::{
    GetDagAccumulatorLeafDetail, GetDagBlockParents, UpdateDagTips,
};
use starcoin_flexidag::{flexidag_service, FlexidagService};
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::dag_protocol::{
    GetDagAccumulatorLeaves, GetTargetDagAccumulatorLeafDetail, TargetDagAccumulatorLeaf,
    TargetDagAccumulatorLeafDetail,
};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_types::block::ExecutedBlock;
use starcoin_types::contract_event::ContractEventInfo;
use starcoin_types::dag_block::KTotalDifficulty;
use starcoin_types::filter::Filter;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    contract_event::ContractEvent,
    startup_info::StartupInfo,
    transaction::Transaction,
};
use starcoin_vm_runtime::metrics::VMMetrics;
use starcoin_vm_types::access_path::AccessPath;
use std::sync::{Arc, Mutex};

/// A Chain reader service to provider Reader API.
pub struct ChainReaderService {
    inner: ChainReaderServiceInner,
}

impl ChainReaderService {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        flexidag_service: ServiceRef<FlexidagService>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        Ok(Self {
            inner: ChainReaderServiceInner::new(
                config.clone(),
                startup_info,
                storage.clone(),
                flexidag_service,
                vm_metrics.clone(),
            )?,
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
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
        let flexidag_service = ctx.service_ref::<FlexidagService>()?.clone();
        Self::new(config, startup_info, storage, flexidag_service, vm_metrics)
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
    fn handle_event(&mut self, event: NewHeadBlock, ctx: &mut ServiceContext<ChainReaderService>) {
        let new_head = event.0.block().header().clone();
        if let Err(e) = if self.inner.get_main().can_connect(event.0.as_ref()) {
            match self.inner.update_chain_head(event.0.as_ref().clone()) {
                std::result::Result::Ok(_) => (),
                Err(e) => Err(e),
            }
        } else {
            self.inner.switch_main(new_head.id())
        } {
            warn!("ChainReaderService handle NewHeadBlock err: {:?}", e);
        }
    }
}

impl ServiceHandler<Self, ChainRequest> for ChainReaderService {
    fn handle(
        &mut self,
        msg: ChainRequest,
        ctx: &mut ServiceContext<ChainReaderService>,
    ) -> Result<ChainResponse> {
        match msg {
            ChainRequest::CurrentHeader() => Ok(ChainResponse::BlockHeader(Box::new(
                self.inner.main_head_header(),
            ))),
            ChainRequest::GetHeaderByHash(hash) => Ok(ChainResponse::BlockHeaderOption(Box::new(
                self.inner.get_header_by_hash(hash)?,
            ))),
            ChainRequest::HeadBlock() => {
                Ok(ChainResponse::Block(Box::new(self.inner.main_head_block())))
            }
            ChainRequest::GetBlockByNumber(number) => Ok(ChainResponse::BlockOption(
                self.inner.main_block_by_number(number)?.map(Box::new),
            )),
            ChainRequest::GetBlockHeaderByNumber(number) => {
                Ok(ChainResponse::BlockHeaderOption(Box::new(Some(
                    self.inner
                        .main_block_header_by_number(number)?
                        .ok_or_else(|| {
                            format_err!(
                                "Can not find block header from main by number {:?}",
                                number
                            )
                        })?,
                ))))
            }
            ChainRequest::GetBlockByHash(hash) => Ok(ChainResponse::BlockOption(
                self.inner.get_block_by_hash(hash)?.map(Box::new),
            )),
            ChainRequest::GetBlockInfoByHash(hash) => Ok(ChainResponse::BlockInfoOption(Box::new(
                self.inner.get_block_info_by_hash(hash)?,
            ))),
            ChainRequest::GetBlockInfoByNumber(number) => Ok(ChainResponse::BlockInfoOption(
                Box::new(self.inner.main_block_info_by_number(number)?),
            )),
            ChainRequest::GetStartupInfo() => Ok(ChainResponse::StartupInfo(Box::new(
                self.inner.main_startup_info(),
            ))),
            ChainRequest::GetHeadChainStatus() => Ok(ChainResponse::ChainStatus(Box::new(
                self.inner.main.status(),
            ))),
            ChainRequest::GetTransaction(hash) => Ok(ChainResponse::TransactionOption(
                self.inner.get_transaction(hash)?.map(Box::new),
            )),
            ChainRequest::GetTransactionBlock(txn_id) => {
                let block_id = self
                    .inner
                    .get_transaction_info(txn_id)?
                    .map(|info| info.block_id());
                let block = match block_id {
                    Some(id) => self.inner.get_block_by_hash(id)?,
                    None => None,
                };
                Ok(ChainResponse::BlockOption(block.map(Box::new)))
            }
            ChainRequest::GetTransactionInfo(hash) => Ok(ChainResponse::TransactionInfo(
                self.inner.get_transaction_info(hash)?,
            )),
            ChainRequest::GetBlocksByNumber(number, reverse, count) => Ok(ChainResponse::BlockVec(
                self.inner.main_blocks_by_number(number, reverse, count)?,
            )),
            ChainRequest::GetBlockTransactionInfos(block_id) => Ok(
                ChainResponse::TransactionInfos(self.inner.get_block_txn_infos(block_id)?),
            ),
            ChainRequest::GetTransactionInfoByBlockAndIndex {
                block_hash: block_id,
                txn_idx,
            } => Ok(ChainResponse::TransactionInfo(
                self.inner
                    .get_txn_info_by_block_and_index(block_id, txn_idx)?,
            )),
            ChainRequest::GetEventsByTxnHash { txn_hash } => {
                let txn_info = self
                    .inner
                    .get_transaction_info(txn_hash)?
                    .ok_or_else(|| anyhow::anyhow!("cannot find txn info of txn {}", txn_hash))?;

                let events = self
                    .inner
                    .get_events_by_txn_info_hash(txn_info.id())?
                    .unwrap_or_default();

                let event_infos = if events.is_empty() {
                    vec![]
                } else {
                    events
                        .into_iter()
                        .enumerate()
                        .map(|(idx, evt)| ContractEventInfo {
                            block_hash: txn_info.block_id,
                            block_number: txn_info.block_number,
                            transaction_hash: txn_hash,
                            transaction_index: txn_info.transaction_index,
                            transaction_global_index: txn_info.transaction_global_index,
                            event_index: idx as u32,
                            event: evt,
                        })
                        .collect()
                };
                Ok(ChainResponse::Events(event_infos))
            }
            ChainRequest::MainEvents(filter) => Ok(ChainResponse::MainEvents(
                self.inner.get_main_events(filter)?,
            )),
            ChainRequest::GetBlockIds {
                start_number,
                reverse,
                max_size,
            } => Ok(ChainResponse::HashVec(self.inner.get_block_ids(
                start_number,
                reverse,
                max_size,
            )?)),
            ChainRequest::GetBlocks(ids) => {
                Ok(ChainResponse::BlockOptionVec(self.inner.get_blocks(ids)?))
            }
            ChainRequest::GetHeaders(ids) => {
                Ok(ChainResponse::BlockHeaderVec(self.inner.get_headers(ids)?))
            }
            ChainRequest::GetTransactionInfos {
                start_index,
                reverse,
                max_size,
            } => Ok(ChainResponse::TransactionInfos(
                self.inner
                    .get_transaction_infos(start_index, reverse, max_size)?,
            )),
            ChainRequest::GetTransactionProof {
                block_id,
                transaction_global_index,
                event_index,
                access_path,
            } => Ok(ChainResponse::TransactionProof(Box::new(
                self.inner.get_transaction_proof(
                    block_id,
                    transaction_global_index,
                    event_index,
                    access_path,
                )?,
            ))),
            ChainRequest::GetBlockInfos(ids) => Ok(ChainResponse::BlockInfoVec(Box::new(
                self.inner.get_block_infos(ids)?,
            ))),
            ChainRequest::GetDagAccumulatorLeaves {
                start_index,
                batch_size,
            } => Ok(ChainResponse::TargetDagAccumulatorLeaf(
                self.inner
                    .get_dag_accumulator_leaves(GetDagAccumulatorLeaves {
                        accumulator_leaf_index: start_index,
                        batch_size,
                    })?,
            )),
            ChainRequest::GetTargetDagAccumulatorLeafDetail {
                leaf_index,
                batch_size,
            } => Ok(ChainResponse::TargetDagAccumulatorLeafDetail(
                self.inner.get_target_dag_accumulator_leaf_detail(
                    GetTargetDagAccumulatorLeafDetail {
                        leaf_index,
                        batch_size,
                    },
                )?,
            )),
        }
    }
}

pub struct ChainReaderServiceInner {
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    main: BlockChain,
    storage: Arc<dyn Store>,
    flexidag_service: ServiceRef<FlexidagService>,
    vm_metrics: Option<VMMetrics>,
}

impl ChainReaderServiceInner {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        flexidag_service: ServiceRef<FlexidagService>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let net = config.net();
        let main = BlockChain::new(
            net.time_service(),
            startup_info.main,
            storage.clone(),
            config.net().id().clone(),
            vm_metrics.clone(),
        )?;
        Ok(Self {
            config,
            startup_info,
            main,
            storage,
            flexidag_service,
            vm_metrics,
        })
    }

    pub fn get_main(&self) -> &BlockChain {
        &self.main
    }

    pub fn update_chain_head(&mut self, block: ExecutedBlock) -> Result<()> {
        self.main.connect(block)?;
        Ok(())
    }

    pub fn switch_main(&mut self, new_head_id: HashValue) -> Result<()> {
        let net = self.config.net();
        self.main = BlockChain::new(
            net.time_service(),
            new_head_id,
            self.storage.clone(),
            self.config.net().id().clone(),
            self.vm_metrics.clone(),
        )?;
        Ok(())
    }

    pub fn update_dag_accumulator(&mut self, new_block_header: BlockHeader) -> Result<()> {
        async_std::task::block_on(self.flexidag_service.send(UpdateDagTips {
            block_header: new_block_header,
            current_head_block_id: self.main.status().info().id(),
            k_total_difficulty: KTotalDifficulty {
                head_block_id: self.main.status().info().id(),
                total_difficulty: self.main.status().info().get_total_difficulty(),
            },
        }))?
    }
}

impl ReadableChainService for ChainReaderServiceInner {
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        self.storage.get_block_header_by_hash(hash)
    }

    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(hash)
    }

    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        self.storage.get_blocks(ids)
    }

    fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockHeader>>> {
        Ok(self
            .get_blocks(ids)?
            .into_iter()
            .map(|block| block.map(|b| b.header))
            .collect())
    }

    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>> {
        self.storage.get_block_info(hash)
    }

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>, Error> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> Result<Option<RichTransactionInfo>, Error> {
        self.main.get_transaction_info(txn_hash)
    }

    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<RichTransactionInfo>, Error> {
        self.storage.get_block_transaction_infos(block_id)
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>, Error> {
        self.storage
            .get_transaction_info_by_block_and_index(block_id, idx)
    }
    fn get_events_by_txn_info_hash(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        self.storage.get_contract_events(txn_info_id)
    }

    fn main_head_header(&self) -> BlockHeader {
        self.main.current_header()
    }

    fn main_head_block(&self) -> Block {
        self.main.head_block().block
    }

    fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        self.main.get_block_by_number(number)
    }

    fn main_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        self.main.get_header_by_number(number)
    }

    fn main_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>> {
        self.main.get_block_info_by_number(number)
    }

    fn main_startup_info(&self) -> StartupInfo {
        self.startup_info.clone()
    }
    fn main_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>> {
        self.main.get_blocks_by_number(number, reverse, count)
    }

    fn get_main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
        self.main.filter_events(filter)
    }

    fn get_block_ids(
        &self,
        start_number: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>> {
        self.main.get_block_ids(start_number, reverse, max_size)
    }

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>> {
        self.main
            .get_transaction_infos(start_index, reverse, max_size)
    }

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        self.main.get_transaction_proof(
            block_id,
            transaction_global_index,
            event_index,
            access_path,
        )
    }

    fn get_block_infos(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        self.storage.get_block_infos(ids)
    }

    fn get_dag_accumulator_leaves(
        &self,
        req: GetDagAccumulatorLeaves,
    ) -> anyhow::Result<Vec<TargetDagAccumulatorLeaf>> {
        Ok(async_std::task::block_on(self.flexidag_service.send(
            flexidag_service::GetDagAccumulatorLeaves {
                leaf_index: req.accumulator_leaf_index,
                batch_size: req.batch_size,
                reverse: true,
            },
        ))??
        .into_iter()
        .map(|leaf| TargetDagAccumulatorLeaf {
            accumulator_root: leaf.dag_accumulator_root,
            leaf_index: leaf.leaf_index,
        })
        .collect())
    }

    fn get_target_dag_accumulator_leaf_detail(
        &self,
        req: GetTargetDagAccumulatorLeafDetail,
    ) -> anyhow::Result<Vec<TargetDagAccumulatorLeafDetail>> {
        let dag_details =
            async_std::task::block_on(self.flexidag_service.send(GetDagAccumulatorLeafDetail {
                leaf_index: req.leaf_index,
                batch_size: req.batch_size,
            }))??;
        Ok(dag_details
            .into_iter()
            .map(|detail| TargetDagAccumulatorLeafDetail {
                accumulator_root: detail.accumulator_root,
                tips: detail.tips,
            })
            .collect())
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
        let (storage, chain_info, _) = test_helper::Genesis::init_storage_for_test(config.net())?;
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        registry.put_shared(storage).await?;
        let service_ref = registry.register::<ChainReaderService>().await?;
        let chain_status = service_ref.main_status().await?;
        assert_eq!(&chain_status, chain_info.status());
        Ok(())
    }
}
