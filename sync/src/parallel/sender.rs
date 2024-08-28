use std::{ops::Deref, sync::Arc, vec};

use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec, reachability::inquirer};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::error;
use starcoin_network::worker;
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::{self, SyncDagStore}};

use super::executor::{DagBlockExecutor, ExecuteState};

struct DagBlockWorker {
    pub sender_to_executor: Sender<Block>,
    pub receiver_from_executor: Receiver<ExecuteState>,
    pub state: ExecuteState,
}

pub struct DagBlockSender {
    sync_dag_store: SyncDagStore,
    executors: Vec<DagBlockWorker>,
    queue_size: usize,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
}

impl DagBlockSender {
    pub fn new(
        sync_dag_store: SyncDagStore,
        queue_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> Self {
        Self {
            sync_dag_store,
            executors: vec![],
            queue_size,
            time_service,
            storage,
            vm_metrics,
            dag,
        }
    }

    async fn dispatch_to_worker(&mut self, block: &Block) -> anyhow::Result<bool> {
        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed(executing_header_block) => {
                    if executing_header_block.id() == block.header().parent_hash() {
                        executor.state = ExecuteState::Executing(block.id());
                        executor.sender_to_executor.send(block.clone()).await?;
                        return anyhow::Ok(true);
                    }
                }
                ExecuteState::Executing(header_id) => {
                    if *header_id == block.header().id() {
                        executor.state = ExecuteState::Executing(block.id());
                        executor.sender_to_executor.send(block.clone()).await?;
                        return anyhow::Ok(true);
                    }
                }
                ExecuteState::Ready(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    pub async fn process_absent_blocks(&mut self) -> anyhow::Result<()> {
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
            let chain_header = self
                .storage
                .get_block_header_by_hash(block.header().parent_hash())?
                .ok_or_else(|| {
                    anyhow::format_err!(
                        "in parallel sync, failed to get the block header by hash: {:?}",
                        block.header().parent_hash()
                    )
                })?;
            let (sender_to_main, receiver_from_executor) = mpsc::channel::<ExecuteState>(self.queue_size);
            let (sender_to_worker, executor) = DagBlockExecutor::new(
                sender_to_main,
                self.queue_size,
                self.time_service.clone(),
                chain_header.id(),
                self.storage.clone(),
                self.vm_metrics.clone(),
                self.dag.clone(),
            )?;

            self.executors.push(DagBlockWorker {
                sender_to_executor: sender_to_worker.clone(),
                receiver_from_executor,
                state: ExecuteState::Ready(block.id()),
            });

            executor.start_to_execute()?;
            sender_to_worker.send(block).await?;

            self.flush_executor_state().await?;
        }

        self.sync_dag_store.delete_all_dag_sync_block()?;

        self.wait_for_finish().await?;
        Ok(())
    }
    
    async fn flush_executor_state(&mut self) -> anyhow::Result<()> {
        for worker in &mut self.executors {
            match worker.receiver_from_executor.try_recv() {
                Ok(state) => worker.state = state,
                Err(e) => {
                    match e {
                        mpsc::error::TryRecvError::Empty => continue,
                        mpsc::error::TryRecvError::Disconnected => worker.state = ExecuteState::Closed,
                    }
                }
            }
        }

        self.executors.retain(|worker| {
            if let ExecuteState::Closed = worker.state {
                false
            } else {
                true
            }
        });
        anyhow::Ok(())
    }

    async fn wait_for_finish(&mut self) -> anyhow::Result<()> {
        loop {
            for worker in &mut self.executors {
                match worker.receiver_from_executor.try_recv() {
                    Ok(state) => worker.state = state,
                    Err(e) => {
                        match e {
                            mpsc::error::TryRecvError::Empty => continue,
                            mpsc::error::TryRecvError::Disconnected => worker.state = ExecuteState::Closed,
                        }
                    }
                }
            }

            self.executors.retain(|worker| {
                if let ExecuteState::Closed = worker.state {
                    false
                } else {
                    true
                }
            });           

            if self.executors.is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }

        anyhow::Ok(())
    }
}
