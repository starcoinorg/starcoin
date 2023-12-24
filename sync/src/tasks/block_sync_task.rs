// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::{BlockConnectedEvent, BlockConnectedEventHandle, BlockFetcher, BlockLocalStore};
use crate::verified_rpc_client::RpcVerifyError;
use anyhow::{bail, format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::PeerId;
use network_api::PeerProvider;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::{verifier::BasicVerifier, BlockChain};
use starcoin_chain_api::{ChainReader, ChainWriter, ConnectBlockError, ExecutedBlock};
use starcoin_config::G_CRATE_VERSION;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_storage::{Store, BARNARD_HARD_FORK_HASH};
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{Block, BlockHeader, BlockIdAndNumber, BlockInfo, BlockNumber};
use starcoin_vm_types::account_address::HashAccountAddress;
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
        ancestors: &mut Vec<HashValue>,
        absent_blocks: &mut Vec<HashValue>,
    ) -> Result<()> {
        let parents = block_header.parents_hash().unwrap_or_default();
        if parents.is_empty() {
            return Ok(());
        }
        for parent in parents {
            if !self.chain.has_dag_block(parent)? {
                absent_blocks.push(parent)
            } else {
                ancestors.push(parent);
            }
        }
        Ok(())
    }

    fn find_absent_parent_dag_blocks_for_blocks(
        &self,
        block_headers: Vec<BlockHeader>,
        ancestors: &mut Vec<HashValue>,
        absent_blocks: &mut Vec<HashValue>,
    ) -> Result<()> {
        for block_header in block_headers {
            self.find_absent_parent_dag_blocks(block_header, ancestors, absent_blocks)?;
        }
        Ok(())
    }

    async fn find_ancestor_dag_block_header(
        &self,
        mut block_headers: Vec<BlockHeader>,
        peer_id: PeerId,
    ) -> Result<Vec<HashValue>> {
        let mut ancestors = vec![];
        loop {
            let mut absent_blocks = vec![];
            self.find_absent_parent_dag_blocks_for_blocks(
                block_headers,
                &mut ancestors,
                &mut absent_blocks,
            )?;
            if absent_blocks.is_empty() {
                return Ok(ancestors);
            }
            let absent_block_headers = self
                .fetcher
                .fetch_block_headers(absent_blocks, peer_id.clone())
                .await?;
            if absent_block_headers.iter().any(|(id, header)| {
                if header.is_none() {
                    error!(
                        "fetch absent block header failed, block id: {:?}, peer_id: {:?}, it should not be absent!",
                        id, peer_id
                    );
                    return true;
                }
                false
            }) {
                bail!("fetch absent block header failed, it should not be absent!");
            }
            block_headers = absent_block_headers
                .into_iter()
                .map(|(_, header)| header.expect("block header should not be none!"))
                .collect();
        }
    }

    pub fn ensure_dag_parent_blocks_exist(
        &mut self,
        block_header: &BlockHeader,
        peer_id: Option<PeerId>,
    ) -> Result<()> {
        if !block_header.is_dag() {
            return Ok(());
        }
        let peer_id = peer_id.ok_or_else(|| format_err!("peer_id should not be none!"))?;
        let fut = async {
            let dag_ancestors = self
                .find_ancestor_dag_block_header(vec![block_header.clone()], peer_id.clone())
                .await?;

            let mut dag_ancestors = dag_ancestors
                .into_iter()
                .map(|header| header)
                .collect::<Vec<_>>();

            while !dag_ancestors.is_empty() {
                for ancestor_block_header_id in &dag_ancestors {
                    if block_header.id() == *ancestor_block_header_id {
                        continue;
                    }

                    match self
                        .local_store
                        .get_block_by_hash(ancestor_block_header_id.clone())?
                    {
                        Some(block) => {
                            self.chain.apply(block)?;
                        }
                        None => {
                            for block in self
                                .fetcher
                                .fetch_blocks_by_peerid(
                                    vec![ancestor_block_header_id.clone()],
                                    peer_id.clone(),
                                )
                                .await?
                            {
                                match block {
                                    Some(block) => {
                                        let _ = self.chain.apply(block)?;
                                    }
                                    None => bail!(
                                        "fetch ancestor block failed, block id: {:?}, peer_id: {:?}",
                                        ancestor_block_header_id,
                                        peer_id
                                    ),
                                }
                            }
                        }
                    }
                }
                dag_ancestors = self
                    .fetcher
                    .fetch_dag_block_children(dag_ancestors, peer_id)
                    .await?;
            }

            Ok(())
        };
        async_std::task::block_on(fut)
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
        self.ensure_dag_parent_blocks_exist(block.header(), peer_id.clone())?;
        ////////////

        let timestamp = block.header().timestamp();
        let (block_info, action) = match block_info {
            Some(block_info) => {
                //If block_info exists, it means that this block was already executed and try connect in the previous sync, but the sync task was interrupted.
                //So, we just need to update chain and continue
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
            };

        self.notify_connected_block(block, block_info, action, state?)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self.chain)
    }
}
