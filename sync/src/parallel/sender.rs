use std::{ops::Deref, sync::Arc, vec};

use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schema::ValueCodec, reachability::inquirer};
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
use starcoin_network::worker;
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::{sync::mpsc::{self, Receiver, Sender}, task::JoinHandle};

use crate::{store::{sync_absent_ancestor::DagSyncBlock, sync_dag_store::{self, SyncDagStore}}, tasks::continue_execute_absent_block::ContinueChainOperator};

use super::executor::{DagBlockExecutor, ExecuteState};

struct DagBlockWorker {
    pub sender_to_executor: Sender<Block>,
    pub receiver_from_executor: Receiver<ExecuteState>,
    pub state: ExecuteState,
    pub handle: JoinHandle<()>,
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
                // ExecuteState::Executed(executing_header_block) => {
                //     if executing_header_block.id() == block.header().parent_hash() {
                //         executor.state = ExecuteState::Executing(block.id());
                //         executor.sender_to_executor.send(block.clone()).await?;
                //         return anyhow::Ok(true);
                //     }
                // }
                ExecuteState::Executing(header_id) => {
                    if *header_id == block.header().parent_hash() || block.header.parents_hash().contains(header_id) {
                        executor.state = ExecuteState::Executing(block.id());
                        executor.sender_to_executor.send(block.clone()).await?;
                        return anyhow::Ok(true);
                    }
                }
                ExecuteState::Executed(_) | ExecuteState::Ready(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        for executor in &mut self.executors {
            match &executor.state {
                ExecuteState::Executed(_) => {
                    executor.state = ExecuteState::Executing(block.id());
                    executor.sender_to_executor.send(block.clone()).await?;
                    return anyhow::Ok(true);
                }

                ExecuteState::Executing(_) | ExecuteState::Ready(_) | ExecuteState::Error(_) | ExecuteState::Closed => {
                    continue;
                }
            }
        }

        anyhow::Ok(false)
    }

    pub async fn process_absent_blocks<'a>(mut self, notify: &'a mut dyn ContinueChainOperator) -> anyhow::Result<()> {
        let sync_dag_store = self.sync_dag_store.clone();
        let iter = sync_dag_store.iter_at_first()?;
        for result_value in iter {
            let (_, value) = result_value?;
            let block = DagSyncBlock::decode_value(&value)?.block.ok_or_else(|| {
                anyhow::format_err!("failed to decode for the block in parallel!")
            })?;

            // Finding the executing state is the priority
            if self.dispatch_to_worker(&block).await? {
                continue;
            }

            // no suitable worker found, create a new worker
            let (sender_to_main, receiver_from_executor) = mpsc::channel::<ExecuteState>(self.queue_size);
            let (sender_to_worker, executor) = DagBlockExecutor::new(
                sender_to_main,
                self.queue_size,
                self.time_service.clone(),
                block.header().parent_hash(),
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

            sender_to_worker.send(block).await?;

            self.flush_executor_state(notify).await?;
        }

        self.sync_dag_store.delete_all_dag_sync_block()?;

        self.wait_for_finish().await?;

        Ok(())
    }
    
    async fn flush_executor_state<'a>(&mut self, notify: &'a mut dyn ContinueChainOperator) -> anyhow::Result<()> {
        for worker in &mut self.executors {
            match worker.receiver_from_executor.try_recv() {
                Ok(state) => {
                    match state {
                        ExecuteState::Executed(executed_block) => {
                            notify.notify(&executed_block)?;
                            worker.state = ExecuteState::Closed;
                        }
                        _ => ()
                    }
                }
                Err(e) => {
                    match e {
                        mpsc::error::TryRecvError::Empty => (),
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
        info!("sync workers count: {:?}", self.executors.len());
        anyhow::Ok(())
    }

    async fn wait_for_finish(self) -> anyhow::Result<()> {
        for mut worker in self.executors {
            drop(worker.sender_to_executor);
            while let Some(_) = worker.receiver_from_executor.recv().await {
                ()
            }
            worker.handle.await?;
        }

        anyhow::Ok(())
    }
}
