use std::collections::{HashMap, VecDeque};
use std::{cmp::min, sync::Arc};

use anyhow::{format_err, Result};
use futures::executor::block_on;
use rand::seq::SliceRandom;
use rand::Rng;
use starcoin_account_api::{AccountAsyncService, AccountInfo, DefaultAccountChangeEvent};
use starcoin_account_service::AccountService;
use starcoin_accumulator::Accumulator;
use starcoin_chain::{
    block_merkle_tree_from_header, get_merge_bound_hash, BlockChain, ChainReader,
};
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
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use starcoin_storage::BlockStore;
use starcoin_storage::{Storage, Store};
use starcoin_sync::block_connector::MinerResponse;
use starcoin_txpool::{Pool, TxPoolService};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::blockhash::BlockHashSet;
use starcoin_types::{
    block::{Block, BlockHeader, BlockTemplate, Version},
    transaction::SignedUserTransaction,
};
use std::sync::RwLock;

use crate::NewHeaderChannel;

use super::metrics::BlockBuilderMetrics;

enum MergesetIncreaseResult {
    Accepted { increase_size: u64 },
    Rejected { new_candidate: HashValue },
}

#[derive(Debug)]
pub enum BlockTemplateError {
    NoReceivedHeader,
    Other(anyhow::Error),
}

#[derive(Debug)]
pub struct BlockTemplateRequest;

impl ServiceRequest for BlockTemplateRequest {
    type Response = std::result::Result<BlockTemplateResponse, BlockTemplateError>;
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

enum ReceiveHeader {
    Received,
    NotReceived,
}

impl BlockBuilderService {
    fn receive_header(&mut self) -> ReceiveHeader {
        info!("jacktest: receive header in block builder service");
        match self.new_header_channel.new_header_receiver.try_recv() {
            Ok(new_header) => {
                info!("jacktest: receive header in block builder service2");
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

        match self.inner.miner_account.write() {
            Ok(mut account) => *account = msg.new_account,
            Err(e) => {
                error!("Failed to acquire write lock for miner_account: {:?}. Miner may use outdated account!", e);
            }
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
        let _ = self.receive_header();
        self.inner
            .create_block_template(header_version)
            .map_err(BlockTemplateError::Other)
    }
}

pub trait TemplateTxProvider {
    fn get_txns_with_header(&self, max: u64, header: &BlockHeader) -> Vec<SignedUserTransaction>;
    fn remove_invalid_txn(&self, txn_hash: HashValue);
    fn try_read(&self) -> Option<parking_lot::RwLockReadGuard<Pool>>;
}

pub struct EmptyProvider;

impl TemplateTxProvider for EmptyProvider {
    fn get_txns_with_header(&self, _max: u64, _header: &BlockHeader) -> Vec<SignedUserTransaction> {
        vec![]
    }
    fn remove_invalid_txn(&self, _txn_hash: HashValue) {}
    fn try_read(&self) -> Option<parking_lot::RwLockReadGuard<Pool>> {
        None
    }
}

impl TemplateTxProvider for TxPoolService {
    fn get_txns_with_header(&self, max: u64, header: &BlockHeader) -> Vec<SignedUserTransaction> {
        self.get_pending_with_header(max, None, header)
    }

    fn remove_invalid_txn(&self, txn_hash: HashValue) {
        self.remove_txn(txn_hash, true);
    }

    fn try_read(&self) -> Option<parking_lot::RwLockReadGuard<Pool>> {
        self.inner.try_read()
    }
}

pub struct Inner<P> {
    storage: Arc<dyn Store>,
    tx_provider: P,
    local_block_gas_limit: Option<u64>,
    miner_account: RwLock<AccountInfo>,
    main: BlockHeader,
    config: Arc<NodeConfig>,
    dag: BlockDAG,
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
            main: header,
            config,
            dag,
            metrics,
            vm_metrics,
        })
    }

    fn resolve_block_parents(&mut self) -> Result<(MinerResponse, BlockChain)> {
        info!("jacktest: block template resolve block parents");
        let MineNewDagBlockInfo {
            selected_parents,
            ghostdata,
            pruning_point,
        } = {
            info!("jacktest: block template main is {:?}", self.main);
            // get the current pruning point and the current dag state, which contains the tip blocks, some of which may be the selected parents
            let pruning_point = self.main.pruning_point();
            info!("jacktest: block template resolve block parents2");

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
            info!("jacktest: block template resolve block parents3");

            self.update_main_chain(ghostdata.selected_parent)?;
            info!("jacktest: block template resolve block parents4");

            // filter the parent candidates that bring too many ancestors which are not the descendants of the selected parent
            let parents_candidates = self.merge_size_limit_filter(
                ghostdata.selected_parent,
                selected_parents
                    .into_iter()
                    .filter(|id| *id != ghostdata.selected_parent)
                    .collect(),
            )?;
            info!("jacktest: block template resolve block parents5");

            let merge_bound_hash = get_merge_bound_hash(
                ghostdata.selected_parent,
                self.dag.clone(),
                self.storage.clone(),
            )?;
            info!("jacktest: block template resolve block parents6");

            let (selected_parents, ghostdata) = self.dag.remove_bounded_merge_breaking_parents(
                parents_candidates,
                ghostdata,
                pruning_point,
                merge_bound_hash,
            )?;
            info!("jacktest: block template resolve block parents7");

            self.update_main_chain(ghostdata.selected_parent)?;
            info!("jacktest: block template resolve block parents8");

            MineNewDagBlockInfo {
                selected_parents,
                ghostdata,
                pruning_point,
            }
        };

        info!("jacktest: block template resolve block parents9");
        let selected_parent = ghostdata.selected_parent;

        info!("jacktest: block template resolve block parents11");

        let main = BlockChain::new(
            self.config.net().time_service().clone(),
            selected_parent,
            self.storage.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
        )?;

        info!("jacktest: block template resolve block parents11.1");
        let epoch = main.epoch().clone();
        let strategy = epoch.strategy();
        let max_transaction_per_block = epoch.max_transaction_per_block();
        let on_chain_block_gas_limit = epoch.block_gas_limit();
        let previous_header = self
            .storage
            .get_block_header_by_hash(selected_parent)?
            .ok_or_else(|| format_err!("BlockHeader should exist by hash: {}", selected_parent))?;
        info!("jacktest: block template resolve block parents12");
        let next_difficulty = epoch.strategy().calculate_next_difficulty(&main)?;
        info!("jacktest: block template resolve block parents13");
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

    pub fn create_block_template(&mut self, _version: Version) -> Result<BlockTemplateResponse> {
        info!("jacktest: create block template 1");
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
        info!("jacktest: create block template 2");

        let block_gas_limit = self
            .local_block_gas_limit
            .map(|block_gas_limit| min(block_gas_limit, on_chain_block_gas_limit))
            .unwrap_or(on_chain_block_gas_limit);
        info!("jacktest: create block template 3");

        //TODO use a GasConstant value to replace 200.
        // block_gas_limit / min_gas_per_txn
        let max_txns = min((block_gas_limit / 200) * 2, max_transaction_per_block);
        info!("jacktest: create block template 4");

        let author = *self
            .miner_account
            .read()
            .map_err(|e| format_err!("Failed to acquire read lock for miner_account: {:?}", e))?
            .address();
        info!("jacktest: create block template 5");

        if now_millis <= previous_header.timestamp() {
            info!(
                "Adjust new block timestamp by parent timestamp, parent.timestamp: {}, now: {}, gap: {}",
                previous_header.timestamp(), now_millis, previous_header.timestamp() - now_millis,
            );
            now_millis = previous_header.timestamp() + 1;
        }
        info!("jacktest: create block template 6");

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

        info!("jacktest: create block template 7");

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

        let header_version = 1;

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
            selected_parents,
            header_version,
            pruning_point,
            ghostdata.mergeset_reds.len() as u64,
            main.statedb(),
        )?;

        let txn = self.fetch_transactions(&previous_header, &blue_blocks, max_txns)?;
        let excluded_txns = opened_block.push_txns(txn)?;
        for invalid_txn in excluded_txns.discarded_txns {
            self.tx_provider.remove_invalid_txn(invalid_txn.id());
        }

        let template = opened_block.finalize()?;
        Ok(BlockTemplateResponse {
            parent: previous_header,
            template,
        })
    }

    fn fetch_transactions(
        &self,
        selected_header: &BlockHeader,
        blue_blocks: &[Block],
        max_txns: u64,
    ) -> Result<Vec<SignedUserTransaction>> {
        let pending_transactions = self
            .tx_provider
            .get_txns_with_header(max_txns, selected_header);

        if pending_transactions.len() >= max_txns as usize {
            return Ok(pending_transactions);
        }

        let mut pending_transaction_map =
            HashMap::<AccountAddress, Vec<SignedUserTransaction>>::new();
        pending_transactions.into_iter().for_each(|transaction| {
            pending_transaction_map
                .entry(transaction.sender())
                .or_default()
                .push(transaction);
        });

        let mut uncle_transaction_map =
            HashMap::<AccountAddress, Vec<SignedUserTransaction>>::new();
        blue_blocks.iter().for_each(|block| {
            block.transactions().iter().for_each(|transaction| {
                uncle_transaction_map
                    .entry(transaction.sender())
                    .or_default()
                    .push(transaction.clone());
            })
        });

        for transactions in uncle_transaction_map.values_mut() {
            if transactions.len() <= 1 {
                continue;
            }

            let mut index = 1;
            while index < transactions.len() {
                if transactions[index].sequence_number()
                    != transactions[index - 1].sequence_number() + 1
                {
                    break;
                }
                index += 1;
            }
            transactions.truncate(index);
        }

        for (sender, uncle_transactions) in uncle_transaction_map.iter() {
            if let Some(pending_transactions) = pending_transaction_map.get_mut(sender) {
                let pending_last_seq = pending_transactions
                    .last()
                    .expect("transaction not found in pending transactions")
                    .sequence_number();
                if let Some(index) = uncle_transactions
                    .iter()
                    .position(|transaction| transaction.sequence_number() == pending_last_seq)
                {
                    pending_transactions.extend_from_slice(&uncle_transactions[(index + 1)..]);
                }
            } else if let Some(next_seq) = self
                .tx_provider
                .next_sequence_number_with_header(*sender, selected_header)
            {
                if let Some(index) = uncle_transactions
                    .iter()
                    .position(|transaction| transaction.sequence_number() == next_seq)
                {
                    pending_transaction_map.insert(*sender, uncle_transactions[index..].to_vec());
                }
            }
        }

        Ok(pending_transaction_map
            .iter()
            .flat_map(|(_sender, transactions)| transactions.clone())
            .take(max_txns as usize)
            .collect())
    }

    pub fn set_current_block_header(&mut self, header: BlockHeader) -> Result<()> {
        info!("jacktest: receive header in block builder service3");
        if self.main.id() == header.id() {
            return Ok(());
        }
        info!("jacktest: receive header in block builder service4");
        self.main = header;
        info!("jacktest: receive header in block builder service5");
        Ok(())
    }

    fn merge_size_limit_filter(
        &self,
        selected_parent: HashValue,
        mut candidates: VecDeque<HashValue>,
    ) -> Result<Vec<HashValue>> {
        let max_block_parents: usize = usize::try_from(self.config.miner.maximum_parents_count())?;
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
                let j = rand::thread_rng().gen_range(i..slice.len()); // i < max_candidates < slice.len()
                slice.swap(i, j);
            }

            // Truncate the unchosen elements
            candidates.truncate(max_candidates);
        } else if candidates.len() > max_block_parents / 2 {
            // Fallback to a simpler algo in this case
            candidates.make_contiguous()[max_block_parents / 2..].shuffle(&mut rand::thread_rng());
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
