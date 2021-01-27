use crate::tasks::{
    AccumulatorCollector, BlockAccumulatorSyncTask, BlockCollector, BlockConnectedEventHandle,
    BlockFetcher, BlockIdFetcher, BlockSyncTask, PeerOperator,
};
use anyhow::format_err;
use chain::BlockChain;
use logger::prelude::*;
use network_api::{NetworkService, PeerId};
use rand::seq::IteratorRandom;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::Accumulator;
use starcoin_crypto::HashValue;
use starcoin_storage::Store;
use starcoin_types::block::{BlockIdAndNumber, BlockInfo};
use starcoin_types::{block::BlockNumber, time::TimeService};
use std::sync::Arc;
use stream_task::{Generator, TaskError, TaskEventHandle, TaskGenerator, TaskHandle};

pub struct FindSubTargetTask<F>
where
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
{
    fetcher: F,
    target_number: BlockNumber,
}

impl<F> FindSubTargetTask<F>
where
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
{
    pub fn new(fetcher: F, target_number: BlockNumber) -> Self {
        Self {
            fetcher,
            target_number,
        }
    }

    async fn block_id_from_peer(&self, peer_id: PeerId) -> Option<HashValue> {
        if let Ok(mut ids) = self
            .fetcher
            .fetch_block_ids_from_peer(Some(peer_id), self.target_number, false, 1)
            .await
        {
            return ids.pop();
        }
        None
    }

    pub async fn sub_target(
        self,
    ) -> anyhow::Result<(Vec<PeerId>, Option<(BlockIdAndNumber, AccumulatorInfo)>)> {
        //1. best peer get block id
        let best_peer = self
            .fetcher
            .find_best_peer()
            .ok_or_else(|| format_err!("Best peer is none when create sub target"))?;
        let mut target_peers: Vec<PeerId> = Vec::new();
        target_peers.push(best_peer.peer_id());
        if let Some(target_id) = self.block_id_from_peer(best_peer.peer_id()).await {
            info!(
                "Best peer target id : {}: {:?}",
                self.target_number, target_id
            );
            let mut hashs = Vec::new();
            hashs.push(target_id);
            //2. filter other peers
            for peer_id in self.fetcher.peers() {
                if best_peer.peer_id() != peer_id {
                    if let Some(id) = self.block_id_from_peer(peer_id.clone()).await {
                        if id == target_id {
                            target_peers.push(peer_id);
                        }
                    }
                }
            }

            //3. get AccumulatorInfo
            let peer_id = target_peers
                .iter()
                .choose(&mut rand::thread_rng())
                .cloned()
                .ok_or_else(|| format_err!("Random peer is none when create sub target"))?;
            let info = self
                .fetcher
                .fetch_block_infos_from_peer(Some(peer_id), hashs)
                .await?
                .pop()
                .ok_or_else(|| format_err!("Target BlockInfo is none when create sub target"))?;
            Ok((
                target_peers,
                Some((
                    BlockIdAndNumber::new(target_id, self.target_number),
                    info.block_accumulator_info,
                )),
            ))
        } else {
            Ok((target_peers, None))
        }
    }
}

pub struct InnerSyncTask<H, F, N>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
    N: NetworkService + 'static,
{
    ancestor: BlockIdAndNumber,
    target_block_accumulator: AccumulatorInfo,
    storage: Arc<dyn Store>,
    block_event_handle: H,
    fetcher: Arc<F>,
    event_handle: Arc<dyn TaskEventHandle>,
    time_service: Arc<dyn TimeService>,
    network: N,
}

impl<H, F, N> InnerSyncTask<H, F, N>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    F: BlockIdFetcher + BlockFetcher + PeerOperator + 'static,
    N: NetworkService + 'static,
{
    pub fn new(
        ancestor: BlockIdAndNumber,
        target_block_accumulator: AccumulatorInfo,
        storage: Arc<dyn Store>,
        block_event_handle: H,
        fetcher: Arc<F>,
        event_handle: Arc<dyn TaskEventHandle>,
        time_service: Arc<dyn TimeService>,
        network: N,
    ) -> Self {
        Self {
            ancestor,
            target_block_accumulator,
            storage,
            block_event_handle,
            fetcher,
            event_handle,
            time_service,
            network,
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
            self.ancestor.number + 1,
            self.target_block_accumulator.clone(),
            self.fetcher.clone(),
            100,
        );
        let sub_accumulator_task = TaskGenerator::new(
            accumulator_sync_task,
            buffer_size,
            max_retry_times,
            delay_milliseconds_on_error,
            AccumulatorCollector::new(
                self.storage.get_accumulator_store(AccumulatorStoreType::Block),
                self.ancestor,
                ancestor_block_info.clone().block_accumulator_info,
                self.target_block_accumulator.clone(),
            ),
            self.event_handle.clone(),
        ).and_then(move |(ancestor, accumulator), event_handle| {
            //start_number is include, so start from ancestor.number + 1
            let start_number = ancestor.number + 1;
            let check_local_store = ancestor_block_info.total_difficulty < current_block_info.total_difficulty;
            info!(
                "[sync] Start sync block, ancestor: {:?}, start_number: {}, check_local_store: {:?}, target_number: {}",
                ancestor, start_number, check_local_store, accumulator.num_leaves() -1 );
            let block_sync_task = BlockSyncTask::new(
                accumulator,
                start_number,
                self.fetcher.clone(),
                check_local_store,
                self.storage.clone(),
                1,
            );
            let chain = BlockChain::new(self.time_service.clone(), ancestor.id, self.storage.clone())?;
            let block_collector = BlockCollector::new_with_handle(current_block_info.clone(), chain, self.block_event_handle.clone(), self.network.clone(), skip_pow_verify_when_sync);
            Ok(TaskGenerator::new(
                block_sync_task,
                buffer_size,
                max_retry_times,
                delay_milliseconds_on_error,
                block_collector,
                event_handle,
            ))
        }).generate();

        let (fut, handle) = sub_accumulator_task.with_handle();
        let block_chain = fut.await?;

        Ok((block_chain, handle))
    }
}
