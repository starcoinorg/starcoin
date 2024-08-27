#![feature(linked_list_cursors)] 
use std::{collections::LinkedList, ops::Deref, sync::Arc};

use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec, reachability::inquirer};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::error;
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::SyncDagStore};

use super::executor::{DagBlockExecutor, ExecuteState};

struct DagBlockWorker {
    pub sender_to_executor: Sender<Block>,
    pub receiver_from_executor: Receiver<ExecuteState>,
    pub state: ExecuteState,
}

struct DagBlockSender {
    sync_dag_store: SyncDagStore,
    executors: LinkedList<DagBlockWorker>,
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
            executors: LinkedList::new(),
            queue_size,
            time_service,
            storage,
            vm_metrics,
            dag,
        }
    }

    async fn dispatch_to_executing_ancestor_worker(&self, block: &Block) -> anyhow::Result<bool> {
        for executor in &self.executors {
            match &executor.state {
                ExecuteState::Executing(executing_header_block) => {
                    if inquirer::is_dag_ancestor_of(
                        self.sync_dag_store.reachability_store.read().deref(),
                        executing_header_block.id(),
                        block.id(),
                    )? {
                        executor.sender_to_executor.send(block.clone()).await?;
                        return anyhow::Ok(true);
                    }
                }
                &ExecuteState::Waiting(_) | ExecuteState::Error(_) => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    async fn dispatch_to_waiting_ancestor_worker(&self, block: &Block) -> anyhow::Result<bool> {
        for executor in &self.executors {
            match &executor.state {
                ExecuteState::Waiting(executing_header_block) => {
                    if inquirer::is_dag_ancestor_of(
                        self.sync_dag_store.reachability_store.read().deref(),
                        executing_header_block.id(),
                        block.id(),
                    )? {
                        executor.sender_to_executor.send(block.clone()).await?;
                        return anyhow::Ok(true);
                    }
                }
                &ExecuteState::Executing(_) | ExecuteState::Error(_) => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    async fn dispatch_to_waiting_worker(&self, block: &Block) -> anyhow::Result<bool> {
        for executor in &self.executors {
            match &executor.state {
                ExecuteState::Waiting(_) => {
                    executor.sender_to_executor.send(block.clone()).await?;
                    return anyhow::Ok(true);
                }
                &ExecuteState::Executing(_) | ExecuteState::Error(_) => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    pub async fn process_absent_blocks(&mut self) -> anyhow::Result<()> {
        let iter = self.sync_dag_store.iter_at_first()?;
        for result_value in iter {
            let (_, value) = result_value?;
            let block = DagSyncBlock::decode_value(&value)?.block.ok_or_else(|| {
                anyhow::format_err!("failed to decode for the block in parallel!")
            })?;

            // Finding the executing state is the priority
            if self.dispatch_to_executing_ancestor_worker(&block).await? {
                continue;
            }

            // Finding the waiting state is the secondary
            if self.dispatch_to_waiting_ancestor_worker(&block).await? {
                continue;
            }

            // Finding the waiting state is the third
            if self.dispatch_to_waiting_worker(&block).await? {
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
            let (sender_for_main, receiver) = mpsc::channel::<ExecuteState>(self.queue_size);
            let (sender_to_worker, executor) = DagBlockExecutor::new(
                sender_for_main,
                self.queue_size,
                self.time_service.clone(),
                chain_header.id(),
                self.storage.clone(),
                self.vm_metrics.clone(),
                self.dag.clone(),
            )?;

            self.executors.push_back(DagBlockWorker {
                sender_to_executor: sender_to_worker.clone(),
                receiver_from_executor: receiver,
                state: ExecuteState::Waiting(chain_header),
            });

            executor.start_to_execute()?;
            sender_to_worker.send(block).await?;

            self.flush_executor_state().await?;
        }
        Ok(())
    }
    
    async fn flush_executor_state(&mut self) -> anyhow::Result<()> {
        let mut cursor = self.executors.cursor_front_mut();

        while let Some(&mut worker) = cursor.current() {
            match worker.receiver_from_executor.recv().await {
                Some(state) => {
                    worker.state = state;
                    cursor.move_next();
                }
                None => {
                    let _ = cursor.remove_current();
                },
            }
        }

        anyhow::Ok(())
    }
}
