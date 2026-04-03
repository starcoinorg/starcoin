use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet},
    sync::Arc,
    vec,
};

use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{info, warn};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::{Store, Store2};
use starcoin_types::block::Block;
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        watch,
    },
    task::JoinHandle,
};

use crate::{
    store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::SyncDagStore},
    sync_profiling_info_enabled,
    tasks::continue_execute_absent_block::ContinueChainOperator,
};

use super::executor::{DagBlockExecutor, ExecuteDurations, ExecuteState, WorkerExecuteEvent};
use super::parallel_info_service::{
    ParallelInfoService, ParallelWorkerId, RegisterWorkerRequest, ReportWorkerSyncedBlockRequest,
    UnregisterWorkerRequest,
};

const SYNC_PROF_PREFIX: &str = "[sync-prof]";

#[derive(Debug, Default)]
struct ParallelStageProfile {
    executed_blocks: u64,
    scheduled_blocks: u64,
    wait_parents_ms: u128,
    execute_ms: u128,
    apply_notify_ms: u128,
    peak_workers: usize,
    peak_pending_blocks: usize,
    no_ready_cycles: u64,
    fallback_dispatch_count: u64,
}

impl ParallelStageProfile {
    fn record_execute(&mut self, durations: ExecuteDurations) {
        self.executed_blocks = self.executed_blocks.saturating_add(1);
        self.wait_parents_ms = self
            .wait_parents_ms
            .saturating_add(durations.wait_parents_ms);
        self.execute_ms = self.execute_ms.saturating_add(durations.execute_ms);
    }

    fn record_apply_notify(&mut self, apply_notify_ms: u128) {
        self.apply_notify_ms = self.apply_notify_ms.saturating_add(apply_notify_ms);
    }

    fn record_scheduled(&mut self, count: usize) {
        self.scheduled_blocks = self.scheduled_blocks.saturating_add(count as u64);
    }

    fn observe_workers(&mut self, workers: usize) {
        self.peak_workers = self.peak_workers.max(workers);
    }

    fn observe_pending(&mut self, pending: usize) {
        self.peak_pending_blocks = self.peak_pending_blocks.max(pending);
    }

    fn record_no_ready_cycle(&mut self) {
        self.no_ready_cycles = self.no_ready_cycles.saturating_add(1);
    }

    fn record_fallback_dispatch(&mut self) {
        self.fallback_dispatch_count = self.fallback_dispatch_count.saturating_add(1);
    }

    fn total_accounted_ms(&self) -> u128 {
        self.wait_parents_ms
            .saturating_add(self.execute_ms)
            .saturating_add(self.apply_notify_ms)
    }

    fn avg_ms(total_ms: u128, count: u64) -> u128 {
        if count == 0 {
            0
        } else {
            total_ms / u128::from(count)
        }
    }
}

struct DagBlockWorker {
    pub worker_id: ParallelWorkerId,
    pub registered: bool,
    pub sender_to_executor: Sender<Option<Block>>,
    pub state: ExecuteState,
    pub handle: JoinHandle<()>,
}

pub struct DagBlockSender<'a> {
    sync_dag_store: Arc<SyncDagStore>,
    executors: Vec<DagBlockWorker>,
    queue_size: usize,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    storage2: Arc<dyn Store2>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
    execute_timeout_ms: u64,
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    parallel_info_service: Option<ServiceRef<ParallelInfoService>>,
    next_worker_id: ParallelWorkerId,
    free_worker_ids: BinaryHeap<Reverse<ParallelWorkerId>>,
    executor_event_tx: Sender<WorkerExecuteEvent>,
    executor_event_rx: Receiver<WorkerExecuteEvent>,
    parent_ready_signal_tx: watch::Sender<u64>,
    parent_ready_signal_seq: u64,
    ready_parent_cache: HashSet<HashValue>,
    profiling_enabled: bool,
    profile: ParallelStageProfile,
    notifier: &'a mut dyn ContinueChainOperator,
}

impl<'a> DagBlockSender<'a> {
    pub fn new(
        sync_dag_store: Arc<SyncDagStore>,
        queue_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
        execute_timeout_ms: u64,
        cancel_flag: Arc<std::sync::atomic::AtomicBool>,
        parallel_info_service: Option<ServiceRef<ParallelInfoService>>,
        notifier: &'a mut dyn ContinueChainOperator,
    ) -> Self {
        let (executor_event_tx, executor_event_rx) =
            mpsc::channel::<WorkerExecuteEvent>(queue_size);
        let (parent_ready_signal_tx, _) = watch::channel(0_u64);
        let profiling_enabled = sync_profiling_info_enabled();
        Self {
            sync_dag_store,
            executors: vec![],
            queue_size,
            time_service,
            storage,
            storage2,
            vm_metrics,
            dag,
            execute_timeout_ms,
            cancel_flag,
            parallel_info_service,
            next_worker_id: 0,
            free_worker_ids: BinaryHeap::new(),
            executor_event_tx,
            executor_event_rx,
            parent_ready_signal_tx,
            parent_ready_signal_seq: 0,
            ready_parent_cache: HashSet::new(),
            profiling_enabled,
            profile: ParallelStageProfile::default(),
            notifier,
        }
    }

    fn register_worker(
        parallel_info_service: &Option<ServiceRef<ParallelInfoService>>,
        worker_id: ParallelWorkerId,
    ) {
        if let Some(parallel_info_service) = parallel_info_service {
            parallel_info_service.do_send(RegisterWorkerRequest { worker_id });
        }
    }

    fn report_worker_synced_block(
        parallel_info_service: &Option<ServiceRef<ParallelInfoService>>,
        worker_id: ParallelWorkerId,
    ) {
        if let Some(parallel_info_service) = parallel_info_service {
            parallel_info_service.do_send(ReportWorkerSyncedBlockRequest { worker_id });
        }
    }

    fn mark_worker_closed(
        parallel_info_service: &Option<ServiceRef<ParallelInfoService>>,
        free_worker_ids: &mut BinaryHeap<Reverse<ParallelWorkerId>>,
        worker: &mut DagBlockWorker,
    ) {
        worker.state = ExecuteState::Closed;
        if worker.registered {
            if let Some(parallel_info_service) = parallel_info_service {
                parallel_info_service.do_send(UnregisterWorkerRequest {
                    worker_id: worker.worker_id,
                });
            }
            free_worker_ids.push(Reverse(worker.worker_id));
            worker.registered = false;
        }
    }

    fn allocate_worker_id(&mut self) -> anyhow::Result<ParallelWorkerId> {
        if let Some(Reverse(worker_id)) = self.free_worker_ids.pop() {
            return Ok(worker_id);
        }

        let worker_id = self.next_worker_id;
        self.next_worker_id = self.next_worker_id.checked_add(1).ok_or_else(|| {
            anyhow::format_err!("parallel worker id overflow, cannot allocate new worker id")
        })?;
        Ok(worker_id)
    }

    fn signal_parent_ready(&mut self) {
        self.parent_ready_signal_seq = self.parent_ready_signal_seq.wrapping_add(1);
        let _ = self
            .parent_ready_signal_tx
            .send_replace(self.parent_ready_signal_seq);
    }

    async fn dispatch_to_worker(&mut self, block: &Block) -> anyhow::Result<bool> {
        // ready-first dispatch:
        // only route to workers that are already idle (Executed).
        // do not route to Executing(parent), otherwise child blocks arrive too early and wait.
        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed { executed_block, .. } => {
                    let executed_id = executed_block.header().id();
                    if executed_id == block.header().parent_hash()
                        || block.header().parents_hash().contains(&executed_id)
                    {
                        executor.state = ExecuteState::Executing(block.id());
                        executor
                            .sender_to_executor
                            .send(Some(block.clone()))
                            .await?;
                        return anyhow::Ok(true);
                    }
                }
                ExecuteState::Executing(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed { .. } => {
                    executor.state = ExecuteState::Executing(block.id());
                    executor
                        .sender_to_executor
                        .send(Some(block.clone()))
                        .await?;
                    return anyhow::Ok(true);
                }

                ExecuteState::Executing(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    fn block_parents_ready(&mut self, block: &Block) -> anyhow::Result<bool> {
        DagBlockExecutor::waiting_for_parents(
            &self.dag,
            self.storage.clone(),
            block.header().parents_hash(),
            &mut self.ready_parent_cache,
        )
    }

    async fn spawn_worker_for_block(&mut self, block: Block) -> anyhow::Result<()> {
        let worker_id = self.allocate_worker_id()?;
        let (sender_to_worker, executor) = DagBlockExecutor::new(
            self.executor_event_tx.clone(),
            self.queue_size,
            self.time_service.clone(),
            self.storage.clone(),
            self.storage2.clone(),
            self.vm_metrics.clone(),
            self.dag.clone(),
            self.execute_timeout_ms,
            self.parent_ready_signal_tx.subscribe(),
            worker_id,
        )?;

        self.executors.push(DagBlockWorker {
            worker_id,
            registered: true,
            sender_to_executor: sender_to_worker.clone(),
            state: ExecuteState::Executing(block.id()),
            handle: executor.start_to_execute()?,
        });
        if self.profiling_enabled {
            self.profile.observe_workers(self.executors.len());
        }
        Self::register_worker(&self.parallel_info_service, worker_id);

        sender_to_worker.send(Some(block)).await?;
        Ok(())
    }

    async fn schedule_ready_blocks(
        &mut self,
        pending_blocks: &mut Vec<Block>,
    ) -> anyhow::Result<usize> {
        if pending_blocks.is_empty() {
            return Ok(0);
        }

        let mut scheduled = 0_usize;
        let mut remaining = Vec::with_capacity(pending_blocks.len());

        for block in pending_blocks.drain(..) {
            if !self.block_parents_ready(&block)? {
                remaining.push(block);
                continue;
            }

            if self.dispatch_to_worker(&block).await? {
                scheduled = scheduled.saturating_add(1);
                continue;
            }

            self.spawn_worker_for_block(block).await?;
            scheduled = scheduled.saturating_add(1);
        }

        *pending_blocks = remaining;
        if self.profiling_enabled {
            self.profile.record_scheduled(scheduled);
            self.profile.observe_pending(pending_blocks.len());
        }
        Ok(scheduled)
    }

    async fn fallback_dispatch_one_block(
        &mut self,
        pending_blocks: &mut Vec<Block>,
    ) -> anyhow::Result<bool> {
        if pending_blocks.is_empty() {
            return Ok(false);
        }

        let block = pending_blocks.remove(0);
        if self.profiling_enabled {
            self.profile.observe_pending(pending_blocks.len());
        }
        if self.dispatch_to_worker(&block).await? {
            if self.profiling_enabled {
                self.profile.record_scheduled(1);
            }
        } else {
            self.spawn_worker_for_block(block).await?;
            if self.profiling_enabled {
                self.profile.record_scheduled(1);
            }
        }
        if self.profiling_enabled {
            self.profile.record_fallback_dispatch();
        }
        Ok(true)
    }

    fn has_executing_workers(&self) -> bool {
        self.executors
            .iter()
            .any(|worker| matches!(worker.state, ExecuteState::Executing(_)))
    }

    pub async fn process_absent_blocks(mut self) -> anyhow::Result<()> {
        let process_begin = if self.profiling_enabled {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let result = async {
            let sync_dag_store = self.sync_dag_store.clone();
            let iter = sync_dag_store.iter_at_first()?;
            let mut pending_blocks = Vec::new();
            for result_value in iter {
                if self.cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    self.abort_workers();
                    return Ok(());
                }
                let (_, value) = result_value?;
                let block = DagSyncBlock::decode_value(&value)?.block.ok_or_else(|| {
                    anyhow::format_err!("failed to decode for the block in parallel!")
                })?;
                pending_blocks.push(block);
                if self.profiling_enabled {
                    self.profile.observe_pending(pending_blocks.len());
                }
                self.flush_executor_events_nonblocking()?;
                let _ = self.schedule_ready_blocks(&mut pending_blocks).await?;
                self.flush_executor_events_nonblocking()?;
            }

            while !pending_blocks.is_empty() {
                self.flush_executor_events_nonblocking()?;
                let scheduled = self.schedule_ready_blocks(&mut pending_blocks).await?;
                self.flush_executor_events_nonblocking()?;
                if scheduled == 0 {
                    if self.profiling_enabled {
                        self.profile.record_no_ready_cycle();
                    }
                    if !self.has_executing_workers() {
                        // Fallback path:
                        // if no worker is running and no block is considered ready by ready-first,
                        // dispatch one pending block to re-enter executor-side parent waiting.
                        let _ = self
                            .fallback_dispatch_one_block(&mut pending_blocks)
                            .await?;
                        self.flush_executor_events_nonblocking()?;
                        continue;
                    }
                    if !self.wait_for_executor_event().await? {
                        return Err(anyhow::format_err!(
                            "executor event channel closed while pending blocks remain"
                        ));
                    }
                }
            }

            self.wait_for_finish().await?;
            sync_dag_store.delete_all_dag_sync_block()?;
            Ok(())
        }
        .await;

        if let Some(begin) = process_begin {
            self.log_parallel_stage_profile(begin.elapsed().as_millis(), &result);
        }

        result
    }

    fn retain_open_workers(&mut self) {
        let len = self.executors.len();
        self.executors
            .retain(|worker| !matches!(worker.state, ExecuteState::Closed));

        if len != self.executors.len() {
            info!("sync workers count: {:?}", self.executors.len());
        }
    }

    fn handle_worker_event(&mut self, event: WorkerExecuteEvent) -> anyhow::Result<bool> {
        let worker = match self
            .executors
            .iter_mut()
            .find(|worker| worker.worker_id == event.worker_id)
        {
            Some(worker) => worker,
            None => {
                warn!(
                    "received execute event from unknown worker id: {:?}, ignore",
                    event.worker_id
                );
                return Ok(false);
            }
        };

        match event.state {
            ExecuteState::Executing(block_id) => {
                worker.state = ExecuteState::Executing(block_id);
                Ok(false)
            }
            ExecuteState::Executed {
                executed_block,
                durations,
            } => {
                info!("finish to execute block {:?}", executed_block.header());
                if self.profiling_enabled {
                    self.profile.record_execute(durations);
                }
                if self.profiling_enabled {
                    let notify_begin = std::time::Instant::now();
                    self.notifier.notify((*executed_block).clone())?;
                    self.profile
                        .record_apply_notify(notify_begin.elapsed().as_millis());
                } else {
                    self.notifier.notify((*executed_block).clone())?;
                }
                Self::report_worker_synced_block(&self.parallel_info_service, worker.worker_id);
                worker.state = ExecuteState::Executed {
                    executed_block,
                    durations,
                };
                Ok(true)
            }
            ExecuteState::Error(header) => Err(anyhow::format_err!(
                "parallel worker failed while executing block: {:?}",
                header
            )),
            ExecuteState::Closed => {
                Self::mark_worker_closed(
                    &self.parallel_info_service,
                    &mut self.free_worker_ids,
                    worker,
                );
                Ok(false)
            }
        }
    }

    fn flush_executor_events_nonblocking(&mut self) -> anyhow::Result<()> {
        let mut has_new_executed = false;
        loop {
            match self.executor_event_rx.try_recv() {
                Ok(event) => {
                    has_new_executed |= self.handle_worker_event(event)?;
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        if has_new_executed {
            self.signal_parent_ready();
        }
        self.retain_open_workers();
        Ok(())
    }

    async fn wait_for_executor_event(&mut self) -> anyhow::Result<bool> {
        match self.executor_event_rx.recv().await {
            Some(event) => {
                let has_new_executed = self.handle_worker_event(event)?;
                if has_new_executed {
                    self.signal_parent_ready();
                }
                self.retain_open_workers();
                Ok(true)
            }
            None => Ok(false),
        }
    }

    async fn wait_for_finish(&mut self) -> anyhow::Result<()> {
        // tell the workers to exit
        for worker in &self.executors {
            worker.sender_to_executor.send(None).await?;
        }

        loop {
            if self.cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                self.abort_workers();
                break;
            }
            if self
                .executors
                .iter()
                .all(|worker| matches!(worker.state, ExecuteState::Closed))
            {
                break;
            }

            if !self.wait_for_executor_event().await? {
                return Err(anyhow::format_err!(
                    "executor event channel closed before workers fully closed"
                ));
            }
        }

        for mut worker in std::mem::take(&mut self.executors) {
            Self::mark_worker_closed(
                &self.parallel_info_service,
                &mut self.free_worker_ids,
                &mut worker,
            );
            match worker.handle.await {
                Ok(()) => {}
                Err(join_err) if join_err.is_cancelled() => {}
                Err(join_err) => return Err(join_err.into()),
            }
        }

        anyhow::Ok(())
    }

    fn log_parallel_stage_profile(&self, total_ms: u128, result: &anyhow::Result<()>) {
        let status = if result.is_ok() { "ok" } else { "err" };
        let blocks = self.profile.executed_blocks;
        let wait_avg = ParallelStageProfile::avg_ms(self.profile.wait_parents_ms, blocks);
        let execute_avg = ParallelStageProfile::avg_ms(self.profile.execute_ms, blocks);
        let notify_avg = ParallelStageProfile::avg_ms(self.profile.apply_notify_ms, blocks);
        let other_ms = total_ms.saturating_sub(self.profile.total_accounted_ms());

        info!(
            "{} stage=parallel_stage_wait_parents status={} executed_blocks={} total_ms={} avg_ms={}",
            SYNC_PROF_PREFIX,
            status,
            blocks,
            self.profile.wait_parents_ms,
            wait_avg
        );
        info!(
            "{} stage=parallel_stage_execute status={} executed_blocks={} total_ms={} avg_ms={}",
            SYNC_PROF_PREFIX, status, blocks, self.profile.execute_ms, execute_avg
        );
        info!(
            "{} stage=parallel_stage_apply_notify status={} executed_blocks={} total_ms={} avg_ms={}",
            SYNC_PROF_PREFIX,
            status,
            blocks,
            self.profile.apply_notify_ms,
            notify_avg
        );
        if let Err(err) = result {
            warn!(
                "{} stage=parallel_pipeline_summary status=err executed_blocks={} scheduled_blocks={} total_ms={} wait_parents_ms={} execute_ms={} apply_notify_ms={} other_ms={} peak_workers={} peak_pending_blocks={} no_ready_cycles={} fallback_dispatch_count={} error={:?}",
                SYNC_PROF_PREFIX,
                blocks,
                self.profile.scheduled_blocks,
                total_ms,
                self.profile.wait_parents_ms,
                self.profile.execute_ms,
                self.profile.apply_notify_ms,
                other_ms,
                self.profile.peak_workers,
                self.profile.peak_pending_blocks,
                self.profile.no_ready_cycles,
                self.profile.fallback_dispatch_count,
                err
            );
        } else {
            info!(
                "{} stage=parallel_pipeline_summary status=ok executed_blocks={} scheduled_blocks={} total_ms={} wait_parents_ms={} execute_ms={} apply_notify_ms={} other_ms={} peak_workers={} peak_pending_blocks={} no_ready_cycles={} fallback_dispatch_count={}",
                SYNC_PROF_PREFIX,
                blocks,
                self.profile.scheduled_blocks,
                total_ms,
                self.profile.wait_parents_ms,
                self.profile.execute_ms,
                self.profile.apply_notify_ms,
                other_ms,
                self.profile.peak_workers,
                self.profile.peak_pending_blocks,
                self.profile.no_ready_cycles,
                self.profile.fallback_dispatch_count
            );
        }
    }

    fn abort_workers(&mut self) {
        for worker in &mut self.executors {
            let _ = worker.sender_to_executor.try_send(None);
            worker.handle.abort();
            Self::mark_worker_closed(
                &self.parallel_info_service,
                &mut self.free_worker_ids,
                worker,
            );
        }
    }
}

impl Drop for DagBlockSender<'_> {
    fn drop(&mut self) {
        for worker in &mut self.executors {
            let _ = worker.sender_to_executor.try_send(None);
            worker.handle.abort();
            Self::mark_worker_closed(
                &self.parallel_info_service,
                &mut self.free_worker_ids,
                worker,
            );
        }
    }
}
