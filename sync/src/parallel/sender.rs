use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    sync::Arc,
    vec,
};

use starcoin_config::TimeService;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::info;
use starcoin_service_registry::ServiceRef;
use starcoin_storage::{Store, Store2};
use starcoin_types::block::Block;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use crate::{
    store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::SyncDagStore},
    tasks::continue_execute_absent_block::ContinueChainOperator,
};

use super::executor::{DagBlockExecutor, ExecuteState};
use super::parallel_info_service::{
    ParallelInfoService, ParallelWorkerId, RegisterWorkerRequest, ReportWorkerSyncedBlockRequest,
    UnregisterWorkerRequest,
};

struct DagBlockWorker {
    pub worker_id: ParallelWorkerId,
    pub registered: bool,
    pub sender_to_executor: Sender<Option<Block>>,
    pub receiver_from_executor: Receiver<ExecuteState>,
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

    async fn dispatch_to_worker(&mut self, block: &Block) -> anyhow::Result<bool> {
        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executing(header_id) => {
                    if *header_id == block.header().parent_hash()
                        || block.header.parents_hash().contains(header_id)
                    {
                        executor.state = ExecuteState::Executing(block.id());
                        executor
                            .sender_to_executor
                            .send(Some(block.clone()))
                            .await?;
                        return anyhow::Ok(true);
                    }
                }
                ExecuteState::Executed(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed(_) => {
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

        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed(_) => {
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

    pub async fn process_absent_blocks(mut self) -> anyhow::Result<()> {
        let sync_dag_store = self.sync_dag_store.clone();
        let iter = sync_dag_store.iter_at_first()?;
        for result_value in iter {
            if self.cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                self.abort_workers();
                return Ok(());
            }
            let (_, value) = result_value?;
            let block = DagSyncBlock::decode_value(&value)?.block.ok_or_else(|| {
                anyhow::format_err!("failed to decode for the block in parallel!")
            })?;

            // Finding the executing state is the priority
            if self.dispatch_to_worker(&block).await? {
                self.flush_executor_state().await?;
                continue;
            }

            // no suitable worker found, create a new worker
            let (sender_to_main, receiver_from_executor) =
                mpsc::channel::<ExecuteState>(self.queue_size);
            let (sender_to_worker, executor) = DagBlockExecutor::new(
                sender_to_main,
                self.queue_size,
                self.time_service.clone(),
                self.storage.clone(),
                self.storage2.clone(),
                self.vm_metrics.clone(),
                self.dag.clone(),
                self.execute_timeout_ms,
            )?;

            let worker_id = self.allocate_worker_id()?;
            self.executors.push(DagBlockWorker {
                worker_id,
                registered: true,
                sender_to_executor: sender_to_worker.clone(),
                receiver_from_executor,
                state: ExecuteState::Executing(block.id()),
                handle: executor.start_to_execute()?,
            });
            Self::register_worker(&self.parallel_info_service, worker_id);

            sender_to_worker.send(Some(block)).await?;
            self.flush_executor_state().await?;
        }

        self.wait_for_finish().await?;
        sync_dag_store.delete_all_dag_sync_block()?;
        Ok(())
    }

    async fn flush_executor_state(&mut self) -> anyhow::Result<()> {
        for worker in &mut self.executors {
            match worker.receiver_from_executor.try_recv() {
                Ok(state) => {
                    if let ExecuteState::Executed(executed_block) = state {
                        info!("finish to execute block {:?}", executed_block.header());
                        self.notifier.notify((*executed_block).clone())?;
                        Self::report_worker_synced_block(
                            &self.parallel_info_service,
                            worker.worker_id,
                        );
                        worker.state = ExecuteState::Executed(executed_block);
                    } else if let ExecuteState::Error(header) = state {
                        return Err(anyhow::format_err!(
                            "parallel worker failed while executing block: {:?}",
                            header
                        ));
                    }
                }
                Err(e) => match e {
                    mpsc::error::TryRecvError::Empty => (),
                    mpsc::error::TryRecvError::Disconnected => {
                        Self::mark_worker_closed(
                            &self.parallel_info_service,
                            &mut self.free_worker_ids,
                            worker,
                        );
                    }
                },
            }
        }

        let len = self.executors.len();
        self.executors
            .retain(|worker| !matches!(worker.state, ExecuteState::Closed));

        if len != self.executors.len() {
            info!("sync workers count: {:?}", self.executors.len());
        }

        anyhow::Ok(())
    }

    async fn wait_for_finish(mut self) -> anyhow::Result<()> {
        // tell the workers to exit
        for worker in &self.executors {
            worker.sender_to_executor.send(None).await?;
        }

        loop {
            if self.cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                self.abort_workers();
                break;
            }
            for worker in &mut self.executors {
                if let ExecuteState::Closed = worker.state {
                    continue;
                }

                match worker.receiver_from_executor.try_recv() {
                    Ok(state) => {
                        if let ExecuteState::Executed(executed_block) = state {
                            info!("finish to execute block {:?}", executed_block.header());
                            Self::report_worker_synced_block(
                                &self.parallel_info_service,
                                worker.worker_id,
                            );
                            self.notifier.notify(*executed_block)?;
                        } else if let ExecuteState::Error(header) = state {
                            return Err(anyhow::format_err!(
                                "parallel worker failed while finishing block execution: {:?}",
                                header
                            ));
                        }
                    }
                    Err(e) => match e {
                        mpsc::error::TryRecvError::Empty => (),
                        mpsc::error::TryRecvError::Disconnected => {
                            Self::mark_worker_closed(
                                &self.parallel_info_service,
                                &mut self.free_worker_ids,
                                worker,
                            );
                        }
                    },
                }
            }

            if self
                .executors
                .iter()
                .all(|worker| matches!(worker.state, ExecuteState::Closed))
            {
                break;
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
