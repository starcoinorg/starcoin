use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::sync_profiling_info_enabled;
use starcoin_chain::{verifier::FullVerifier, BlockChain, ChainReader};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_storage::Store;
use starcoin_storage::Store2;
use starcoin_types::block::{Block, BlockHeader};
#[cfg(test)]
use std::sync::atomic::AtomicBool;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        watch,
    },
    task::JoinHandle,
};

const MAX_TOTAL_WAITING_TIME: u64 = 3600000; // an hour
const WAIT_PARENTS_LOG_MS: u128 = 500;
const EXECUTE_SLOW_LOG_MS: u128 = 200;
const SYNC_PROF_PREFIX: &str = "[sync-prof]";
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
    Executed {
        executed_block: Box<ExecutedBlock>,
        durations: ExecuteDurations,
    },
    Error(Box<BlockHeader>),
    Closed,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExecuteDurations {
    pub wait_parents_ms: u128,
    pub execute_ms: u128,
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
    parent_ready_rx: watch::Receiver<u64>,
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
        parent_ready_rx: watch::Receiver<u64>,
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
            parent_ready_rx,
        };
        anyhow::Ok((sender_for_main, executor))
    }

    pub fn waiting_for_parents(
        chain: &BlockDAG,
        storage: Arc<dyn Store>,
        parents_hash: &[HashValue],
        ready_parent_cache: &mut HashSet<HashValue>,
    ) -> anyhow::Result<bool> {
        #[cfg(test)]
        if TEST_ASSUME_PARENTS_READY.load(Ordering::Relaxed) {
            return Ok(true);
        }
        for parent_id in parents_hash {
            if ready_parent_cache.contains(parent_id) {
                continue;
            }
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
            ready_parent_cache.insert(*parent_id);
        }
        Ok(true)
    }

    async fn wait_for_parents_ready(
        &mut self,
        header: &BlockHeader,
        ready_parent_cache: &mut HashSet<HashValue>,
    ) -> anyhow::Result<u128> {
        let wait_begin = Instant::now();
        loop {
            match Self::waiting_for_parents(
                &self.dag,
                self.storage.clone(),
                header.parents_hash(),
                ready_parent_cache,
            ) {
                Ok(true) => {
                    let waited_ms = wait_begin.elapsed().as_millis();
                    if sync_profiling_info_enabled() && waited_ms >= WAIT_PARENTS_LOG_MS {
                        info!(
                            "{} stage=wait_for_parents status=slow block_id={} block_number={} waited_ms={}",
                            SYNC_PROF_PREFIX,
                            header.id(),
                            header.number(),
                            waited_ms
                        );
                    }
                    return Ok(waited_ms);
                }
                Ok(false) => {}
                Err(err) => return Err(err),
            }

            let waited_ms = wait_begin.elapsed().as_millis();
            if waited_ms >= u128::from(MAX_TOTAL_WAITING_TIME) {
                return Err(anyhow::format_err!(
                    "failed to check parents: {:?}, for reason: timeout",
                    header
                ));
            }

            let waited_u64 = waited_ms.min(u128::from(u64::MAX)) as u64;
            let remaining_ms = MAX_TOTAL_WAITING_TIME.saturating_sub(waited_u64);
            match tokio::time::timeout(
                Duration::from_millis(remaining_ms),
                self.parent_ready_rx.changed(),
            )
            .await
            {
                Ok(Ok(())) => {}
                Ok(Err(_)) => {
                    return Err(anyhow::format_err!(
                        "failed to wait for parent-ready event: channel closed, block: {:?}",
                        header
                    ));
                }
                Err(_) => {
                    return Err(anyhow::format_err!(
                        "failed to check parents: {:?}, for reason: timeout",
                        header
                    ));
                }
            }
        }
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            let mut chain = None;
            let mut ready_parent_cache = HashSet::new();
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

                        let wait_parents_ms = match self
                            .wait_for_parents_ready(&header, &mut ready_parent_cache)
                            .await
                        {
                            Ok(wait_parents_ms) => wait_parents_ms,
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
                                        error!(
                                            "failed to send error state: {:?}, for reason: {:?}",
                                            header, e
                                        );
                                        return;
                                    }
                                }
                                return;
                            }
                        };

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
                        let execute_begin = Instant::now();
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

                        let mut execute_timed_out = false;
                        let execute_result = match tokio::time::timeout(
                            tokio::time::Duration::from_millis(self.execute_timeout_ms),
                            &mut execute_handle,
                        )
                        .await
                        {
                            Ok(result) => result,
                            Err(_) => {
                                execute_timed_out = true;
                                warn!(
                                    "sync parallel worker execute exceeded timeout ({}ms), wait for completion before reporting failure: {:?}",
                                    self.execute_timeout_ms,
                                    header
                                );
                                execute_handle.await
                            }
                        };
                        if execute_timed_out {
                            error!("sync parallel worker execute timeout: {:?}", header);
                            if sync_profiling_info_enabled() {
                                error!(
                                    "{} stage=parallel_execute status=timeout block_id={} block_number={} elapsed_ms={}",
                                    SYNC_PROF_PREFIX,
                                    header.id(),
                                    header.number(),
                                    execute_begin.elapsed().as_millis()
                                );
                            }
                            let _ = self
                                .sender
                                .send(ExecuteState::Error(Box::new(header.clone())))
                                .await;
                            return;
                        }
                        match execute_result {
                            Ok((updated_chain, result)) => {
                                chain = Some(updated_chain);
                                match result {
                                    Ok(executed_block) => {
                                        let execute_elapsed_ms =
                                            execute_begin.elapsed().as_millis();
                                        if sync_profiling_info_enabled()
                                            && execute_elapsed_ms >= EXECUTE_SLOW_LOG_MS
                                        {
                                            info!(
                                                "{} stage=parallel_execute status=ok block_id={} block_number={} elapsed_ms={}",
                                                SYNC_PROF_PREFIX,
                                                executed_block.header().id(),
                                                executed_block.header().number(),
                                                execute_elapsed_ms
                                            );
                                        }
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
                                            .send(ExecuteState::Executed {
                                                executed_block: Box::new(executed_block),
                                                durations: ExecuteDurations {
                                                    wait_parents_ms,
                                                    execute_ms: execute_elapsed_ms,
                                                },
                                            })
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
                                        if sync_profiling_info_enabled() {
                                            error!(
                                                "{} stage=parallel_execute status=err block_id={} block_number={} elapsed_ms={} error={:?}",
                                                SYNC_PROF_PREFIX,
                                                header.id(),
                                                header.number(),
                                                execute_begin.elapsed().as_millis(),
                                                e
                                            );
                                        }
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
                            Err(e) => {
                                error!(
                                    "sync parallel worker join error: {:?}, header: {:?}",
                                    e, header
                                );
                                if sync_profiling_info_enabled() {
                                    error!(
                                        "{} stage=parallel_execute status=join_err block_id={} block_number={} elapsed_ms={} error={:?}",
                                        SYNC_PROF_PREFIX,
                                        header.id(),
                                        header.number(),
                                        execute_begin.elapsed().as_millis(),
                                        e
                                    );
                                }
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
