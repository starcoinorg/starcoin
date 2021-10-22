use crate::tasks::{
    AccumulatorCollector, BlockAccumulatorSyncTask, BlockCollector, BlockConnectedEventHandle,
    BlockFetcher, BlockIdFetcher, BlockSyncTask, PeerOperator,
};
use anyhow::format_err;
use executor::VMMetrics;
use network_api::PeerProvider;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_chain::BlockChain;
use starcoin_storage::Store;
use starcoin_sync_api::SyncTarget;
use starcoin_types::block::{BlockIdAndNumber, BlockInfo};
use starcoin_types::time::TimeService;
use std::cmp::min;
use std::sync::Arc;
use stream_task::{
    CustomErrorHandle, Generator, TaskError, TaskEventHandle, TaskGenerator, TaskHandle, TaskState,
};

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
        max_retry_times: u64,
        delay_milliseconds_on_error: u64,
        skip_pow_verify_when_sync: bool,
        vm_metrics: Option<VMMetrics>,
    ) -> Result<(BlockChain, TaskHandle), TaskError> {
        let buffer_size = self.target.peers.len();

        let ancestor_block_info = self.ancestor_block_info().map_err(TaskError::BreakError)?;
        let accumulator_sync_task = BlockAccumulatorSyncTask::new(
            // start_number is include, so start from ancestor.number + 1
            self.ancestor.number.saturating_add(1),
            self.target.block_info.block_accumulator_info.clone(),
            self.fetcher.clone(),
            100,
        )
        .map_err(TaskError::BreakError)?;
        let acc_buffer_size = min(
            accumulator_sync_task
                .total_items()
                .expect("total_items must exist") as usize,
            buffer_size,
        );
        let sub_accumulator_task = TaskGenerator::new(
            accumulator_sync_task.clone(),
            acc_buffer_size,
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
            let chain = BlockChain::new(
                self.time_service.clone(),
                ancestor.id,
                self.storage.clone(),
                vm_metrics,
            )?;
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
