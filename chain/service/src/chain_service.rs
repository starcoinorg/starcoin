// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Error, Result};
use starcoin_chain::BlockChain;
use starcoin_chain_api::message::{BlockColor, BlockColorInfo, ChainRequest, ChainResponse};
use starcoin_chain_api::range_locate::{self, RangeInLocation};
use starcoin_chain_api::{
    ChainReader, ChainWriter, ReadableChainService, TransactionInfoWithProof,
};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::{BlockDAG, DagBlockColor};
use starcoin_dag::consensusdb::consensus_state::DagStateView;
use starcoin_dag::consensusdb::schemadb::GhostdagStoreReader;
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_dag::GetAbsentBlock;
use starcoin_logger::prelude::*;
use starcoin_metrics::metrics::VMMetrics;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::{BlockStore, Storage, Storage2, Store, Store2};
use starcoin_types::block::ExecutedBlock;
use starcoin_types::contract_event::{StcContractEvent, StcContractEventInfo};
use starcoin_types::filter::Filter;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_types::system_events::{NewDagBlock, NewHeadBlock};
use starcoin_types::transaction::{StcRichTransactionInfo, StcTransaction};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    contract_event::ContractEvent,
    startup_info::StartupInfo,
};
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
        storage2: Arc<dyn Store2>,
        dag: BlockDAG,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        Ok(Self {
            inner: ChainReaderServiceInner::new(
                config,
                startup_info,
                storage,
                storage2,
                dag,
                vm_metrics,
            )?,
        })
    }
}

impl ServiceFactory<Self> for ChainReaderService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("StartupInfo should exist at service init."))?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        let vm_metrics = ctx.get_shared_opt::<VMMetrics>()?;
        Self::new(config, startup_info, storage, storage2, dag, vm_metrics)
    }
}

impl ActorService for ChainReaderService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewDagBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewDagBlock>();
        Ok(())
    }
}

impl EventHandler<Self, NewDagBlock> for ChainReaderService {
    fn handle_event(&mut self, event: NewDagBlock, _ctx: &mut ServiceContext<Self>) {
        info!("NewDagBlock in chain reader service");
        let mut main = self
            .inner
            .get_main()
            .fork(self.inner.main_head_header().id())
            .unwrap_or_else(|e| {
                panic!(
                    "fork error when handle NewDagBlock in chain reader service: {:?}",
                    e
                )
            });
        self.inner.main = main
            .select_dag_state(event.executed_block.as_ref().header())
            .unwrap_or_else(|e| {
                panic!(
                    "select_dag_state error when handle NewDagBlock in chain reader service: {:?}",
                    e
                )
            });
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainReaderService {
    fn handle_event(&mut self, event: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
        let new_head = event.executed_block.block().header().clone();
        if let Err(e) = if self
            .inner
            .get_main()
            .can_connect(event.executed_block.as_ref())
        {
            self.inner
                .update_chain_head(event.executed_block.as_ref().clone())
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
        _ctx: &mut ServiceContext<ChainReaderService>,
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
            ChainRequest::GetBlockTransactionInfosInSeq(block_id) => {
                Ok(ChainResponse::TransactionInfosInSeq(
                    self.inner.get_block_txn_infos_in_seq(block_id)?,
                ))
            }
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
                    .get_events_by_txn_info_hash(txn_info.transaction_info.id())?
                    .unwrap_or_default();

                let event_infos = if events.is_empty() {
                    vec![]
                } else {
                    events
                        .into_iter()
                        .enumerate()
                        .map(|(idx, evt)| StcContractEventInfo {
                            block_hash: txn_info.block_id,
                            block_number: txn_info.block_number,
                            transaction_hash: txn_hash,
                            transaction_index: txn_info.transaction_index,
                            transaction_global_index: txn_info.transaction_global_index,
                            event_index: idx as u32,
                            event: evt.into(),
                        })
                        .collect()
                };
                Ok(ChainResponse::Events(event_infos))
            }
            ChainRequest::GetEventsByTxnHash2 { txn_hash } => Ok(ChainResponse::Events(
                self.inner.get_events_by_txn_hash2(txn_hash)?,
            )),
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
            ChainRequest::GetDagBlockChildren { block_ids } => Ok(ChainResponse::HashVec(
                self.inner.get_dag_block_children(block_ids)?,
            )),
            ChainRequest::GetDagStateView => Ok(ChainResponse::DagStateView(Box::new(
                self.inner.get_dag_state()?,
            ))),
            ChainRequest::GetGhostdagData(ids) => Ok(ChainResponse::GhostdagDataOption(Box::new(
                self.inner.get_ghostdagdata(ids)?,
            ))),
            ChainRequest::GetCurrentBlockColor(block_id) => Ok(ChainResponse::BlockColorOption(
                Box::new(self.inner.get_current_block_color(block_id)?),
            )),
            ChainRequest::IsAncestorOfCommand {
                ancestor,
                descendants,
            } => Ok(ChainResponse::IsAncestorOfCommand {
                reachability_view: self.inner.dag.is_ancestor_of(ancestor, descendants)?,
            }),
            ChainRequest::GetMultiStateByHash(hash) => {
                let state = self.inner.storage.get_vm_multi_state(hash)?;
                Ok(ChainResponse::MultiStateResp(Some(state)))
            }
            ChainRequest::GetRangeInLocation { start_id, end_id } => {
                Ok(ChainResponse::GetRangeInLocation {
                    range: self.inner.get_range_in_location(start_id, end_id)?,
                })
            }
            ChainRequest::GetAbsentBlocks { absent_id, exp } => {
                Ok(ChainResponse::GetAbsentBlocks {
                    absent_blocks: self.inner.get_absent_blocks(absent_id, exp)?,
                })
            }
        }
    }
}

pub struct ChainReaderServiceInner {
    config: Arc<NodeConfig>,
    startup_info: StartupInfo,
    main: BlockChain,
    storage: Arc<dyn Store>,
    storage2: Arc<dyn Store2>,
    dag: BlockDAG,
    vm_metrics: Option<VMMetrics>,
}

impl ChainReaderServiceInner {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        dag: BlockDAG,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let net = config.net();
        let main = BlockChain::new(
            net.time_service(),
            startup_info.main,
            storage.clone(),
            storage2.clone(),
            vm_metrics.clone(),
            dag.clone(),
        )?;
        Ok(Self {
            config,
            startup_info,
            main,
            storage,
            storage2,
            dag,
            vm_metrics,
        })
    }

    pub fn get_main(&self) -> &BlockChain {
        &self.main
    }

    pub fn get_storages(&self) -> (Arc<dyn Store>, Arc<dyn Store2>) {
        (self.storage.clone(), self.storage2.clone())
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
            self.storage2.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )?;
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

    fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>> {
        self.storage.get_transaction(txn_hash)
    }

    fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> Result<Option<StcRichTransactionInfo>, Error> {
        let txn_info_ids = self
            .storage
            .get_rich_transaction_info_ids_by_txn_hash(txn_hash)?;
        for txn_info_id in txn_info_ids {
            let txn_info = match self
                .storage
                .get_transaction_info_by_rich_info_id(txn_info_id)?
            {
                Some(info) => info,
                None => continue,
            };
            if let Some(color) = self.get_current_block_color(txn_info.block_id())? {
                if matches!(color.color, BlockColor::Blue) {
                    return Ok(Some(txn_info));
                }
            }
        }
        Ok(None)
    }

    fn get_block_txn_infos(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>, Error> {
        self.storage.get_block_transaction_infos(block_id)
    }

    fn get_block_txn_infos_in_seq(
        &self,
        block_id: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>, Error> {
        let mut all_txn_infos = self.storage.get_block_transaction_infos(block_id)?;
        all_txn_infos.sort_by_key(|info| info.transaction_global_index);
        Ok(all_txn_infos)
    }

    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<StcRichTransactionInfo>, Error> {
        self.storage
            .get_transaction_info_by_block_and_index(block_id, idx)
    }
    fn get_events_by_txn_info_hash(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>, Error> {
        self.storage.get_contract_events(txn_info_id)
    }

    fn get_events_by_txn_info_hash2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>> {
        self.storage.get_contract_events_v2(txn_info_id)
    }

    fn main_head_header(&self) -> BlockHeader {
        self.main.current_header()
    }

    fn main_head_block(&self) -> Block {
        self.main.head_block().block().clone()
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

    fn get_main_events(&self, filter: Filter) -> Result<Vec<StcContractEventInfo>> {
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
    ) -> Result<Vec<StcRichTransactionInfo>> {
        self.main
            .get_transaction_infos(start_index, reverse, max_size)
    }

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
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

    fn get_dag_block_children(&self, ids: Vec<HashValue>) -> Result<Vec<HashValue>> {
        ids.into_iter().try_fold(vec![], |mut result, id| {
            self.dag.get_children(id).map(|children| {
                result.extend(children);
                result
            })
        })
    }

    fn get_dag_state(&self) -> Result<DagStateView> {
        let state = self.main.get_dag_state()?;
        let pruning_point = self.main.status().head().pruning_point();
        Ok(DagStateView {
            tips: state.tips,
            pruning_point,
        })
    }

    fn get_ghostdagdata(&self, ids: Vec<HashValue>) -> Result<Vec<Option<GhostdagData>>> {
        let arc_results = self.dag.ghostdata_by_hashes(&ids)?;

        let results = arc_results
            .into_iter()
            .map(|maybe_arc| {
                maybe_arc.map(|arc| Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()))
            })
            .collect();

        Ok(results)
    }

    fn get_current_block_color(&self, block_id: HashValue) -> Result<Option<BlockColorInfo>> {
        if self.storage.get_block_header_by_hash(block_id)?.is_none() {
            return Ok(None);
        }
        let chain_head = self.main.current_header().id();
        Ok(self.dag.get_block_color(block_id, chain_head)?.map(|info| {
            let color = match info.color {
                DagBlockColor::Blue => BlockColor::Blue,
                DagBlockColor::Red => BlockColor::Red,
            };
            BlockColorInfo {
                color,
                confirmed_block: info.confirmed_block,
            }
        }))
    }

    fn get_range_in_location(
        &self,
        start_id: HashValue,
        end_id: Option<HashValue>,
    ) -> Result<RangeInLocation> {
        range_locate::get_range_in_location(self.get_main(), self.storage.clone(), start_id, end_id)
    }

    fn get_absent_blocks(&self, absent_id: Vec<HashValue>, exp: u64) -> Result<Vec<Block>> {
        let result = self
            .dag
            .get_absent_blocks(GetAbsentBlock { absent_id, exp })?;

        let origin_id = self.config.net().genesis_block_parameter().parent_hash;
        let genesis_id = self
            .storage
            .get_genesis()?
            .unwrap_or_else(|| panic!("genesis not exist"));

        result
            .absent_blocks
            .into_iter()
            .filter(|id| *id != origin_id && *id != genesis_id)
            .map(|block_id| match self.storage.get_block(block_id) {
                Ok(op_block) => {
                    op_block.ok_or_else(|| format_err!("block {:?} should exist", block_id))
                }
                Err(e) => bail!(
                    "in get absent blocks, get block {:?} err: {:?}",
                    block_id,
                    e
                ),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_account_api::AccountInfo;
    use starcoin_chain_api::ChainAsyncService;
    use starcoin_config::NodeConfig;
    use starcoin_consensus::Consensus;
    use starcoin_service_registry::{RegistryAsyncService, RegistryService};
    use starcoin_types::startup_info::StartupInfo;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(config.net())?;
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        registry.put_shared(storage).await?;
        registry.put_shared(storage2).await?;
        registry.put_shared(dag).await?;
        let service_ref = registry.register::<ChainReaderService>().await?;
        let chain_status = service_ref.main_status().await?;
        assert_eq!(&chain_status, chain_info.status());
        Ok(())
    }

    fn create_block_with_tips(
        chain: &BlockChain,
        author: AccountInfo,
        tips: Vec<HashValue>,
        net: &starcoin_config::ChainNetwork,
    ) -> Result<Block> {
        let ghostdata = chain.dag().ghost_dag_manager().ghostdag(&tips)?;
        let parent_header = chain
            .get_storage()
            .get_block_header_by_hash(ghostdata.selected_parent)?
            .ok_or_else(|| {
                format_err!("cannot find parent header {:?}", ghostdata.selected_parent)
            })?;
        let (template, _) = chain.create_block_template(
            *author.address(),
            Some(parent_header),
            Vec::new(),
            None,
            None,
            Some(tips),
            HashValue::zero(),
        )?;
        chain
            .consensus()
            .create_block(template, net.time_service().as_ref())
    }

    fn create_block_simple(
        chain: &BlockChain,
        author: AccountInfo,
        net: &starcoin_config::ChainNetwork,
    ) -> Result<Block> {
        let (template, _) = chain.create_block_template_simple(*author.address())?;
        chain
            .consensus()
            .create_block(template, net.time_service().as_ref())
    }

    fn apply_block_on_parent(
        base: &BlockChain,
        author: AccountInfo,
        parent: HashValue,
        tips: Vec<HashValue>,
        net: &starcoin_config::ChainNetwork,
    ) -> Result<Block> {
        let mut branch = base.fork(parent)?;
        let block = create_block_with_tips(&branch, author, tips, net)?;
        branch.apply(block.clone())?;
        Ok(block)
    }

    #[stest::test]
    fn test_get_current_block_color_merge_blue() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let net = config.net().clone();
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(&net)?;
        let mut chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        )?;

        let miner = AccountInfo::random();
        let genesis_id = chain.current_header().id();

        let b1 = create_block_with_tips(&chain, miner.clone(), vec![genesis_id], &net)?;
        chain.apply(b1.clone())?;

        let mut fork_chain = chain.fork(genesis_id)?;
        let c1 = create_block_with_tips(&fork_chain, miner.clone(), vec![genesis_id], &net)?;
        fork_chain.apply(c1.clone())?;

        let merge = create_block_with_tips(&chain, miner, vec![b1.id(), c1.id()], &net)?;
        chain.apply(merge.clone())?;

        assert_eq!(chain.current_header().id(), merge.id());

        let service_inner = ChainReaderServiceInner::new(
            config.clone(),
            StartupInfo::new(merge.id()),
            storage,
            storage2,
            dag,
            None,
        )?;

        let info = service_inner
            .get_current_block_color(c1.id())?
            .expect("c1 should be colored by merge block");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, merge.id());
        Ok(())
    }

    #[stest::test]
    fn test_get_current_block_color_no_merge() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let net = config.net().clone();
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(&net)?;
        let mut chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        )?;

        let miner = AccountInfo::random();
        let genesis_id = chain.current_header().id();

        let b1 = create_block_with_tips(&chain, miner.clone(), vec![genesis_id], &net)?;
        chain.apply(b1.clone())?;

        let mut fork_chain = chain.fork(genesis_id)?;
        let c1 = create_block_with_tips(&fork_chain, miner.clone(), vec![genesis_id], &net)?;
        fork_chain.apply(c1.clone())?;

        let b2 = create_block_with_tips(&chain, miner, vec![b1.id()], &net)?;
        chain.apply(b2.clone())?;

        assert_eq!(chain.current_header().id(), b2.id());

        let service_inner = ChainReaderServiceInner::new(
            config,
            StartupInfo::new(b2.id()),
            storage,
            storage2,
            dag,
            None,
        )?;

        let info = service_inner
            .get_current_block_color(b1.id())?
            .expect("b1 should be colored by b2");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, b2.id());

        let none = service_inner.get_current_block_color(c1.id())?;
        assert!(none.is_none());
        Ok(())
    }

    #[stest::test]
    fn test_get_current_block_color_no_merge_long_fork() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let net = config.net().clone();
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(&net)?;
        let mut chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        )?;

        let miner = AccountInfo::random();
        let mut main_blocks = Vec::new();
        for _ in 0..6 {
            let block = create_block_simple(&chain, miner.clone(), &net)?;
            chain.apply(block.clone())?;
            main_blocks.push(block);
        }
        let fork_point = main_blocks[1].id();
        let b5 = main_blocks[4].clone();
        let head = main_blocks[5].clone();

        let c1 = apply_block_on_parent(&chain, miner.clone(), fork_point, vec![fork_point], &net)?;
        let c2 = apply_block_on_parent(&chain, miner.clone(), c1.id(), vec![c1.id()], &net)?;
        let _c3 = apply_block_on_parent(&chain, miner.clone(), c2.id(), vec![c2.id()], &net)?;

        assert_eq!(chain.current_header().id(), head.id());

        let service_inner = ChainReaderServiceInner::new(
            config,
            StartupInfo::new(head.id()),
            storage,
            storage2,
            dag,
            None,
        )?;

        let info = service_inner
            .get_current_block_color(b5.id())?
            .expect("b5 should be colored by head");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, head.id());

        let none = service_inner.get_current_block_color(c2.id())?;
        assert!(none.is_none());
        Ok(())
    }

    #[stest::test]
    fn test_get_current_block_color_reorg() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let net = config.net().clone();
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(&net)?;
        let mut chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        )?;

        let miner = AccountInfo::random();

        let a1 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(a1.clone())?;
        let a2 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(a2.clone())?;
        let a3 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(a3.clone())?;
        let a4 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(a4.clone())?;

        let b1 = apply_block_on_parent(&chain, miner.clone(), a1.id(), vec![a1.id()], &net)?;
        let b2 = apply_block_on_parent(&chain, miner.clone(), b1.id(), vec![b1.id()], &net)?;
        let b3 = apply_block_on_parent(&chain, miner.clone(), b2.id(), vec![b2.id()], &net)?;
        let b4 = apply_block_on_parent(&chain, miner.clone(), b3.id(), vec![b3.id()], &net)?;

        let service_inner = ChainReaderServiceInner::new(
            config,
            StartupInfo::new(b4.id()),
            storage,
            storage2,
            dag,
            None,
        )?;

        let info = service_inner
            .get_current_block_color(a1.id())?
            .expect("a1 should be colored on new main chain");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, b1.id());

        let none = service_inner.get_current_block_color(a3.id())?;
        assert!(none.is_none());
        Ok(())
    }

    #[stest::test]
    fn test_get_current_block_color_nearest_confirm() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let net = config.net().clone();
        let (storage, storage2, chain_info, _, dag) =
            test_helper::Genesis::init_storage_for_test(&net)?;
        let mut chain = BlockChain::new(
            net.time_service(),
            chain_info.head().id(),
            storage.clone(),
            storage2.clone(),
            None,
            dag.clone(),
        )?;

        let miner = AccountInfo::random();

        let b1 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(b1.clone())?;
        let b2 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(b2.clone())?;
        let b3 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(b3.clone())?;
        let b4 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(b4.clone())?;
        let b5 = create_block_simple(&chain, miner.clone(), &net)?;
        chain.apply(b5.clone())?;

        let service_inner = ChainReaderServiceInner::new(
            config,
            StartupInfo::new(b5.id()),
            storage,
            storage2,
            dag,
            None,
        )?;

        let info = service_inner
            .get_current_block_color(b2.id())?
            .expect("b2 should be colored by its selected-parent child");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, b3.id());

        let info = service_inner
            .get_current_block_color(b4.id())?
            .expect("b4 should be colored by b5");
        assert!(matches!(info.color, BlockColor::Blue));
        assert_eq!(info.confirmed_block, b5.id());
        Ok(())
    }
}
