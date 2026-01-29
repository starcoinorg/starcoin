use std::sync::Arc;

use starcoin_chain::{verifier::FullVerifier, BlockChain, ChainReader};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
use starcoin_storage::Store;
use starcoin_storage::Store2;
use starcoin_types::block::{Block, BlockHeader};
#[cfg(test)]
use std::sync::atomic::AtomicBool;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

const MAX_TOTAL_WAITING_TIME: u64 = 3600000; // an hour
#[cfg(test)]
static TEST_EXECUTE_DELAY_MS: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static TEST_ASSUME_PARENTS_READY: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
pub(crate) fn set_test_execute_delay_ms(delay_ms: u64) {
    TEST_EXECUTE_DELAY_MS.store(delay_ms, Ordering::Relaxed);
}

#[cfg(test)]
pub(crate) fn set_test_assume_parents_ready(ready: bool) {
    TEST_ASSUME_PARENTS_READY.store(ready, Ordering::Relaxed);
}

#[allow(dead_code)]
#[derive(Debug)]
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
    storage2: Arc<dyn Store2>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
    execute_timeout_ms: u64,
}

impl DagBlockExecutor {
    pub fn new(
        sender_to_main: Sender<ExecuteState>,
        buffer_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        storage2: Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
        execute_timeout_ms: u64,
    ) -> anyhow::Result<(Sender<Option<Block>>, Self)> {
        let (sender_for_main, receiver) = mpsc::channel::<Option<Block>>(buffer_size);
        let executor = Self {
            sender: sender_to_main,
            receiver,
            time_service,
            storage,
            storage2,
            vm_metrics,
            dag,
            execute_timeout_ms,
        };
        anyhow::Ok((sender_for_main, executor))
    }

    pub fn waiting_for_parents(
        chain: &BlockDAG,
        storage: Arc<dyn Store>,
        parents_hash: &[HashValue],
    ) -> anyhow::Result<bool> {
        #[cfg(test)]
        if TEST_ASSUME_PARENTS_READY.load(Ordering::Relaxed) {
            return Ok(true);
        }
        for parent_id in parents_hash {
            let header = match storage.get_block_header_by_hash(*parent_id)? {
                Some(header) => header,
                None => return Ok(false),
            };

            if storage.get_block_info(header.id())?.is_none() {
                return Ok(false);
            }

            if !chain.has_block_connected(&header)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            let mut chain = None;
            loop {
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

                        let mut total_waiting_time: u64 = 0;
                        let waiting_per_time: u64 = 100;
                        loop {
                            match Self::waiting_for_parents(
                                &self.dag,
                                self.storage.clone(),
                                block.header().parents_hash(),
                            ) {
                                Ok(true) => break,
                                Ok(false) => {
                                    if total_waiting_time >= MAX_TOTAL_WAITING_TIME {
                                        error!(
                                            "failed to check parents: {:?}, for reason: timeout",
                                            header
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
                                    tokio::task::yield_now().await;
                                    tokio::time::sleep(tokio::time::Duration::from_millis(
                                        waiting_per_time,
                                    ))
                                    .await;
                                    total_waiting_time =
                                        total_waiting_time.saturating_add(waiting_per_time);
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
                                    self.storage2.clone(),
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
                        let mut local_chain = chain.take().expect("it cannot be none!");
                        let mut execute_handle = tokio::task::spawn_blocking(move || {
                            #[cfg(test)]
                            {
                                let delay_ms = TEST_EXECUTE_DELAY_MS.load(Ordering::Relaxed);
                                if delay_ms > 0 {
                                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                                }
                            }
                            let result = local_chain.apply_with_verifier::<FullVerifier>(block);
                            (local_chain, result)
                        });

                        match tokio::time::timeout(
                            tokio::time::Duration::from_millis(self.execute_timeout_ms),
                            &mut execute_handle,
                        )
                        .await
                        {
                            Ok(Ok((updated_chain, result))) => {
                                chain = Some(updated_chain);
                                match result {
                                    Ok(executed_block) => {
                                        info!(
                                            "succeed to execute block: number: {:?}, id: {:?}",
                                            executed_block.header().number(),
                                            executed_block.header().id()
                                        );
                                        // Adjust time after successful block execution to ensure proper time synchronization
                                        // This is important for validating subsequent blocks in the parallel execution pipeline
                                        self.time_service
                                            .adjust(executed_block.header().timestamp());
                                        match self
                                            .sender
                                            .send(ExecuteState::Executed(Box::new(executed_block)))
                                            .await
                                        {
                                            Ok(_) => tokio::task::yield_now().await,
                                            Err(e) => {
                                                error!(
                                                    "failed to send executed state: {:?}, for reason: {:?}",
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
                            Ok(Err(e)) => {
                                error!(
                                    "sync parallel worker join error: {:?}, header: {:?}",
                                    e, header
                                );
                                let _ = self
                                    .sender
                                    .send(ExecuteState::Error(Box::new(header.clone())))
                                    .await;
                                return;
                            }
                            Err(_) => {
                                error!("sync parallel worker execute timeout: {:?}", header);
                                execute_handle.abort();
                                let _ = self
                                    .sender
                                    .send(ExecuteState::Error(Box::new(header.clone())))
                                    .await;
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
