use crate::tasks::{
    AccumulatorCollector, BlockAccumulatorSyncTask, BlockCollector, BlockConnectedEventHandle,
    BlockFetcher, BlockIdFetcher, BlockSyncTask, PeerOperator, SyncFetcher,
};
use anyhow::format_err;
use logger::prelude::*;
use network_api::{PeerId, PeerProvider};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_chain::BlockChain;
use starcoin_storage::Store;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{BlockIdAndNumber, BlockInfo};
use starcoin_types::time::TimeService;
use std::sync::Arc;
use stream_task::{
    CustomErrorHandle, Generator, TaskError, TaskEventHandle, TaskGenerator, TaskHandle,
};

/// Split the full target to sub target
pub async fn sub_target<F>(
    full_target: SyncTarget,
    ancestor: BlockIdAndNumber,
    fetcher: F,
) -> anyhow::Result<SyncTarget>
where
    F: SyncFetcher + 'static,
{
    let target_number = ancestor.number.saturating_add(1000);
    if target_number >= full_target.target_id.number() {
        return Ok(full_target);
    }
    //1. best peer get block id
    let best_peer = fetcher
        .peer_selector()
        .best()
        .ok_or_else(|| format_err!("Best peer is none when create sub target"))?;
    let mut target_peers: Vec<PeerId> = Vec::new();
    target_peers.push(best_peer.peer_id());
    if let Some(target_id) = fetcher
        .fetch_block_id(Some(best_peer.peer_id()), target_number)
        .await?
    {
        info!("Best peer target id : {}: {:?}", target_number, target_id);
        let best_block_info = fetcher
            .fetch_block_info(Some(best_peer.peer_id()), target_id)
            .await?
            .ok_or_else(|| {
                format_err!(
                    "Fetch {} BlockInfo from {:?} return none when create sub target",
                    target_id,
                    best_peer.peer_id()
                )
            })?;
        //2. filter other peers
        for peer_id in fetcher.peer_selector().peers_by_filter(|peer| {
            best_peer.peer_id() != peer.peer_id()
                && peer.chain_info().status().head.number() >= target_number
        }) {
            if let Some(block_info) = fetcher
                .fetch_block_info(Some(peer_id.clone()), target_id)
                .await?
            {
                if best_block_info == block_info {
                    target_peers.push(peer_id);
                } else {
                    warn!("[sync] Block {}'s block info is different at best peer {}({:?}) and peer {}({:?})", target_id, best_peer.peer_id(), best_block_info, peer_id, block_info);
                }
            }
        }

        Ok(SyncTarget {
            target_id: BlockIdAndNumber::new(target_id, target_number),
            block_info: best_block_info,
            peers: target_peers,
        })
    } else {
        Ok(full_target)
    }
}

pub struct InnerSyncTask<H, F, N>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
    N: PeerProvider + Clone + 'static,
{
    ancestor: BlockIdAndNumber,
    target: SyncTarget,
    storage: Arc<dyn Store>,
    block_event_handle: H,
    fetcher: Arc<F>,
    event_handle: Arc<dyn TaskEventHandle>,
    time_service: Arc<dyn TimeService>,
    peer_provider: N,
    custom_error_handle: Arc<dyn CustomErrorHandle>,
}

impl<H, F, N> InnerSyncTask<H, F, N>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
    N: PeerProvider + Clone + 'static,
{
    pub fn new(
        ancestor: BlockIdAndNumber,
        target: SyncTarget,
        storage: Arc<dyn Store>,
        block_event_handle: H,
        fetcher: Arc<F>,
        event_handle: Arc<dyn TaskEventHandle>,
        time_service: Arc<dyn TimeService>,
        peer_provider: N,
        custom_error_handle: Arc<dyn CustomErrorHandle>,
    ) -> Self {
        Self {
            ancestor,
            target,
            storage,
            block_event_handle,
            fetcher,
            event_handle,
            time_service,
            peer_provider,
            custom_error_handle,
        }
    }

    fn ancestor_block_info(&self) -> anyhow::Result<BlockInfo> {
        self.storage
            .get_block_info(self.ancestor.id)?
            .ok_or_else(|| {
                format_err!(
                    "[sync] Can not find ancestor block info by id: {}",
                    self.ancestor.id
                )
            })
    }

    pub async fn do_sync(
        self,
        current_block_info: BlockInfo,
        buffer_size: usize,
        max_retry_times: u64,
        delay_milliseconds_on_error: u64,
        skip_pow_verify_when_sync: bool,
    ) -> Result<(BlockChain, TaskHandle), TaskError> {
        let ancestor_block_info = self.ancestor_block_info().map_err(TaskError::BreakError)?;
        let accumulator_sync_task = BlockAccumulatorSyncTask::new(
            // start_number is include, so start from ancestor.number + 1
            self.ancestor.number.saturating_add(1),
            self.target.block_info.block_accumulator_info.clone(),
            self.fetcher.clone(),
            100,
        )
        .map_err(TaskError::BreakError)?;
        let sub_accumulator_task = TaskGenerator::new(
            accumulator_sync_task,
            buffer_size,
            max_retry_times,
            delay_milliseconds_on_error,
            AccumulatorCollector::new(
                self.storage
                    .get_accumulator_store(AccumulatorStoreType::Block),
                self.ancestor,
                ancestor_block_info.clone().block_accumulator_info,
                self.target.block_info.block_accumulator_info.clone(),
            ),
            self.event_handle.clone(),
            self.custom_error_handle.clone(),
        )
        .and_then(move |(ancestor, accumulator), event_handle| {
            let check_local_store =
                ancestor_block_info.total_difficulty < current_block_info.total_difficulty;

            let block_sync_task = BlockSyncTask::new(
                accumulator,
                ancestor,
                self.fetcher.clone(),
                check_local_store,
                self.storage.clone(),
                1,
            );
            let chain =
                BlockChain::new(self.time_service.clone(), ancestor.id, self.storage.clone())?;
            let block_collector = BlockCollector::new_with_handle(
                current_block_info.clone(),
                self.target.clone(),
                chain,
                self.block_event_handle.clone(),
                self.peer_provider.clone(),
                skip_pow_verify_when_sync,
            );
            Ok(TaskGenerator::new(
                block_sync_task,
                buffer_size,
                max_retry_times,
                delay_milliseconds_on_error,
                block_collector,
                event_handle,
                self.custom_error_handle.clone(),
            ))
        })
        .generate();

        let (fut, handle) = sub_accumulator_task.with_handle();
        let block_chain = fut.await?;

        Ok((block_chain, handle))
    }
}
