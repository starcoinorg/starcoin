// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{BlockConnectedEventHandle, BlockFetcher, BlockLocalStore};
use crate::verified_rpc_client::RpcVerifyError;
use anyhow::{anyhow, format_err, Ok, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::PeerId;
use network_api::PeerProvider;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::{verifier::BasicVerifier, BlockChain};
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, ExecutedBlock};
use starcoin_config::G_CRATE_VERSION;
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_flexidag::flexidag_service::{AddToDag, FinishSync, ForkDagAccumulator};
use starcoin_flexidag::FlexidagService;
use starcoin_logger::prelude::*;
use starcoin_service_registry::ServiceRef;
use starcoin_storage::{Store, BARNARD_HARD_FORK_HASH};
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockInfo, BlockNumber};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use stream_task::{CollectorState, TaskError, TaskResultCollector, TaskState};

use super::{BlockConnectAction, BlockConnectedEvent, BlockConnectedFinishEvent};

#[derive(Clone, Debug)]
pub struct SyncBlockData {
    pub(crate) block: Block,
    pub(crate) info: Option<BlockInfo>,
    pub(crate) peer_id: Option<PeerId>,
    pub(crate) accumulator_root: Option<HashValue>, // the block belongs to this dag accumulator leaf
    pub(crate) count_in_leaf: u64, // the count of the block in the dag accumulator leaf
    pub(crate) dag_accumulator_index: Option<u64>, // the index of the accumulator leaf which the block belogs to
}

impl SyncBlockData {
    pub fn new(
        block: Block,
        block_info: Option<BlockInfo>,
        peer_id: Option<PeerId>,
        accumulator_root: Option<HashValue>,
        count_in_leaf: u64,
        dag_accumulator_index: Option<u64>,
    ) -> Self {
        Self {
            block,
            info: block_info,
            peer_id,
            accumulator_root,
            count_in_leaf,
            dag_accumulator_index,
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<(Block, Option<BlockInfo>, Option<PeerId>)> for SyncBlockData {
    fn into(self) -> (Block, Option<BlockInfo>, Option<PeerId>) {
        (self.block, self.info, self.peer_id)
    }
}

#[derive(Clone)]
pub struct BlockSyncTask {
    accumulator: Arc<MerkleAccumulator>,
    start_number: BlockNumber,
    fetcher: Arc<dyn BlockFetcher>,
    // if check_local_store is true, get block from local first.
    check_local_store: bool,
    local_store: Arc<dyn BlockLocalStore>,
    batch_size: u64,
}

impl BlockSyncTask {
    pub fn new<F, S>(
        accumulator: MerkleAccumulator,
        ancestor: BlockIdAndNumber,
        fetcher: F,
        check_local_store: bool,
        local_store: S,
        batch_size: u64,
    ) -> Self
    where
        F: BlockFetcher + 'static,
        S: BlockLocalStore + 'static,
    {
        //start_number is include, so start from ancestor.number + 1
        let start_number = ancestor.number.saturating_add(1);
        info!(
            "[sync] Start sync block, ancestor: {:?}, start_number: {}, check_local_store: {:?}, target_number: {}",
            ancestor, start_number, check_local_store, accumulator.num_leaves().saturating_sub(1) );

        Self {
            accumulator: Arc::new(accumulator),
            start_number,
            fetcher: Arc::new(fetcher),
            check_local_store,
            local_store: Arc::new(local_store),
            batch_size,
        }
    }
}

impl TaskState for BlockSyncTask {
    type Item = SyncBlockData;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let block_ids =
                self.accumulator
                    .get_leaves(self.start_number, false, self.batch_size)?;
            if block_ids.is_empty() {
                return Ok(vec![]);
            }
            if self.check_local_store {
                let block_with_info = self.local_store.get_block_with_info(block_ids.clone())?;
                let (no_exist_block_ids, result_map) =
                    block_ids.clone().into_iter().zip(block_with_info).fold(
                        (vec![], HashMap::new()),
                        |(mut no_exist_block_ids, mut result_map), (block_id, block_with_info)| {
                            match block_with_info {
                                Some(block_data) => {
                                    result_map.insert(block_id, block_data);
                                }
                                None => {
                                    no_exist_block_ids.push(block_id);
                                }
                            }
                            (no_exist_block_ids, result_map)
                        },
                    );
                debug!(
                    "[sync] get_block_with_info from local store, ids: {}, found: {}",
                    block_ids.len(),
                    result_map.len()
                );
                let mut result_map = if no_exist_block_ids.is_empty() {
                    result_map
                } else {
                    self.fetcher
                        .fetch_blocks(no_exist_block_ids)
                        .await?
                        .into_iter()
                        .fold(result_map, |mut result_map, (block, peer_id)| {
                            result_map.insert(
                                block.id(),
                                SyncBlockData::new(block, None, peer_id, None, 1, None),
                            );
                            result_map
                        })
                };
                //ensure return block's order same as request block_id's order.
                let result: Result<Vec<SyncBlockData>> = block_ids
                    .iter()
                    .map(|block_id| {
                        result_map
                            .remove(block_id)
                            .ok_or_else(|| format_err!("Get block by id {:?} failed", block_id))
                    })
                    .collect();
                result
            } else {
                Ok(self
                    .fetcher
                    .fetch_blocks(block_ids)
                    .await?
                    .into_iter()
                    .map(|(block, peer_id)| SyncBlockData::new(block, None, peer_id, None, 1, None))
                    .collect())
            }
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        let next_start_number = self.start_number.saturating_add(self.batch_size);
        if next_start_number > self.accumulator.num_leaves() {
            None
        } else {
            Some(Self {
                accumulator: self.accumulator.clone(),
                start_number: next_start_number,
                fetcher: self.fetcher.clone(),
                check_local_store: self.check_local_store,
                local_store: self.local_store.clone(),
                batch_size: self.batch_size,
            })
        }
    }

    fn total_items(&self) -> Option<u64> {
        Some(
            self.accumulator
                .num_leaves()
                .saturating_sub(self.start_number),
        )
    }
}

pub struct BlockCollector<N, H> {
    //node's current block info
    current_block_info: BlockInfo,
    target: Option<SyncTarget>, // single chain use only
    // the block chain init by ancestor
    chain: BlockChain,
    event_handle: H,
    peer_provider: N,
    skip_pow_verify: bool,
    last_accumulator_root: HashValue,
    dag_block_pool: Vec<SyncBlockData>,
    target_accumulator_root: HashValue,
    flexidag_service: Option<ServiceRef<FlexidagService>>,
    new_dag_accumulator_info: Option<AccumulatorInfo>,
    storage: Arc<dyn Store>,
}

impl<N, H> BlockCollector<N, H>
where
    N: PeerProvider + 'static,
    H: BlockConnectedEventHandle + 'static,
{
    pub fn new_with_handle(
        current_block_info: BlockInfo,
        target: Option<SyncTarget>,
        chain: BlockChain,
        event_handle: H,
        peer_provider: N,
        skip_pow_verify: bool,
        target_accumulator_root: HashValue,
        flexidag_service: Option<ServiceRef<FlexidagService>>,
        storage: Arc<dyn Store>,
    ) -> Self {
        Self {
            current_block_info,
            target,
            chain,
            event_handle,
            peer_provider,
            skip_pow_verify,
            last_accumulator_root: HashValue::zero(),
            dag_block_pool: Vec::new(),
            target_accumulator_root,
            flexidag_service,
            new_dag_accumulator_info: None,
            storage,
        }
    }

    #[cfg(test)]
    pub fn apply_block_for_test(
        &mut self,
        block: Block,
        parents_hash: Option<Vec<HashValue>>,
        next_tips: &mut Option<Vec<HashValue>>,
    ) -> Result<()> {
        self.apply_block(block, None)
    }

    fn notify_connected_block(
        &mut self,
        block: Block,
        block_info: BlockInfo,
        action: BlockConnectAction,
        state: CollectorState,
    ) -> Result<CollectorState> {
        let total_difficulty = block_info.get_total_difficulty();

        // if the new block's total difficulty is smaller than the current,
        // do nothing because we do not need to update the current chain in any other services.
        if total_difficulty <= self.current_block_info.total_difficulty {
            return Ok(state); // nothing to do
        }

        // only try connect block when sync chain total_difficulty > node's current chain.

        // first, create the sender and receiver for ensuring that
        // the last block is connected before the next synchronization is triggered.
        // if the block is not the last one, we do not want to do this.
        let (sender, mut receiver) = match state {
            CollectorState::Enough => {
                let (s, r) = futures::channel::mpsc::unbounded::<BlockConnectedFinishEvent>();
                (Some(s), Some(r))
            }
            CollectorState::Need => (None, None),
        };

        // second, construct the block connect event.
        let block_connect_event = BlockConnectedEvent {
            block,
            feedback: sender,
            action,
        };

        // third, broadcast it.
        if let Err(e) = self.event_handle.handle(block_connect_event.clone()) {
            error!(
                "Send BlockConnectedEvent error: {:?}, block_id: {}",
                e,
                block_info.block_id()
            );
        }

        // finally, if it is the last one, wait for the last block to be processed.
        if block_connect_event.feedback.is_some() && receiver.is_some() {
            let mut count: i32 = 0;
            while count < 3 {
                count = count.saturating_add(1);
                match receiver.as_mut().unwrap().try_next() {
                    std::result::Result::Ok(_) => {
                        break;
                    }
                    Err(_) => {
                        info!("Waiting for last block to be processed");
                        async_std::task::block_on(async_std::task::sleep(Duration::from_secs(10)));
                    }
                }
            }
        }
        Ok(state)
    }

    fn apply_block(&mut self, block: Block, peer_id: Option<PeerId>) -> Result<()> {
        if let Some((_failed_block, pre_peer_id, err, version)) = self
            .chain
            .get_storage()
            .get_failed_block_by_id(block.id())?
        {
            if version == *G_CRATE_VERSION {
                warn!(
                    "[sync] apply a previous failed block: {}, previous_peer_id:{:?}, err: {}",
                    block.id(),
                    pre_peer_id,
                    err
                );
                if let Some(peer) = peer_id {
                    self.peer_provider
                        .report_peer(peer, ConnectBlockError::REP_VERIFY_BLOCK_FAILED);
                }
                return Err(format_err!("collect previous failed block:{}", block.id()));
            }
        }
        if block.id() == *BARNARD_HARD_FORK_HASH {
            if let Some(peer) = peer_id {
                warn!("[barnard hard fork] ban peer {}", peer);
                self.peer_provider.ban_peer(peer, true);
            }
            return Err(format_err!("reject barnard hard fork block:{}", block.id()));
        }
        let apply_result = if self.skip_pow_verify {
            self.chain
                .apply_with_verifier::<BasicVerifier>(block.clone())
        } else {
            self.chain.apply(block.clone())
        };
        if let Err(err) = apply_result {
            let error_msg = err.to_string();
            error!(
                "[sync] collect block error: {:?}, peer_id:{:?} ",
                error_msg, peer_id
            );
            match err.downcast::<ConnectBlockError>() {
                std::result::Result::Ok(connect_error) => match connect_error {
                    ConnectBlockError::FutureBlock(block) => {
                        Err(ConnectBlockError::FutureBlock(block).into())
                    }
                    e => {
                        self.chain.get_storage().save_failed_block(
                            block.id(),
                            block,
                            peer_id.clone(),
                            error_msg,
                            G_CRATE_VERSION.to_string(),
                        )?;
                        if let Some(peer) = peer_id {
                            self.peer_provider.report_peer(peer, e.reputation());
                        }

                        Err(e.into())
                    }
                },
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }

    fn broadcast_dag_chain_block(
        &mut self,
        broadcast_blocks: Vec<(Block, BlockInfo, BlockConnectAction)>,
        start_index: u64,
    ) -> Result<CollectorState> {
        let state = if self.last_accumulator_root == self.target_accumulator_root {
            CollectorState::Enough
        } else {
            CollectorState::Need
        };

        let service = self
            .flexidag_service
            .as_ref()
            .expect("flexidag service is None");
        self.new_dag_accumulator_info = Some(async_std::task::block_on(
            service.send(ForkDagAccumulator {
                new_blocks: broadcast_blocks
                    .into_iter()
                    .map(|(block, _, _)| block.id())
                    .collect(),
                dag_accumulator_index: start_index,
                block_header_id: self.chain.head_block().block().id(),
            }),
        )??);
        if state == CollectorState::Enough {
            async_std::task::block_on(
                service.send(FinishSync {
                    dag_accumulator_info: self
                        .new_dag_accumulator_info
                        .clone()
                        .expect("dag acc should exists"),
                }),
            )??
        }
        return Ok(state);
    }

    fn broadcast_single_chain_block(
        &mut self,
        block: Block,
        block_info: BlockInfo,
        action: BlockConnectAction,
    ) -> Result<CollectorState> {
        let target = self
            .target
            .as_ref()
            .expect("the process is for single chain");
        let state = if block_info.block_accumulator_info.num_leaves
            == target.block_info.block_accumulator_info.num_leaves
        {
            if block_info != target.block_info {
                Err(TaskError::BreakError(
                    RpcVerifyError::new_with_peers(
                        target.peers.clone(),
                        format!(
                    "Verify target error, expect target: {:?}, collect target block_info:{:?}",
                    target.block_info,
                    block_info
                ),
                    )
                    .into(),
                )
                .into())
            } else {
                Ok(CollectorState::Enough)
            }
        } else {
            Ok(CollectorState::Need)
        };

        self.notify_connected_block(block, block_info, action, state?)
    }

    fn collect_dag_item(&mut self, item: SyncBlockData) -> Result<()> {
        let (block, block_info, peer_id) = item.into();
        let block_id = block.id();
        let timestamp = block.header().timestamp();

        let add_dag_result = self
            .flexidag_service
            .as_ref()
            .map(|service| {
                async_std::task::block_on(service.send(AddToDag {
                    block_header: block.header().clone(),
                }))?
            })
            .ok_or_else(|| anyhow!("flexidag service is None"))??;
        let selected_parent = self
            .storage
            .get_block_by_hash(add_dag_result.selected_parent)?
            .expect("selected parent should in storage");
        let mut chain = self.chain.fork(selected_parent.header.parent_hash())?;
        for blue_hash in add_dag_result.mergeset_blues.iter() {
            if let Some(blue_block) = self.storage.get_block(blue_hash.to_owned())? {
                match chain.apply(blue_block) {
                    std::result::Result::Ok(_executed_block) => (),
                    Err(e) => warn!("failed to connect dag block: {:?}", e),
                }
            } else {
                error!("Failed to get block {:?}", blue_hash);
            }
        }

        if chain.status().info().total_difficulty > self.chain.status().info().total_difficulty {
            self.chain = chain;
        }

        Ok(())
    }

    fn collect_item(
        &mut self,
        item: SyncBlockData,
    ) -> Result<(Block, BlockInfo, BlockConnectAction)> {
        let (block, block_info, peer_id) = item.into();
        let block_id = block.id();
        let timestamp = block.header().timestamp();

        return match block_info {
            Some(block_info) => {
                //If block_info exists, it means that this block was already executed and
                // try connect in the previous sync, but the sync task was interrupted.
                //So, we just need to update chain and continue
                self.chain.connect(ExecutedBlock {
                    block: block.clone(),
                    block_info: block_info.clone(),
                })?;
                let block_info = self.chain.status().info;
                Ok((block, block_info, BlockConnectAction::ConnectExecutedBlock))
            }
            None => {
                self.apply_block(block.clone(), peer_id)?;
                self.chain.time_service().adjust(timestamp);
                let block_info = self.chain.status().info;
                Ok((block, block_info, BlockConnectAction::ConnectNewBlock))
            }
        };
    }
}

impl<N, H> TaskResultCollector<SyncBlockData> for BlockCollector<N, H>
where
    N: PeerProvider + 'static,
    H: BlockConnectedEventHandle + 'static,
{
    type Output = BlockChain;

    fn collect(&mut self, item: SyncBlockData) -> Result<CollectorState> {
        let mut process_block_pool = vec![];
        if item.accumulator_root.is_some() {
            // it is a flexidag
            self.dag_block_pool.push(item.clone());
            self.last_accumulator_root = item.accumulator_root.unwrap();

            if item.count_in_leaf != self.dag_block_pool.len() as u64 {
                return Ok(CollectorState::Need);
            } else {
                process_block_pool = std::mem::take(&mut self.dag_block_pool);
            }
        } else {
            // it is a single chain
            process_block_pool.push(item.clone());
        }

        assert!(!process_block_pool.is_empty());

        // let mut next_tips = Some(vec![]);
        let mut block_to_broadcast = vec![];
        if item.accumulator_root.is_some() {
            for item in process_block_pool {
                self.collect_dag_item(item)?
            }
        } else {
            for item in process_block_pool {
                block_to_broadcast.push(self.collect_item(item)?)
            }
        }
        //verify target
        match self.target {
            Some(_) => {
                assert_eq!(
                    block_to_broadcast.len(),
                    1,
                    "in single chain , block_info should exist!"
                );
                let (block, block_info, action) = block_to_broadcast.pop().unwrap();
                // self.check_if_sync_complete(block_info)
                let block_number = block.header().number();
                match self.broadcast_single_chain_block(block, block_info, action) {
                    std::result::Result::Ok(state) => {
                        if self.chain.dag_fork_height() == block_number {
                            Ok(CollectorState::Enough)
                        } else {
                            Ok(state)
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            None => self.broadcast_dag_chain_block(
                block_to_broadcast,
                item.dag_accumulator_index
                    .expect("dag accumulator index is invalid"),
            ),
        }
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self.chain)
    }
}
