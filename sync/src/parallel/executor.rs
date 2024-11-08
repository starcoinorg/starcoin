use std::sync::Arc;

use starcoin_chain::{verifier::DagVerifierWithGhostData, BlockChain, ChainReader};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
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

impl Drop for DagBlockExecutor {
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
        worker_scheduler.worker_start();
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
    ) -> anyhow::Result<bool> {
        for parent_id in parents_hash {
            let header = match storage.get_block_header_by_hash(parent_id)? {
                Some(header) => header,
                None => return Ok(false),
            };

            if storage.get_block_info(header.id())?.is_none() {
                return Ok(false);
            }

            if !chain.has_dag_block(parent_id)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
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
                                return;
                            }
                        };
                        let header = block.header().clone();

                        info!(
                            "sync parallel worker {:p} received block: {:?}",
                            &self,
                            block.header().id()
                        );

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
                                Ok(true) => break,
                                Ok(false) => {
                                    tokio::task::yield_now().await;
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100))
                                        .await
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
                        return;
                    }
                }
            }
        });

        anyhow::Ok(handle)
    }
}
