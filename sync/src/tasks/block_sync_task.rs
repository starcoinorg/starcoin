// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::parallel::sender::DagBlockSender;
use crate::store::sync_absent_ancestor::DagSyncBlock;
use crate::store::sync_dag_store::SyncDagStore;
use crate::tasks::continue_execute_absent_block::ContinueExecuteAbsentBlock;
use crate::tasks::{BlockConnectedEvent, BlockConnectedEventHandle, BlockFetcher, BlockLocalStore};
use crate::verified_rpc_client::RpcVerifyError;
use anyhow::{format_err, Context, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::PeerId;
use network_api::PeerProvider;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::{verifier::BasicVerifier, BlockChain};
use starcoin_chain_api::{ChainReader, ChainType, ChainWriter, ConnectBlockError, ExecutedBlock};
use starcoin_config::G_CRATE_VERSION;
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schema::ValueCodec;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::MAX_BLOCK_REQUEST_SIZE;
use starcoin_storage::db_storage::SchemaIterator;
use starcoin_storage::Store;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber};
use std::collections::HashMap;
use std::sync::Arc;
use stream_task::{CollectorState, TaskError, TaskResultCollector, TaskState};

use super::continue_execute_absent_block::ContinueChainOperator;
use super::{BlockConnectAction, BlockConnectedFinishEvent};

const ASYNC_BLOCK_COUNT: u64 = 1000;

enum ParallelSign {
    NeedMoreBlocks,
    Continue,
}

#[derive(Clone, Debug)]
pub struct SyncBlockData {
    pub(crate) block: Block,
    pub(crate) info: Option<BlockInfo>,
    pub(crate) peer_id: Option<PeerId>,
}

impl SyncBlockData {
    pub fn new(block: Block, block_info: Option<BlockInfo>, peer_id: Option<PeerId>) -> Self {
        Self {
            block,
            info: block_info,
            peer_id,
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
            info!(
                "[sync] fetch block ids from accumulator, start_number: {}, ids: {}",
                self.start_number,
                block_ids.len()
            );
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
                            result_map.insert(block.id(), SyncBlockData::new(block, None, peer_id));
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
                    .map(|(block, peer_id)| SyncBlockData::new(block, None, peer_id))
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
    target: SyncTarget,
    // the block chain init by ancestor
    chain: BlockChain,
    event_handle: H,
    peer_provider: N,
    skip_pow_verify: bool,
    local_store: Arc<dyn Store>,
    fetcher: Arc<dyn BlockFetcher>,
    latest_block_id: HashValue,
    sync_dag_store: Arc<SyncDagStore>,
}

impl<N, H> ContinueChainOperator for BlockCollector<N, H>
where
    N: PeerProvider + 'static,
    H: BlockConnectedEventHandle + 'static,
{
    fn has_dag_block(&self, block_id: HashValue) -> anyhow::Result<bool> {
        self.chain
            .has_dag_block(block_id)
            .context("Failed to check if DAG block exists")
    }

    fn apply(&mut self, block: Block) -> anyhow::Result<ExecutedBlock> {
        self.chain.apply(block)
    }

    fn notify(&mut self, executed_block: ExecutedBlock) -> anyhow::Result<CollectorState> {
        let block = executed_block.block;
        let block_id = block.id();
        let block_info = self
            .local_store
            .get_block_info(block_id)?
            .ok_or_else(|| format_err!("block info should exist, id: {:?}", block_id))?;
        self.notify_connected_block(
            block,
            block_info.clone(),
            BlockConnectAction::ConnectNewBlock,
            self.check_enough_by_info(block_info)?,
        )
        .context("Failed to notify connected block")
    }
}

impl<N, H> BlockCollector<N, H>
where
    N: PeerProvider + 'static,
    H: BlockConnectedEventHandle + 'static,
{
    pub fn new_with_handle(
        current_block_info: BlockInfo,
        target: SyncTarget,
        chain: BlockChain,
        event_handle: H,
        peer_provider: N,
        skip_pow_verify: bool,
        local_store: Arc<dyn Store>,
        fetcher: Arc<dyn BlockFetcher>,
        sync_dag_store: Arc<SyncDagStore>,
    ) -> Self {
        let latest_block_id = chain.current_header().id();
        Self {
            current_block_info,
            target,
            chain,
            event_handle,
            peer_provider,
            skip_pow_verify,
            local_store,
            fetcher,
            latest_block_id,
            sync_dag_store,
        }
    }

    #[cfg(test)]
    pub fn apply_block_for_test(&mut self, block: Block) -> Result<()> {
        self.apply_block(block, None)
    }

    fn notify_connected_block(
        &mut self,
        block: Block,
        block_info: BlockInfo,
        action: BlockConnectAction,
        state: CollectorState,
    ) -> Result<CollectorState> {
        info!("sync processs complete a block execution: number: {}, hash value {}, current main: number: {}, header hash value: {}", 
        block.header().number(), block.header().id(), self.chain.current_header().number(), self.chain.current_header().id());
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
        let (sender, _) = match state {
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
        // if block_connect_event.feedback.is_some() && receiver.is_some() {
        //     let mut count: i32 = 0;
        //     while count < 3 {
        //         count = count.saturating_add(1);
        //         match receiver.as_mut().unwrap().try_next() {
        //             Ok(_) => {
        //                 break;
        //             }
        //             Err(_) => {
        //                 info!("Waiting for last block to be processed");
        //                 async_std::task::block_on(async_std::task::sleep(Duration::from_secs(10)));
        //             }
        //         }
        //     }
        // }
        Ok(state)
    }

    fn apply_block(&mut self, block: Block, peer_id: Option<PeerId>) -> Result<()> {
        let apply_result = if self.skip_pow_verify {
            self.chain
                .apply_with_verifier::<BasicVerifier>(block.clone())
        } else {
            self.chain.apply_for_sync(block.clone())
        };
        if let Err(err) = apply_result {
            let error_msg = err.to_string();
            error!(
                "[sync] collect block error: {:?}, peer_id:{:?} ",
                error_msg, peer_id
            );
            match err.downcast::<ConnectBlockError>() {
                Ok(connect_error) => match connect_error {
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

    fn find_absent_parent_dag_blocks(
        &self,
        block_header: BlockHeader,
        absent_blocks: &mut Vec<HashValue>,
    ) -> Result<()> {
        let parents = block_header.parents_hash();
        if parents.is_empty() {
            return Ok(());
        }
        for parent in parents {
            if absent_blocks.contains(&parent) {
                continue;
            }
            if self.chain.has_dag_block(parent)? {
                continue;
            }
            absent_blocks.push(parent);
        }
        Ok(())
    }

    fn find_absent_parent_dag_blocks_for_blocks(
        &self,
        block_headers: Vec<BlockHeader>,
        absent_blocks: &mut Vec<HashValue>,
    ) -> Result<()> {
        for block_header in block_headers {
            self.find_absent_parent_dag_blocks(block_header, absent_blocks)?;
        }
        Ok(())
    }

    async fn find_absent_ancestor(&self, mut block_headers: Vec<BlockHeader>) -> Result<()> {
        loop {
            let mut absent_blocks = vec![];
            self.find_absent_parent_dag_blocks_for_blocks(block_headers, &mut absent_blocks)?;
            if absent_blocks.is_empty() {
                return Ok(());
            }
            block_headers = self.fetch_blocks(absent_blocks).await?;
        }
    }

    fn ensure_dag_parent_blocks_exist(&mut self, block: Block) -> Result<ParallelSign> {
        let block_header = &block.header().clone();
        if self.chain.has_dag_block(block_header.id())? {
            info!(
                "the dag block exists, skipping, its id: {:?}, its number {:?}",
                block_header.id(),
                block_header.number()
            );
            return Ok(ParallelSign::Continue);
        }
        info!(
            "the block is a dag block, its id: {:?}, number: {:?}, its parents: {:?}",
            block_header.id(),
            block_header.number(),
            block_header.parents_hash()
        );
        let fut = async {
            if block_header.number() % ASYNC_BLOCK_COUNT == 0
                || block_header.number() >= self.target.target_id.number()
            {
                self.sync_dag_store.delete_all_dag_sync_block()?;
                self.find_absent_ancestor(vec![block_header.clone()])
                    .await?;

                let parallel_execute = DagBlockSender::new(
                    self.sync_dag_store.clone(),
                    100000,
                    self.chain.time_service(),
                    self.local_store.clone(),
                    None,
                    self.chain.dag(),
                    self,
                );
                parallel_execute.process_absent_blocks().await?;
                anyhow::Ok(ParallelSign::Continue)
            } else {
                self.local_store
                    .save_dag_sync_block(starcoin_storage::block::DagSyncBlock {
                        block: block.clone(),
                        children: vec![],
                    })?;
                anyhow::Ok(ParallelSign::NeedMoreBlocks)
            }
        };
        async_std::task::block_on(fut)
    }

    pub fn read_local_absent_block(
        &self,
        iter: &mut SchemaIterator<Vec<u8>, Vec<u8>>,
        absent_ancestor: &mut Vec<Block>,
    ) -> anyhow::Result<()> {
        let results = iter
            .take(720)
            .map(|result_block| match result_block {
                anyhow::Result::Ok((_, data_raw)) => {
                    let dag_sync_block = DagSyncBlock::decode_value(&data_raw)?;
                    Ok(dag_sync_block.block.ok_or_else(|| {
                        format_err!("block in sync dag block should not be none!")
                    })?)
                }
                Err(e) => Err(e),
            })
            .collect::<Vec<_>>();
        for result_block in results {
            match result_block {
                anyhow::Result::Ok(block) => absent_ancestor.push(block),
                Err(e) => return Err(e),
            }
        }
        anyhow::Result::Ok(())
    }

    async fn fetch_blocks(&self, mut block_ids: Vec<HashValue>) -> Result<Vec<BlockHeader>> {
        let mut result = vec![];
        block_ids.retain(|id| {
            match self.local_store.get_dag_sync_block(*id) {
                Ok(op_dag_sync_block) => {
                    if let Some(dag_sync_block) = op_dag_sync_block {
                        match self.sync_dag_store.save_block(dag_sync_block.block.clone()) {
                            Ok(_) => {
                                result.push(dag_sync_block.block.header().clone());
                                false // read from local store, remove from p2p request
                            }
                            Err(e) => {
                                debug!("failed to save block for: {:?}", e);
                                true // need retaining
                            }
                        }
                    } else {
                        true // need retaining
                    }
                }
                Err(_) => true, // need retaining
            }
        });
        for chunk in block_ids.chunks(usize::try_from(MAX_BLOCK_REQUEST_SIZE)?) {
            let remote_dag_sync_blocks = self.fetcher.fetch_blocks(chunk.to_vec()).await?;
            for (block, _) in remote_dag_sync_blocks {
                self.local_store
                    .save_dag_sync_block(starcoin_storage::block::DagSyncBlock {
                        block: block.clone(),
                        children: vec![],
                    })?;
                self.sync_dag_store.save_block(block.clone())?;
                result.push(block.header().clone());
            }
        }
        Ok(result)
    }

    #[allow(dead_code)]
    async fn fetch_blocks_in_batch(
        &self,
        mut block_ids: Vec<HashValue>,
    ) -> Result<Vec<BlockHeader>> {
        let mut result = vec![];
        block_ids.retain(|id| {
            match self.local_store.get_dag_sync_block(*id) {
                Ok(op_dag_sync_block) => {
                    if let Some(dag_sync_block) = op_dag_sync_block {
                        match self.sync_dag_store.save_block(dag_sync_block.block.clone()) {
                            Ok(_) => {
                                result.push(dag_sync_block.block.header().clone());
                                false // read from local store, remove from p2p request
                            }
                            Err(e) => {
                                debug!("failed to save block for: {:?}", e);
                                true // need retaining
                            }
                        }
                    } else {
                        true // need retaining
                    }
                }
                Err(_) => true, // need retaining
            }
        });

        let mut exp: u64 = 1;
        for chunk in block_ids.chunks(usize::try_from(MAX_BLOCK_REQUEST_SIZE)?) {
            let filtered_chunk = chunk
                .iter()
                .filter(|id| {
                    self.local_store
                        .get_dag_sync_block(**id)
                        .unwrap_or(None)
                        .is_none()
                        || self.chain.has_dag_block(**id).unwrap_or(true)
                })
                .cloned()
                .collect::<Vec<_>>();
            if filtered_chunk.is_empty() {
                continue;
            }
            let remote_dag_sync_blocks = self
                .fetcher
                .fetch_dag_block_in_batch(filtered_chunk.to_vec(), exp)
                .await?;
            for (block, _) in remote_dag_sync_blocks {
                self.local_store
                    .save_dag_sync_block(starcoin_storage::block::DagSyncBlock {
                        block: block.clone(),
                        children: vec![],
                    })?;
                self.sync_dag_store.save_block(block.clone())?;
                result.push(block.header().clone());
            }
            if exp < 16 {
                exp = exp.saturating_mul(2);
            }
        }

        Ok(result
            .into_iter()
            .filter(|block_header| {
                !block_header.parents_hash().iter().all(|parent_id| {
                    self.local_store
                        .get_dag_sync_block(*parent_id)
                        .map(|opt_block| opt_block.is_none())
                        .unwrap_or(true)
                        || self.chain.has_dag_block(*parent_id).unwrap_or(false)
                })
            })
            .collect())
    }

    #[allow(dead_code)]
    async fn fetch_blocks_in_batch(
        &self,
        mut block_ids: Vec<HashValue>,
    ) -> Result<Vec<BlockHeader>> {
        let mut result = vec![];
        block_ids.retain(|id| {
            match self.local_store.get_dag_sync_block(*id) {
                Ok(op_dag_sync_block) => {
                    if let Some(dag_sync_block) = op_dag_sync_block {
                        match self.sync_dag_store.save_block(dag_sync_block.block.clone()) {
                            Ok(_) => {
                                result.push(dag_sync_block.block.header().clone());
                                false // read from local store, remove from p2p request
                            }
                            Err(e) => {
                                debug!("failed to save block for: {:?}", e);
                                true // need retaining
                            }
                        }
                    } else {
                        true // need retaining
                    }
                }
                Err(_) => true, // need retaining
            }
        });

        let mut exp: u64 = 1;
        for chunk in block_ids.chunks(usize::try_from(MAX_BLOCK_REQUEST_SIZE)?) {
            let filtered_chunk = chunk
                .iter()
                .filter(|id| {
                    self.local_store
                        .get_dag_sync_block(**id)
                        .unwrap_or(None)
                        .is_none()
                        || self.chain.has_dag_block(**id).unwrap_or(true)
                })
                .cloned()
                .collect::<Vec<_>>();
            if filtered_chunk.is_empty() {
                continue;
            }
            let remote_dag_sync_blocks = self
                .fetcher
                .fetch_dag_block_in_batch(filtered_chunk.to_vec(), exp)
                .await?;
            for block in remote_dag_sync_blocks {
                self.local_store
                    .save_dag_sync_block(starcoin_storage::block::DagSyncBlock {
                        block: block.clone(),
                        children: vec![],
                    })?;
                self.sync_dag_store.save_block(block.clone())?;
                result.push(block.header().clone());
            }
            if exp < 16 {
                exp = exp.saturating_mul(2);
            }
        }

        Ok(result
            .into_iter()
            .filter(|block_header| {
                !block_header.parents_hash().iter().all(|parent_id| {
                    self.local_store
                        .get_dag_sync_block(*parent_id)
                        .map(|opt_block| opt_block.is_none())
                        .unwrap_or(true)
                        || self.chain.has_dag_block(*parent_id).unwrap_or(false)
                })
            })
            .collect())
    }

    pub fn check_enough_by_info(&self, block_info: BlockInfo) -> Result<CollectorState> {
        if block_info.block_accumulator_info.num_leaves
            == self.target.block_info.block_accumulator_info.num_leaves
        {
            if self.chain.check_chain_type()? == ChainType::Dag {
                Ok(CollectorState::Enough)
            } else if block_info != self.target.block_info {
                Err(TaskError::BreakError(
                    RpcVerifyError::new_with_peers(
                        self.target.peers.clone(),
                        format!(
                    "Verify target error, expect target: {:?}, collect target block_info:{:?}",
                    self.target.block_info,
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
        }
    }

    pub fn check_enough(&self) -> Result<CollectorState> {
        if let Some(block_info) = self
            .local_store
            .get_block_info(self.chain.current_header().id())?
        {
            self.check_enough_by_info(block_info)
        } else {
            Ok(CollectorState::Need)
        }
    }

    pub fn execute_absent_block(&mut self, absent_ancestor: &mut Vec<Block>) -> Result<()> {
        let local_store = self.local_store.clone();
        let sync_dag_store = self.sync_dag_store.clone();
        let mut execute_absent_block =
            ContinueExecuteAbsentBlock::new(self, local_store, sync_dag_store)?;
        execute_absent_block.execute_absent_blocks(absent_ancestor)
    }
}

impl<N, H> TaskResultCollector<SyncBlockData> for BlockCollector<N, H>
where
    N: PeerProvider + 'static,
    H: BlockConnectedEventHandle + 'static,
{
    type Output = BlockChain;

    fn collect(&mut self, item: SyncBlockData) -> Result<CollectorState> {
        let (block, block_info, peer_id) = item.into();

        // if it is a dag block, we must ensure that its dag parent blocks exist.
        // if it is not, we must pull the dag parent blocks from the peer.
        info!("now sync dag block -- ensure_dag_parent_blocks_exist");
        match self.ensure_dag_parent_blocks_exist(block.clone())? {
            ParallelSign::NeedMoreBlocks => return Ok(CollectorState::Need),
            ParallelSign::Continue => (),
        }
        let state = self.check_enough();
        if let anyhow::Result::Ok(CollectorState::Enough) = &state {
            if self.chain.has_dag_block(block.header().id())? {
                let current_header = self.chain.current_header();
                let current_block = self
                    .local_store
                    .get_block(current_header.id())?
                    .expect("failed to get the current block which should exist");
                self.latest_block_id = block.header().id();
                return self.notify_connected_block(
                    current_block,
                    self.local_store
                        .get_block_info(current_header.id())?
                        .expect("block info should exist"),
                    BlockConnectAction::ConnectExecutedBlock,
                    state?,
                );
            }
        }
        info!("successfully ensure block's parents exist");

        let timestamp = block.header().timestamp();

        let block_info = if self.chain.has_dag_block(block.header().id())? {
            block_info
        } else {
            None
        };

        let (block_info, action) = match block_info {
            Some(block_info) => {
                self.chain.connect(ExecutedBlock {
                    block: block.clone(),
                    block_info: block_info.clone(),
                })?;
                (block_info, BlockConnectAction::ConnectExecutedBlock)
            }
            None => {
                self.apply_block(block.clone(), peer_id)?;
                self.chain.time_service().adjust(timestamp);
                (
                    self.chain.status().info,
                    BlockConnectAction::ConnectNewBlock,
                )
            }
        };
        self.latest_block_id = block.header().id();

        //verify target
        let state: Result<CollectorState, anyhow::Error> =
            self.check_enough_by_info(block_info.clone());

        self.notify_connected_block(block, block_info, action, state?)
    }

    fn finish(self) -> Result<Self::Output> {
        self.local_store.delete_all_dag_sync_blocks()?;
        self.chain.fork(self.latest_block_id)
    }
}
