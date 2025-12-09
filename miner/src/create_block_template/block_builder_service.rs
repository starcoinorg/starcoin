use std::collections::{BTreeMap, HashSet, VecDeque};
use std::{cmp::min, sync::Arc};

use anyhow::{format_err, Result};
use futures::executor::block_on;
use rand::seq::SliceRandom;
use rand::Rng;
use starcoin_chain::{
    get_merge_bound_hash, global_block_state_cache, is_node_shutting_down, BlockChain,
    CachedBlockState, ChainReader,
};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::{BlockDAG, MineNewDagBlockInfo};
use starcoin_dag::consensusdb::schemadb::RelationsStoreReader;
use starcoin_dag::reachability::reachability_service::ReachabilityService;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
use starcoin_open_block::OpenedBlock;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_state_api::ChainStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::BlockStore;
use starcoin_storage::{Storage, Store};
use starcoin_storage::{Storage2, Store2};
use starcoin_txpool::{NonceCache, PoolClient, TxPoolService};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::blockhash::BlockHashSet;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::system_events::GenerateBlockEvent;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate, Version},
    transaction::SignedUserTransaction,
};
use starcoin_vm2_account_api::{AccountAsyncService, AccountInfo, DefaultAccountChangeEvent};
use starcoin_vm2_account_service::AccountService;
use starcoin_vm2_state_api::ChainStateReader as ChainStateReader2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use std::sync::RwLock;

use crate::{MinerService, NewHeaderChannel};

use super::metrics::BlockBuilderMetrics;
use sp_utils::thread_pool::RAYON_EXEC_POOL;
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;

#[derive(Clone, Debug)]
pub struct MinerResponse {
    pub previous_header: BlockHeader,
    pub selected_parents: Vec<HashValue>,
    pub strategy: ConsensusStrategy,
    pub on_chain_block_gas_limit: u64,
    pub next_difficulty: U256,
    pub now_milliseconds: u64,
    pub pruning_point: HashValue,
    pub ghostdata: GhostdagData,
    pub max_transaction_per_block: u64,
}

enum MergesetIncreaseResult {
    Accepted { increase_size: u64 },
    Rejected { new_candidate: HashValue },
}

#[derive(Debug, Clone)]
pub struct BlockTemplateRequest {
    pub event: GenerateBlockEvent,
}

#[derive(Debug, Clone)]
pub struct BlockTemplateResponse {
    pub parent: BlockHeader,
    pub template: BlockTemplate,
    pub event: GenerateBlockEvent,
}

pub struct BlockBuilderService {
    inner: Inner<TxPoolService>,
    new_header_channel: NewHeaderChannel,
}

enum ReceiveHeader {
    Received,
    NotReceived,
}

impl BlockBuilderService {
    fn receive_header(&mut self) -> ReceiveHeader {
        match self.new_header_channel.new_header_receiver.try_recv() {
            Ok(new_header) => {
                match self
                    .inner
                    .set_current_block_header(new_header.as_ref().clone())
                {
                    Ok(()) => ReceiveHeader::Received,
                    Err(e) => panic!(
                        "Failed to set current block header: {:?} in BlockBuilderService",
                        e
                    ),
                }
            }
            Err(e) => match e {
                crossbeam::channel::TryRecvError::Empty => ReceiveHeader::NotReceived,
                crossbeam::channel::TryRecvError::Disconnected => {
                    panic!("the new headerchannel is disconnected")
                }
            },
        }
    }
}

impl ServiceFactory<Self> for BlockBuilderService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
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
        //Currently use vm2 account as default
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
            storage2,
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
        ctx.subscribe::<BlockTemplateRequest>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<DefaultAccountChangeEvent>();
        ctx.unsubscribe::<BlockTemplateRequest>();
        Ok(())
    }
}

impl EventHandler<Self, DefaultAccountChangeEvent> for BlockBuilderService {
    fn handle_event(&mut self, msg: DefaultAccountChangeEvent, _ctx: &mut ServiceContext<Self>) {
        info!("Miner account change to {}", msg.new_account.address);

        match self.inner.miner_account.write() {
            Ok(mut account) => *account = msg.new_account,
            Err(e) => {
                error!("Failed to acquire write lock for miner_account: {:?}. Miner may use outdated account!", e);
            }
        }
    }
}

pub trait BlockTemplateCallBack {
    fn block_template_callback(
        &mut self,
        parent_header: BlockHeader,
        block_template: BlockTemplate,
    ) -> Result<()>;
}
struct BlockBuilderTemplateNotify {
    miner_service: ServiceRef<MinerService>,
    event: GenerateBlockEvent,
}

impl BlockBuilderTemplateNotify {
    pub fn new(miner_service: ServiceRef<MinerService>, event: GenerateBlockEvent) -> Self {
        Self {
            miner_service,
            event,
        }
    }
}

impl BlockTemplateCallBack for BlockBuilderTemplateNotify {
    fn block_template_callback(
        &mut self,
        parent: BlockHeader,
        block_template: BlockTemplate,
    ) -> Result<()> {
        self.miner_service
            .notify(BlockTemplateResponse {
                parent,
                template: block_template,
                event: self.event.clone(),
            })
            .map_err(|e| format_err!("failed to notify block template for: {:?}", e))
    }
}

impl EventHandler<Self, BlockTemplateRequest> for BlockBuilderService {
    fn handle_event(&mut self, msg: BlockTemplateRequest, ctx: &mut ServiceContext<Self>) {
        // TODO: Get block_header_version from GenesisConfig according to dag-master's implementation
        let header_version = 1u8; // Default block header version for now
        let miner_service = ctx
            .service_ref::<MinerService>()
            .expect("MinerService should exist")
            .clone();
        let _ = self.receive_header();
        let callback = BlockBuilderTemplateNotify::new(miner_service, msg.event);
        if let Err(e) = self
            .inner
            .create_block_template(header_version, Box::new(callback))
        {
            error!("Failed to create block template: {}", e);
        }
    }
}

pub trait TemplateTxProvider {
    fn get_txns_with_header(
        &self,
        max: u64,
        header: &BlockHeader,
    ) -> Vec<MultiSignedUserTransaction>;
    fn get_pending_with_state_dbs(
        &self,
        max: u64,
        current_timestamp_secs: u64,
        state_root1: HashValue,
        state_root2: HashValue,
        statedb: Arc<ChainStateDB>,
        statedb2: Arc<ChainStateDB2>,
    ) -> Vec<MultiSignedUserTransaction>;
    fn remove_invalid_txn(&self, txn_hash: HashValue);
}

pub struct EmptyProvider;

impl TemplateTxProvider for EmptyProvider {
    fn get_txns_with_header(
        &self,
        _max: u64,
        _header: &BlockHeader,
    ) -> Vec<MultiSignedUserTransaction> {
        vec![]
    }

    fn get_pending_with_state_dbs(
        &self,
        _max: u64,
        _current_timestamp_secs: u64,
        _state_root1: HashValue,
        _state_root2: HashValue,
        _statedb: Arc<ChainStateDB>,
        _statedb2: Arc<ChainStateDB2>,
    ) -> Vec<MultiSignedUserTransaction> {
        vec![]
    }
    fn remove_invalid_txn(&self, _txn_hash: HashValue) {}
}

impl TemplateTxProvider for TxPoolService {
    fn get_txns_with_header(
        &self,
        max: u64,
        header: &BlockHeader,
    ) -> Vec<MultiSignedUserTransaction> {
        let (state_root1, state_root2) = match self.get_store().get_vm_multi_state(header.id()) {
            Ok(state) => (state.state_root1(), state.state_root2()),
            Err(e) => {
                error!(
                    "Failed to get vm_multi_state when creating block template: {}",
                    e
                );
                return vec![];
            }
        };
        self.get_pending_with_state(max, None, state_root1, state_root2)
            .unwrap_or_else(|e| {
                error!(
                    "Failed to get pending txns when creating block template: {}",
                    e
                );
                vec![]
            })
    }

    fn get_pending_with_state_dbs(
        &self,
        max: u64,
        current_timestamp_secs: u64,
        state_root1: HashValue,
        state_root2: HashValue,
        statedb: Arc<ChainStateDB>,
        statedb2: Arc<ChainStateDB2>,
    ) -> Vec<MultiSignedUserTransaction> {
        let pool_client = PoolClient::new_with_state_dbs(
            state_root1,
            state_root2,
            statedb,
            statedb2,
            NonceCache::new(0),
            None,
        );
        self.inner
            .get_pending_with_pool_client(max, current_timestamp_secs, pool_client)
            .into_iter()
            .map(|t| t.signed().clone())
            .collect()
    }

    fn remove_invalid_txn(&self, txn_hash: HashValue) {
        self.remove_txn(txn_hash, true);
    }
}

pub struct Inner<P> {
    storage: Arc<dyn Store>,
    storage2: Arc<dyn Store2>,
    tx_provider: P,
    local_block_gas_limit: Option<u64>,
    miner_account: RwLock<AccountInfo>,
    main: BlockHeader,
    config: Arc<NodeConfig>,
    dag: BlockDAG,
    genesis_hash: HashValue,
    #[allow(unused)]
    metrics: Option<BlockBuilderMetrics>,
    vm_metrics: Option<VMMetrics>,
}

impl<P> Inner<P>
where
    P: TemplateTxProvider + TxPoolSyncService + 'static,
{
    pub fn new(
        header: BlockHeader,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        tx_provider: P,
        local_block_gas_limit: Option<u64>,
        miner_account: AccountInfo,
        dag: BlockDAG,
        config: Arc<NodeConfig>,
        metrics: Option<BlockBuilderMetrics>,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<Self> {
        let genesis_hash = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("Can not find genesis hash"))?;
        Ok(Self {
            storage: storage.clone(),
            storage2,
            tx_provider,
            local_block_gas_limit,
            miner_account: RwLock::new(miner_account),
            main: header,
            config,
            dag,
            metrics,
            vm_metrics,
            genesis_hash,
        })
    }

    fn resolve_block_parents(&mut self) -> Result<(MinerResponse, BlockChain)> {
        let MineNewDagBlockInfo {
            selected_parents,
            ghostdata,
            pruning_point,
        } = {
            // get the current pruning point and the current dag state, which contains the tip blocks, some of which may be the selected parents
            let pruning_point = if self.main.pruning_point() == HashValue::zero() {
                self.genesis_hash
            } else {
                self.main.pruning_point()
            };

            // calculate the next pruning point and
            // prune the tips and
            // calculate the ghost data and
            // sort the selected parents by work type and blue score
            let MineNewDagBlockInfo {
                selected_parents, // ordered parents by work type or blue score
                ghostdata,
                pruning_point,
            } = self.dag.calc_mergeset_and_tips(
                pruning_point,
                self.storage
                    .get_genesis()?
                    .expect("genesis not found when resolve block parents"),
            )?;

            self.update_main_chain(ghostdata.selected_parent)?;

            let parents_candidates = self.merge_size_limit_filter(
                ghostdata.selected_parent,
                selected_parents
                    .into_iter()
                    .filter(|id| *id != ghostdata.selected_parent)
                    .collect(),
            )?;

            let merge_bound_hash = get_merge_bound_hash(
                ghostdata.selected_parent,
                self.dag.clone(),
                self.storage.clone(),
            )?;

            let (selected_parents, ghostdata) = self.dag.remove_bounded_merge_breaking_parents(
                parents_candidates,
                ghostdata,
                pruning_point,
                merge_bound_hash,
            )?;

            self.update_main_chain(ghostdata.selected_parent)?;

            MineNewDagBlockInfo {
                selected_parents,
                ghostdata,
                pruning_point,
            }
        };

        let selected_parent = ghostdata.selected_parent;

        let main = BlockChain::new(
            self.config.net().time_service().clone(),
            selected_parent,
            self.storage.clone(),
            self.storage2.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )?;

        let epoch = main.epoch().clone();
        let strategy = main.consensus_strategy();
        let max_transaction_per_block = epoch.max_transaction_per_block();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let previous_header = self
            .storage
            .get_block_header_by_hash(selected_parent)?
            .ok_or_else(|| format_err!("BlockHeader should exist by hash: {}", selected_parent))?;
        let next_difficulty = strategy.calculate_next_difficulty(&main)?;
        let now_milliseconds = self.config.net().time_service().now_millis();

        Ok((
            MinerResponse {
                previous_header,
                on_chain_block_gas_limit,
                selected_parents,
                strategy,
                next_difficulty,
                now_milliseconds,
                pruning_point,
                ghostdata,
                max_transaction_per_block,
            },
            main,
        ))
    }

    pub fn create_block_template(
        &mut self,
        version: Version,
        mut block_template_call_back: Box<dyn BlockTemplateCallBack + Send + Sync>,
        // miner_service: Option<ServiceRef<MinerService>>,
        // event: Option<GenerateBlockEvent>,
    ) -> Result<()> {
        let (
            MinerResponse {
                previous_header,
                selected_parents,
                strategy,
                on_chain_block_gas_limit,
                next_difficulty: difficulty,
                now_milliseconds: mut now_millis,
                pruning_point,
                ghostdata,
                max_transaction_per_block,
            },
            main,
        ) = self.resolve_block_parents()?;

        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);

        //TODO use a GasConstant value to replace 200.
        // block_gas_limit / min_gas_per_txn
        let max_txns = min((block_gas_limit / 200) * 2, max_transaction_per_block);

        let author = *self
            .miner_account
            .read()
            .map_err(|e| format_err!("Failed to acquire read lock for miner_account: {:?}", e))?
            .address();

        if now_millis <= previous_header.timestamp() {
            info!(
                "Adjust new block timestamp by parent timestamp, parent.timestamp: {}, now: {}, gap: {}",
                previous_header.timestamp(), now_millis, previous_header.timestamp() - now_millis,
            );
            now_millis = previous_header.timestamp() + 1;
        }

        let blue_blocks = ghostdata
            .mergeset_blues
            .iter()
            .skip(1) // skip the selected parent
            .map(|hash| self.storage.get_block_by_hash(*hash))
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
            "[CreateBlockTemplate] previous_header: {:?}, block_gas_limit: {}, max_txns: {}, uncles len: {}, timestamp: {}",
            previous_header,
            block_gas_limit,
            max_txns,
            uncles.len(),
            now_millis,
        );

        let (blue_txns, blue_txns2, seen_hashes) = collect_blue_transactions(&blue_blocks);
        info!(
            "[BlockProcess] Blue VM1 txns len: {}, Blue VM2 txns len: {}",
            blue_txns.len(),
            blue_txns2.len()
        );

        let storage = self.storage.clone();
        let storage2 = self.storage2.clone();
        let vm_metrics = self.vm_metrics.clone();
        let tx_provider = self.tx_provider.clone();

        let vm1_offline = previous_header.number().saturating_add(1)
            >= vm1_offline_height(previous_header.chain_id().id().into());

        RAYON_EXEC_POOL.spawn(move || {
            if is_node_shutting_down() {
                return;
            }

            let mut opened_block = match OpenedBlock::new(
                storage.clone(),
                storage2.clone(),
                previous_header.clone(),
                block_gas_limit,
                author,
                now_millis,
                uncles.clone(),
                difficulty,
                strategy,
                vm_metrics.clone(),
                selected_parents,
                version,
                pruning_point,
                ghostdata.mergeset_reds.len() as u64,
                main.into_state_dbs(),
            ) {
                Ok(opened_block) => opened_block,
                Err(e) => {
                    error!("[BlockProcess] open block error: {}", e);
                    return;
                }
            };

            if is_node_shutting_down() {
                return;
            }

            let mut seen_hashes = seen_hashes;
            let mut included_blue = 0usize;

            // Process blue VM1 transactions (includes VM1 block metadata)
            if !vm1_offline {
                let blue_vm1_len = blue_txns.len();
                let excluded_txns = match opened_block.process_vm1_transactions(blue_txns) {
                    Ok(excluded_txns) => excluded_txns,
                    Err(e) => {
                        error!("[BlockProcess] process vm1 transactions error: {}", e);
                        return;
                    }
                };
                for invalid_txn in &excluded_txns.discarded_txns {
                    tx_provider.remove_invalid_txn(invalid_txn.id());
                }
                let included_vm1 = blue_vm1_len
                    .saturating_sub(excluded_txns.discarded_txns.len())
                    .saturating_sub(excluded_txns.untouched_txns.len());
                included_blue += included_vm1;
                info!(
                    "[BlockProcess] Blue VM1 included: {}, discarded: {}, untouched: {}",
                    included_vm1,
                    excluded_txns.discarded_txns.len(),
                    excluded_txns.untouched_txns.len(),
                );
            }

            if is_node_shutting_down() {
                return;
            }

            // Process blue VM2 transactions
            let blue_vm2_len = blue_txns2.len();
            if blue_vm2_len > 0 {
                let excluded_txns2 = match opened_block.push_txns2(blue_txns2) {
                    Ok(excluded_txns) => excluded_txns,
                    Err(e) => {
                        error!("[BlockProcess] push txns2 error: {}", e);
                        return;
                    }
                };
                for invalid_txn in &excluded_txns2.discarded_txns {
                    tx_provider.remove_invalid_txn(invalid_txn.id());
                }
                let included_vm2 = blue_vm2_len
                    .saturating_sub(excluded_txns2.discarded_txns.len())
                    .saturating_sub(excluded_txns2.untouched_txns.len());
                included_blue += included_vm2;
                info!(
                    "[BlockProcess] Blue VM2 included: {}, discarded: {}, untouched: {}",
                    included_vm2,
                    excluded_txns2.discarded_txns.len(),
                    excluded_txns2.untouched_txns.len()
                );
            }

            if is_node_shutting_down() {
                return;
            }

            let remaining_max = max_txns.saturating_sub(included_blue as u64);
            let mut pending_transactions = Vec::new();
            let mut pending_transactions2 = Vec::new();
            if remaining_max > 0 && opened_block.gas_left() > 0 {
                let statedb = opened_block.state_db();
                let statedb2 = opened_block.state_db2();
                let state_root1 = statedb.state_root();
                let state_root2 = statedb2.state_root();
                let current_timestamp_secs = now_millis / 1000;
                let pending_multi_transactions = tx_provider.get_pending_with_state_dbs(
                    remaining_max,
                    current_timestamp_secs,
                    state_root1,
                    state_root2,
                    statedb,
                    statedb2,
                );
                for txn in pending_multi_transactions {
                    match txn {
                        MultiSignedUserTransaction::VM1(txn) => {
                            if vm1_offline {
                                continue;
                            }
                            if seen_hashes.insert(txn.id()) {
                                pending_transactions.push(txn);
                            }
                        }
                        MultiSignedUserTransaction::VM2(txn) => {
                            let hash = HashValue::new(txn.id().to_inner());
                            if seen_hashes.insert(hash) {
                                pending_transactions2.push(txn);
                            }
                        }
                    }
                }
            }

            // Post-process ordering for txpool only.
            let pending_transactions = round_robin_by_sender(
                pending_transactions,
                |txn: &SignedUserTransaction| txn.sender(),
                |txn: &SignedUserTransaction| txn.sequence_number(),
            );
            let pending_transactions2 = round_robin_by_sender(
                pending_transactions2,
                |txn: &SignedUserTransaction2| txn.sender(),
                |txn: &SignedUserTransaction2| txn.sequence_number(),
            );
            info!(
                "[BlockProcess] TxPool VM1 txns len: {}, TxPool VM2 txns len: {}",
                pending_transactions.len(),
                pending_transactions2.len()
            );
            let pending_vm1_len = pending_transactions.len();
            let pending_vm2_len = pending_transactions2.len();

            // Process txpool VM1 transactions
            if !vm1_offline && !pending_transactions.is_empty() {
                let excluded_txns = match opened_block.push_txns(pending_transactions) {
                    Ok(excluded_txns) => excluded_txns,
                    Err(e) => {
                        error!("[BlockProcess] push vm1 txns error: {}", e);
                        return;
                    }
                };
                for invalid_txn in &excluded_txns.discarded_txns {
                    tx_provider.remove_invalid_txn(invalid_txn.id());
                }
                let included_vm1 = pending_vm1_len
                    .saturating_sub(excluded_txns.discarded_txns.len())
                    .saturating_sub(excluded_txns.untouched_txns.len());
                info!(
                    "[BlockProcess] TxPool VM1 included: {}, discarded: {}, untouched: {}",
                    included_vm1,
                    excluded_txns.discarded_txns.len(),
                    excluded_txns.untouched_txns.len(),
                );
            }

            // Process txpool VM2 transactions
            if !pending_transactions2.is_empty() {
                let excluded_txns2 = match opened_block.push_txns2(pending_transactions2) {
                    Ok(excluded_txns) => excluded_txns,
                    Err(e) => {
                        error!("[BlockProcess] push txns2 error: {}", e);
                        return;
                    }
                };
                for invalid_txn in &excluded_txns2.discarded_txns {
                    tx_provider.remove_invalid_txn(invalid_txn.id());
                }

                let included_vm2 = pending_vm2_len
                    .saturating_sub(excluded_txns2.discarded_txns.len())
                    .saturating_sub(excluded_txns2.untouched_txns.len());
                info!(
                    "[BlockProcess] TxPool VM2 included: {}, discarded: {}, untouched: {}",
                    included_vm2,
                    excluded_txns2.discarded_txns.len(),
                    excluded_txns2.untouched_txns.len()
                );
            }

            if is_node_shutting_down() {
                return;
            }

            if !opened_block.included_user_txns2().is_empty() {
                match opened_block.finalize_block_epilogue() {
                    Ok(()) => {}
                    Err(e) => {
                        error!("[BlockProcess] finalize block epilogue error: {}", e);
                        return;
                    }
                }
            }

            if is_node_shutting_down() {
                return;
            }

            let finalized = match opened_block.finalize() {
                Ok(result) => result,
                Err(e) => {
                    error!("[BlockProcess] finalize block error: {}", e);
                    return;
                }
            };

            if is_node_shutting_down() {
                return;
            }

            let template = finalized.template;

            // Cache the StateDB and executed data for reuse during block execution.
            // This allows skipping re-execution and apply_write_set entirely.
            // BlockExecutedData contains all necessary data: txn_infos, events, table_infos, write_sets.
            let state_cache = global_block_state_cache();
            state_cache.insert(
                template.txn_accumulator_root,
                CachedBlockState::new(
                    Some(finalized.statedb),
                    Some(finalized.statedb2),
                    Some(finalized.executed_data),
                    Some(finalized.executed_data2),
                ),
            );

            if let Err(e) =
                block_template_call_back.block_template_callback(previous_header, template)
            {
                error!("[BlockProcess] notify BlockTemplateResponse error: {}", e);
            }
        });
        Ok(())
    }

    fn fetch_transactions(
        &self,
        header: &BlockHeader,
        blue_blocks: &[Block],
        max_txns: u64,
    ) -> Result<(Vec<SignedUserTransaction>, Vec<SignedUserTransaction2>)> {
        let pending_multi_transactions = self.tx_provider.get_txns_with_header(max_txns, header);

        // Separate VM1 and VM2 transactions
        let mut pending_transactions = vec![];
        let mut pending_transactions2 = vec![];
        pending_multi_transactions
            .into_iter()
            .for_each(|txn| match txn {
                MultiSignedUserTransaction::VM1(txn) => pending_transactions.push(txn),
                MultiSignedUserTransaction::VM2(txn) => pending_transactions2.push(txn),
            });

        return Ok((pending_transactions, pending_transactions2));
        // if pending_transactions.len() + pending_transactions2.len() >= max_txns as usize {
        //     return Ok((pending_transactions, pending_transactions2));
        // }

        // blue_blocks.iter().for_each(|block| {
        //     block.transactions().iter().for_each(|transaction| {
        //         pending_transactions.push(transaction.clone());
        //     });

        //     block.transactions2().iter().for_each(|transaction| {
        //         pending_transactions2.push(transaction.clone());
        //     })
        // });

        // pending_transactions.sort_by(|a, b| match a.sender().cmp(&b.sender()) {
        //     std::cmp::Ordering::Equal => a.sequence_number().cmp(&b.sequence_number()),
        //     other => other,
        // });

        // pending_transactions2.sort_by(|a, b| match a.sender().cmp(&b.sender()) {
        //     std::cmp::Ordering::Equal => a.sequence_number().cmp(&b.sequence_number()),
        //     other => other,
        // });

        // Ok((pending_transactions, pending_transactions2))
    }
    pub fn set_current_block_header(&mut self, header: BlockHeader) -> Result<()> {
        if self.main.id() == header.id() {
            return Ok(());
        }
        self.main = header;
        Ok(())
    }

    fn merge_size_limit_filter(
        &self,
        selected_parent: HashValue,
        mut candidates: VecDeque<HashValue>,
    ) -> Result<Vec<HashValue>> {
        let max_block_parents = self.config.miner.maximum_parents_count();
        let max_candidates: usize = max_block_parents * 3;

        // Prioritize half the blocks with highest blue work and pick the rest randomly to ensure diversity between nodes
        if candidates.len() > max_candidates {
            // make_contiguous should be a no op since the deque was just built
            let slice = candidates.make_contiguous();

            // Keep slice[..max_block_parents / 2] as is, choose max_candidates - max_block_parents / 2 in random
            // from the remainder of the slice while swapping them to slice[max_block_parents / 2..max_candidates].
            //
            // Inspired by rand::partial_shuffle (which lacks the guarantee on chosen elements location).
            for i in max_block_parents / 2..max_candidates {
                if i >= slice.len() {
                    break;
                }
                let j = rand::rng().random_range(i..slice.len()); // i < max_candidates < slice.len()
                slice.swap(i, j);
            }

            // Truncate the unchosen elements
            candidates.truncate(max_candidates);
        } else if candidates.len() > max_block_parents / 2 {
            // Fallback to a simpler algo in this case
            candidates.make_contiguous()[max_block_parents / 2..].shuffle(&mut rand::rng());
        }

        let mut parents = Vec::with_capacity(min(max_block_parents, candidates.len() + 1));
        parents.push(selected_parent);
        let mut mergeset_size = 1;
        let mergeset_size_limit = self.dag.mergeset_size_limit();

        // Try adding parents as long as mergeset size and number of parents limits are not reached
        while let Some(candidate) = candidates.pop_front() {
            if mergeset_size >= mergeset_size_limit || parents.len() >= max_block_parents {
                break;
            }
            match self.mergeset_increase(
                &parents,
                candidate,
                mergeset_size_limit - mergeset_size,
            )? {
                MergesetIncreaseResult::Accepted { increase_size } => {
                    mergeset_size += increase_size;
                    parents.push(candidate);
                }
                MergesetIncreaseResult::Rejected { new_candidate } => {
                    // If we already have a candidate in the past of new candidate then skip.
                    if self
                        .dag
                        .reachability_service()
                        .is_any_dag_ancestor(&mut candidates.iter().copied(), new_candidate)
                    {
                        continue; // TODO (optimization): not sure this check is needed if candidates invariant as antichain is kept
                    }
                    // Remove all candidates which are in the future of the new candidate
                    candidates.retain(|&h| {
                        !self
                            .dag
                            .reachability_service()
                            .is_dag_ancestor_of(new_candidate, h)
                    });
                    candidates.push_back(new_candidate);
                }
            }
        }

        Ok(parents)
    }

    fn mergeset_increase(
        &self,
        selected_parents: &[HashValue],
        candidate: HashValue,
        budget: u64,
    ) -> Result<MergesetIncreaseResult> {
        /*
        Algo:
            Traverse past(candidate) \setminus past(selected_parents) and make
            sure the increase in mergeset size is within the available budget
        */

        let mut queue = self
            .dag
            .storage
            .relations_store
            .read()
            .get_parents(candidate)?
            .iter()
            .cloned()
            .collect::<VecDeque<_>>();
        let mut visited: BlockHashSet = queue.iter().copied().collect();
        let mut mergeset_increase = 1u64; // Starts with 1 to count for the candidate itself

        while let Some(current) = queue.pop_front() {
            if self
                .dag
                .reachability_service()
                .is_dag_ancestor_of_any(current, &mut selected_parents.iter().copied())
            {
                continue;
            }
            mergeset_increase += 1;
            if mergeset_increase > budget {
                return Ok(MergesetIncreaseResult::Rejected {
                    new_candidate: current,
                });
            }

            let current_parents = self
                .dag
                .storage
                .relations_store
                .read()
                .get_parents(current)?;
            for &parent in current_parents.iter() {
                if visited.insert(parent) {
                    queue.push_back(parent);
                }
            }
        }
        Ok(MergesetIncreaseResult::Accepted {
            increase_size: mergeset_increase,
        })
    }

    fn update_main_chain(&mut self, selected_parent: HashValue) -> Result<()> {
        if self.main.id() != selected_parent {
            self.main = self
                .storage
                .get_block_header_by_hash(selected_parent)?
                .ok_or_else(|| {
                    format_err!(
                        "Cannot find the selected parent {} in storage",
                        selected_parent
                    )
                })?;
        }
        Ok(())
    }
}

fn collect_blue_transactions(
    blue_blocks: &[Block],
) -> (
    Vec<SignedUserTransaction>,
    Vec<SignedUserTransaction2>,
    HashSet<HashValue>,
) {
    let mut pending_transactions = Vec::new();
    let mut pending_transactions2 = Vec::new();
    let mut seen_hashes = HashSet::new();

    for block in blue_blocks {
        for transaction in block.transactions() {
            if seen_hashes.insert(transaction.id()) {
                pending_transactions.push(transaction.clone());
            }
        }
        for transaction in block.transactions2() {
            let hash = HashValue::new(transaction.id().to_inner());
            if seen_hashes.insert(hash) {
                pending_transactions2.push(transaction.clone());
            }
        }
    }

    (pending_transactions, pending_transactions2, seen_hashes)
}

fn round_robin_by_sender<T, K, FSender, FSeq>(txns: Vec<T>, sender: FSender, seq: FSeq) -> Vec<T>
where
    K: Ord,
    FSender: Fn(&T) -> K,
    FSeq: Fn(&T) -> u64,
{
    if txns.len() <= 1 {
        return txns;
    }

    let total = txns.len();
    let mut buckets: BTreeMap<K, Vec<T>> = BTreeMap::new();
    for txn in txns {
        buckets.entry(sender(&txn)).or_default().push(txn);
    }

    let mut queues: BTreeMap<K, VecDeque<T>> = BTreeMap::new();
    for (key, mut bucket) in buckets {
        bucket.sort_by_key(|txn| seq(txn));
        queues.insert(key, VecDeque::from(bucket));
    }

    let mut ordered = Vec::with_capacity(total);
    loop {
        let mut pushed = false;
        for queue in queues.values_mut() {
            if let Some(txn) = queue.pop_front() {
                ordered.push(txn);
                pushed = true;
            }
        }
        if !pushed {
            break;
        }
    }

    ordered
}
