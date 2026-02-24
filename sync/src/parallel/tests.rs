use super::executor::{DagBlockExecutor, ExecuteState};
use super::sender::DagBlockSender;
use super::{set_test_assume_parents_ready, set_test_execute_delay_ms};
use crate::store::sync_dag_store::SyncDagStore;
use crate::tasks::continue_execute_absent_block::ContinueChainOperator;
use anyhow::Result;
use starcoin_chain::ChainReader;
use starcoin_chain_api::ExecutedBlock;
use starcoin_chain_mock::MockChain;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::HashValue;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use stream_task::CollectorState;
use tokio::sync::mpsc;

struct ExecuteDelayGuard;

impl ExecuteDelayGuard {
    fn new(delay_ms: u64) -> Self {
        set_test_execute_delay_ms(delay_ms);
        Self
    }
}

impl Drop for ExecuteDelayGuard {
    fn drop(&mut self) {
        set_test_execute_delay_ms(0);
    }
}

struct AssumeParentsReadyGuard;

impl AssumeParentsReadyGuard {
    fn new(ready: bool) -> Self {
        set_test_assume_parents_ready(ready);
        Self
    }
}

impl Drop for AssumeParentsReadyGuard {
    fn drop(&mut self) {
        set_test_assume_parents_ready(false);
    }
}

struct CountingOperator {
    notify_count: Arc<AtomicUsize>,
}

impl ContinueChainOperator for CountingOperator {
    fn has_dag_block(&self, _block_id: HashValue) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn apply(&mut self, _block: starcoin_types::block::Block) -> anyhow::Result<ExecutedBlock> {
        Err(anyhow::format_err!(
            "apply should not be called in this test"
        ))
    }

    fn notify(&mut self, _executed_block: ExecutedBlock) -> anyhow::Result<CollectorState> {
        self.notify_count.fetch_add(1, Ordering::Relaxed);
        Ok(CollectorState::Need)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execute_timeout_returns_error() -> Result<()> {
    let _guard = ExecuteDelayGuard::new(50);
    let _parents_guard = AssumeParentsReadyGuard::new(true);
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut chain = MockChain::new(net.clone())?;
    chain.produce_and_apply()?;
    let parent_header = chain.head().current_header().clone();
    let block = chain.produce_block_by_params(
        parent_header.clone(),
        vec![parent_header.id()],
        parent_header.pruning_point(),
    )?;

    let (sender_to_main, mut receiver_from_executor) = mpsc::channel(1);
    let (sender_to_worker, executor) = DagBlockExecutor::new(
        sender_to_main,
        1,
        net.time_service(),
        chain.get_storage(),
        chain.get_storage2(),
        None,
        chain.head().dag(),
        1,
    )?;

    let handle = executor.start_to_execute()?;
    sender_to_worker.send(Some(block)).await?;

    let state = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        receiver_from_executor.recv(),
    )
    .await?
    .expect("expected execute state");
    assert!(matches!(state, ExecuteState::Error(_)));

    let _ = sender_to_worker.send(None).await;
    let _ = handle.await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_process_absent_blocks_timeout_no_notify() -> Result<()> {
    let _guard = ExecuteDelayGuard::new(50);
    let _parents_guard = AssumeParentsReadyGuard::new(true);
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::Test);
    let mut chain = MockChain::new(net.clone())?;
    chain.produce_and_apply()?;
    let parent_header = chain.head().current_header().clone();
    let block = chain.produce_block_by_params(
        parent_header.clone(),
        vec![parent_header.id()],
        parent_header.pruning_point(),
    )?;
    let block_id = block.id();
    let block_number = block.header().number();

    let sync_dag_store = Arc::new(SyncDagStore::create_for_testing()?);
    sync_dag_store.save_block(block)?;

    let notify_count = Arc::new(AtomicUsize::new(0));
    let mut operator = CountingOperator {
        notify_count: notify_count.clone(),
    };

    let sender = DagBlockSender::new(
        sync_dag_store.clone(),
        16,
        net.time_service(),
        chain.get_storage(),
        chain.get_storage2(),
        None,
        chain.head().dag(),
        1,
        Arc::new(AtomicBool::new(false)),
        None,
        &mut operator,
    );

    let result = tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        sender.process_absent_blocks(),
    )
    .await?;

    assert!(result.is_err());
    assert_eq!(notify_count.load(Ordering::Relaxed), 0);
    assert!(sync_dag_store
        .get_dag_sync_block(block_number, block_id)
        .is_ok());
    Ok(())
}
