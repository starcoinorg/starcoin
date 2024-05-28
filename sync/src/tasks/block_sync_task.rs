// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{BlockConnectedEvent, BlockConnectedEventHandle, BlockFetcher, BlockLocalStore};
use crate::verified_rpc_client::RpcVerifyError;
use anyhow::{anyhow, bail, format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::PeerId;
use network_api::PeerProvider;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::verifier::DagBasicVerifier;
use starcoin_chain::{verifier::BasicVerifier, BlockChain};
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, ExecutedBlock};
use starcoin_config::G_CRATE_VERSION;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::MAX_BLOCK_REQUEST_SIZE;
use starcoin_storage::block::DagSyncBlock;
use starcoin_storage::{Store, BARNARD_HARD_FORK_HASH};
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{
    Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber, DagHeaderType,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use stream_task::{CollectorState, TaskError, TaskResultCollector, TaskState};

use super::{BlockConnectAction, BlockConnectedFinishEvent};

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
    ) -> Self {
        Self {
            current_block_info,
            target,
            chain,
            event_handle,
            peer_provider,
            skip_pow_verify,
            local_store,
            fetcher,
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
                    Ok(_) => {
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
        let parents = block_header.parents_hash().unwrap_or_default();
        if parents.is_empty() {
            return Ok(());
        }
        for parent in parents {
            if !self.chain.has_dag_block(parent)? {
                if absent_blocks.contains(&parent) {
                    continue;
                }
                absent_blocks.push(parent)
            }
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

    async fn find_absent_ancestor(
        &self,
        mut block_headers: Vec<BlockHeader>,
    ) -> Result<Vec<Block>> {
        let mut absent_blocks_map = HashMap::new();
        loop {
            let mut absent_blocks = vec![];
            self.find_absent_parent_dag_blocks_for_blocks(block_headers, &mut absent_blocks)?;
            if absent_blocks.is_empty() {
                return Ok(absent_blocks_map.into_values().collect());
            }
            let remote_absent_blocks = self.fetch_blocks(absent_blocks).await?;
            block_headers = remote_absent_blocks
                .iter()
                .map(|block| block.header().clone())
                .collect();

            remote_absent_blocks.into_iter().for_each(|block| {
                absent_blocks_map.insert(block.id(), block);
            });
        }
    }

    pub fn ensure_dag_parent_blocks_exist(&mut self, block_header: BlockHeader) -> Result<()> {
        if self.chain.check_dag_type(&block_header)? != DagHeaderType::Normal {
            info!(
                "the block is not a dag block, skipping, its id: {:?}, its number {:?}",
                block_header.id(),
                block_header.number()
            );
            return Ok(());
        }
        if self.chain.has_dag_block(block_header.id())? {
            info!(
                "the dag block exists, skipping, its id: {:?}, its number {:?}",
                block_header.id(),
                block_header.number()
            );
            return Ok(());
        }
        info!(
            "the block is a dag block, its id: {:?}, number: {:?}, its parents: {:?}",
            block_header.id(),
            block_header.number(),
            block_header.parents_hash()
        );
        let fut = async {
            let mut absent_ancestor = self
                .find_absent_ancestor(vec![block_header.clone()])
                .await?;

            if absent_ancestor.is_empty() {
                return Ok(());
            }

            absent_ancestor.sort_by_key(|a| a.header().number());
            info!(
                "now apply absent ancestors count: {:?}",
                absent_ancestor.len()
            );

            let mut process_dag_ancestors = HashMap::new();
            loop {
                for ancestor_block_headers in absent_ancestor.chunks(20) {
                    let mut blocks = ancestor_block_headers.to_vec();
                    blocks.retain(|block| {
                        match self.chain.has_dag_block(block.header().id()) {
                            Ok(has) => {
                                if has {
                                    info!("{:?} was already applied", block.header().id());
                                    process_dag_ancestors
                                        .insert(block.header().id(), block.header().clone());
                                    false // remove the executed block
                                } else {
                                    true // retain the un-executed block
                                }
                            }
                            Err(_) => true, // retain the un-executed block
                        }
                    });
                    for block in blocks {
                        info!(
                            "now apply for sync after fetching a dag block: {:?}, number: {:?}",
                            block.id(),
                            block.header().number()
                        );

                        if !self.check_parents_exist(block.header())? {
                            info!(
                                "block: {:?}, number: {:?}, its parent still dose not exist, waiting for next round",
                                block.header().id(),
                                block.header().number()
                            );
                            process_dag_ancestors
                                .insert(block.header().id(), block.header().clone());
                            continue;
                        } else {
                            let executed_block = self
                                .chain
                                .apply_with_verifier::<DagBasicVerifier>(block.clone())?;
                            info!(
                                "succeed to apply a dag block: {:?}, number: {:?}",
                                executed_block.block.id(),
                                executed_block.block.header().number()
                            );
                            process_dag_ancestors
                                .insert(block.header().id(), block.header().clone());

                            self.execute_if_parent_ready(executed_block.block.id())?;

                            self.local_store
                                .delete_dag_sync_block(executed_block.block.id())?;

                            self.notify_connected_block(
                                executed_block.block,
                                executed_block.block_info.clone(),
                                BlockConnectAction::ConnectNewBlock,
                                self.check_enough_by_info(executed_block.block_info)?,
                            )?;
                        }
                    }
                }

                if process_dag_ancestors.is_empty() {
                    bail!("no absent ancestor block is executed!, absent ancestor block: {:?}, their child block id: {:?}, number: {:?}", absent_ancestor, block_header.id(), block_header.number());
                } else {
                    absent_ancestor
                        .retain(|header| !process_dag_ancestors.contains_key(&header.id()));
                }

                if absent_ancestor.is_empty() {
                    break;
                }
            }

            Ok(())
        };
        async_std::task::block_on(fut)
    }

    async fn fetch_blocks(&self, mut block_ids: Vec<HashValue>) -> Result<Vec<Block>> {
        let mut result = vec![];
        block_ids.retain(|id| {
            match self.local_store.get_dag_sync_block(*id) {
                Ok(op_dag_sync_block) => {
                    if let Some(dag_sync_block) = op_dag_sync_block {
                        result.push(dag_sync_block.block);
                        false // read from local store, remove from p2p request
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
                self.local_store.save_dag_sync_block(DagSyncBlock {
                    block: block.clone(),
                    children: vec![],
                })?;
                result.push(block);
            }
        }
        Ok(result)
    }

    fn execute_if_parent_ready(&mut self, parent_id: HashValue) -> Result<()> {
        let mut parent_block =
            self.local_store
                .get_dag_sync_block(parent_id)?
                .ok_or_else(|| {
                    anyhow!(
                        "the dag block should exist in local store, parent child block id: {:?}",
                        parent_id,
                    )
                })?;

        let mut executed_children = vec![];
        for child in &parent_block.children {
            let child_block = self
                .local_store
                .get_dag_sync_block(*child)?
                .ok_or_else(|| {
                    anyhow!(
                        "the dag block should exist in local store, child block id: {:?}",
                        child
                    )
                })?;
            if child_block
                .block
                .header()
                .parents_hash()
                .ok_or_else(|| anyhow!("the dag block's parents should exist"))?
                .iter()
                .all(|parent| match self.chain.has_dag_block(*parent) {
                    Ok(has) => has,
                    Err(e) => {
                        error!(
                            "failed to get the block from the chain, block id: {:?}, error: {:?}",
                            *parent, e
                        );
                        false
                    }
                })
            {
                let executed_block = self
                    .chain
                    .apply_with_verifier::<DagBasicVerifier>(child_block.block.clone())?;
                info!(
                    "succeed to apply a dag block: {:?}, number: {:?}",
                    executed_block.block.id(),
                    executed_block.block.header().number()
                );
                executed_children.push(*child);
                self.notify_connected_block(
                    executed_block.block,
                    executed_block.block_info.clone(),
                    BlockConnectAction::ConnectNewBlock,
                    self.check_enough_by_info(executed_block.block_info)?,
                )?;
                self.execute_if_parent_ready(*child)?;
                self.local_store.delete_dag_sync_block(*child)?;
            }
        }
        parent_block
            .children
            .retain(|child| !executed_children.contains(child));
        self.local_store.save_dag_sync_block(parent_block)?;
        Ok(())
    }

    fn check_parents_exist(&self, block_header: &BlockHeader) -> Result<bool> {
        let mut result = Ok(true);
        for parent in block_header.parents_hash().ok_or_else(|| {
            anyhow!(
                "the dag block's parents should exist, block id: {:?}, number: {:?}",
                block_header.id(),
                block_header.number()
            )
        })? {
            if !self.chain.has_dag_block(parent)? {
                info!("block: {:?}, number: {:?}, its parent({:?}) still dose not exist, waiting for next round", block_header.id(), block_header.number(), parent);
                let mut parent_block = self.local_store.get_dag_sync_block(parent)?.ok_or_else(|| {
                    anyhow!(
                        "the dag block should exist in local store, parent block id: {:?}, number: {:?}",
                        block_header.id(),
                        block_header.number()
                    )
                })?;
                parent_block.children.push(block_header.id());
                self.local_store.save_dag_sync_block(parent_block)?;
                result = Ok(false);
            }
        }
        result
    }

    pub fn check_enough_by_info(&self, block_info: BlockInfo) -> Result<CollectorState> {
        if block_info.block_accumulator_info.num_leaves
            == self.target.block_info.block_accumulator_info.num_leaves
        {
            if block_info != self.target.block_info {
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
        self.ensure_dag_parent_blocks_exist(block.header().clone())?;
        let state = self.check_enough();
        if let anyhow::Result::Ok(CollectorState::Enough) = &state {
            let current_header = self.chain.current_header();
            let current_block = self
                .local_store
                .get_block(current_header.id())?
                .expect("failed to get the current block which should exist");
            return self.notify_connected_block(
                current_block,
                self.local_store
                    .get_block_info(current_header.id())?
                    .expect("block info should exist"),
                BlockConnectAction::ConnectExecutedBlock,
                state?,
            );
        }
        info!("successfully ensure block's parents exist");

        let timestamp = block.header().timestamp();

        let block_info = if self.chain.check_dag_type(block.header())? == DagHeaderType::Normal {
            if self.chain.has_dag_block(block.header().id())? {
                block_info
            } else {
                None
            }
        } else {
            block_info
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

        //verify target
        let state: Result<CollectorState, anyhow::Error> =
            self.check_enough_by_info(block_info.clone());

        self.notify_connected_block(block, block_info, action, state?)
    }

    fn finish(self) -> Result<Self::Output> {
        self.local_store.delete_all_dag_sync_blocks()?;
        Ok(self.chain)
    }
}
