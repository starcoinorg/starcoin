use std::{sync::Arc, vec};

use starcoin_config::TimeService;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::info;
use starcoin_storage::Store;
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

struct DagBlockWorker {
    pub sender_to_executor: Sender<Option<Block>>,
    pub receiver_from_executor: Receiver<ExecuteState>,
    pub state: ExecuteState,
    pub handle: JoinHandle<()>,
}

pub struct DagBlockSender<'a> {
    sync_dag_store: SyncDagStore,
    executors: Vec<DagBlockWorker>,
    queue_size: usize,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
    notifier: &'a mut dyn ContinueChainOperator,
}

impl<'a> DagBlockSender<'a> {
    pub fn new(
        sync_dag_store: SyncDagStore,
        queue_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
        notifier: &'a mut dyn ContinueChainOperator,
    ) -> Self {
        Self {
            sync_dag_store,
            executors: vec![],
            queue_size,
            time_service,
            storage,
            vm_metrics,
            dag,
            notifier,
        }
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

        anyhow::Ok(false)
    }

    pub async fn process_absent_blocks(mut self) -> anyhow::Result<()> {
        let sync_dag_store = self.sync_dag_store.clone();
        let iter = sync_dag_store.iter_at_first()?;
        for result_value in iter {
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
                self.vm_metrics.clone(),
                self.dag.clone(),
            )?;

            self.executors.push(DagBlockWorker {
                sender_to_executor: sender_to_worker.clone(),
                receiver_from_executor,
                state: ExecuteState::Executing(block.id()),
                handle: executor.start_to_execute()?,
            });

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
                        worker.state = ExecuteState::Executed(executed_block);
                    }
                }
                Err(e) => match e {
                    mpsc::error::TryRecvError::Empty => (),
                    mpsc::error::TryRecvError::Disconnected => worker.state = ExecuteState::Closed,
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
            for worker in &mut self.executors {
                if let ExecuteState::Closed = worker.state {
                    continue;
                }

                match worker.receiver_from_executor.try_recv() {
                    Ok(state) => {
                        if let ExecuteState::Executed(executed_block) = state {
                            info!("finish to execute block {:?}", executed_block.header());
                            self.notifier.notify(*executed_block)?;
                        }
                    }
                    Err(e) => match e {
                        mpsc::error::TryRecvError::Empty => (),
                        mpsc::error::TryRecvError::Disconnected => {
                            worker.state = ExecuteState::Closed
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

        for worker in self.executors {
            worker.handle.await?;
        }

        anyhow::Ok(())
    }
}
