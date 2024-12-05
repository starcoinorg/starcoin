use std::{sync::Arc, time::Duration};

use starcoin_chain::{verifier::DagVerifierWithGhostData, BlockChain, ChainReader};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use super::worker_scheduler::WorkerScheduler;

#[derive(Debug, Clone)]
pub enum ExecuteState {
    Executing(HashValue),
    Executed(Box<ExecutedBlock>),
    Error(Box<BlockHeader>),
    Closed,
}

pub struct DagBlockExecutor {
    sender: Sender<ExecuteState>,
    receiver: Receiver<Option<Block>>,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
    worker_scheduler: Arc<WorkerScheduler>,
}

struct ExecutorDeconstructor {
    worker_scheduler: Arc<WorkerScheduler>,
}

impl ExecutorDeconstructor {
    pub fn new(worker_scheduler: Arc<WorkerScheduler>) -> Self {
        worker_scheduler.worker_start();
        Self { worker_scheduler }
    }
}

impl Drop for ExecutorDeconstructor {
    fn drop(&mut self) {
        self.worker_scheduler.worker_exits();
    }
}

impl DagBlockExecutor {
    pub fn new(
        sender_to_main: Sender<ExecuteState>,
        buffer_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
        worker_scheduler: Arc<WorkerScheduler>,
    ) -> anyhow::Result<(Sender<Option<Block>>, Self)> {
        let (sender_for_main, receiver) = mpsc::channel::<Option<Block>>(buffer_size);
        let executor = Self {
            sender: sender_to_main,
            receiver,
            time_service,
            storage,
            vm_metrics,
            dag,
            worker_scheduler,
        };
        anyhow::Ok((sender_for_main, executor))
    }

    pub fn waiting_for_parents(
        chain: &BlockDAG,
        storage: Arc<dyn Store>,
        parents_hash: Vec<HashValue>,
    ) -> anyhow::Result<(bool, Option<HashValue>)> {
        for parent_id in parents_hash {
            let header = match storage.get_block_header_by_hash(parent_id)? {
                Some(header) => header,
                None => return Ok((false, Some(parent_id))),
            };

            if storage.get_block_info(header.id())?.is_none() {
                return Ok((false, Some(parent_id)));
            }

            if !chain.has_dag_block(parent_id)? {
                return Ok((false, Some(parent_id)));
            }
        }
        Ok((true, None))
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            let _worker_guard = ExecutorDeconstructor::new(self.worker_scheduler.clone());
            let mut chain = None;
            loop {
                if self.worker_scheduler.check_if_stop().await {
                    info!("sync worker scheduler stopped");
                    return;
                }
                match self.receiver.recv().await {
                    Some(op_block) => {
                        let block = match op_block {
                            Some(block) => block,
                            None => {
                                info!("sync worker channel closed");
                                drop(self.sender);
                                return;
                            }
                        };
                        let header = block.header().clone();

                        info!(
                            "sync parallel worker {:p} received block: {:?}",
                            &self,
                            block.header().id()
                        );

                        const MAX_ATTEMPTS: u32 = 72000;
                        let delay = Duration::from_millis(100);
                        let mut attempts: u32 = 0;

                        loop {
                            if self.worker_scheduler.check_if_stop().await {
                                info!("sync worker scheduler stopped");
                                return;
                            }
                            match Self::waiting_for_parents(
                                &self.dag,
                                self.storage.clone(),
                                block.header().parents_hash(),
                            ) {
                                Ok((true, None)) => break,
                                Ok((false, Some(absent_id))) => {
                                    attempts = attempts.saturating_add(1);
                                    if attempts > MAX_ATTEMPTS {
                                        warn!("Timeout waiting for workers to exit, waiting for parents for block: {:?}, delay: {:?}", absent_id, delay);
                                        return;
                                    }
                                    info!(
                                        "waiting for parents for block: {:?}, waiting for: {:?}, delay: {:?}",
                                        header.id(), absent_id, delay
                                    );
                                    tokio::task::yield_now().await;
                                    tokio::time::sleep(delay).await;
                                }
                                Ok(_) => {
                                    panic!("impossible flow, check the code in waiting_for_parents")
                                }
                                Err(e) => {
                                    error!(
                                        "failed to check parents: {:?}, for reason: {:?}",
                                        header, e
                                    );
                                    match self
                                        .sender
                                        .send(ExecuteState::Error(Box::new(header.clone())))
                                        .await
                                    {
                                        Ok(_) => (),
                                        Err(e) => {
                                            error!("failed to send error state: {:?}, for reason: {:?}", header, e);
                                            return;
                                        }
                                    }
                                    return;
                                }
                            }
                        }

                        match chain {
                            None => {
                                chain = match BlockChain::new(
                                    self.time_service.clone(),
                                    block.header().parent_hash(),
                                    self.storage.clone(),
                                    self.vm_metrics.clone(),
                                    self.dag.clone(),
                                ) {
                                    Ok(new_chain) => Some(new_chain),
                                    Err(e) => {
                                        error!(
                                            "failed to create chain for block: {:?} for {:?}",
                                            block.header().id(),
                                            e
                                        );
                                        return;
                                    }
                                }
                            }
                            Some(old_chain) => {
                                if old_chain.status().head().id() != block.header().parent_hash() {
                                    chain = match old_chain.fork(block.header().parent_hash()) {
                                        Ok(new_chain) => Some(new_chain),
                                        Err(e) => {
                                            error!("failed to fork in parallel for: {:?}", e);
                                            return;
                                        }
                                    }
                                } else {
                                    chain = Some(old_chain);
                                }
                            }
                        }

                        info!(
                            "sync parallel worker {:p} will execute block: {:?}",
                            &self,
                            block.header().id()
                        );
                        match chain
                            .as_mut()
                            .expect("it cannot be none!")
                            .apply_with_verifier::<DagVerifierWithGhostData>(block)
                        {
                            Ok(executed_block) => {
                                info!(
                                    "succeed to execute block: number: {:?}, id: {:?}",
                                    executed_block.header().number(),
                                    executed_block.header().id()
                                );
                                match self
                                    .sender
                                    .send(ExecuteState::Executed(Box::new(executed_block)))
                                    .await
                                {
                                    Ok(_) => tokio::task::yield_now().await,
                                    Err(e) => {
                                        error!(
                                            "failed to send waiting state: {:?}, for reason: {:?}",
                                            header, e
                                        );
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!(
                                    "failed to execute block: {:?}, for reason: {:?}",
                                    header, e
                                );
                                match self
                                    .sender
                                    .send(ExecuteState::Error(Box::new(header.clone())))
                                    .await
                                {
                                    Ok(_) => (),
                                    Err(e) => {
                                        error!(
                                            "failed to send error state: {:?}, for reason: {:?}",
                                            header, e
                                        );
                                        return;
                                    }
                                }
                                return;
                            }
                        }
                    }
                    None => {
                        info!("sync worker channel closed");
                        drop(self.sender);
                        return;
                    }
                }
            }
        });

        anyhow::Ok(handle)
    }
}
