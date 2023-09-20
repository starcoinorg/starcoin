use std::sync::{Arc, Mutex};

use anyhow::{anyhow, format_err, Ok};
use async_std::task::Task;
use futures::{future::BoxFuture, FutureExt};
use network_api::PeerProvider;
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, Accumulator, AccumulatorTreeStore, MerkleAccumulator,
};
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_consensus::BlockDAG;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{debug, info};
use starcoin_network::NetworkServiceRef;
use starcoin_service_registry::ServiceRef;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, storage::CodecKVStore, Store};
use starcoin_time_service::TimeService;
use stream_task::{
    Generator, TaskError, TaskEventCounterHandle, TaskFuture, TaskGenerator, TaskHandle,
};

use crate::{block_connector::BlockConnectorService, verified_rpc_client::VerifiedRpcClient};

use super::{
    sync_dag_accumulator_task::{SyncDagAccumulatorCollector, SyncDagAccumulatorTask},
    sync_dag_block_task::SyncDagBlockTask,
    sync_find_ancestor_task::{AncestorCollector, FindAncestorTask},
    BlockCollector, BlockConnectedEventHandle, ExtSyncTaskErrorHandle,
};

pub async fn find_dag_ancestor_task(
    local_accumulator_info: AccumulatorInfo,
    target_accumulator_info: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_store: Arc<dyn AccumulatorTreeStore>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    event_handle: Arc<TaskEventCounterHandle>,
) -> anyhow::Result<AccumulatorInfo> {
    let max_retry_times = 10; // in startcoin, it is in config
    let delay_milliseconds_on_error = 100;

    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    // here should compare the dag's node not accumulator leaf node
    let sync_task = TaskGenerator::new(
        FindAncestorTask::new(
            local_accumulator_info.num_leaves - 1,
            target_accumulator_info.num_leaves,
            fetcher,
        ),
        2,
        max_retry_times,
        delay_milliseconds_on_error,
        AncestorCollector::new(
            Arc::new(MerkleAccumulator::new_with_info(
                local_accumulator_info,
                accumulator_store.clone(),
            )),
            accumulator_snapshot.clone(),
        ),
        event_handle.clone(),
        ext_error_handle.clone(),
    )
    .generate();
    let (fut, _handle) = sync_task.with_handle();
    match fut.await {
        anyhow::Result::Ok(ancestor) => {
            return Ok(ancestor);
        }
        Err(error) => {
            return Err(anyhow!(error));
        }
    }
}

async fn sync_accumulator(
    local_accumulator_info: AccumulatorInfo,
    target_accumulator_info: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_store: Arc<dyn AccumulatorTreeStore>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
) -> anyhow::Result<(u64, MerkleAccumulator)> {
    let max_retry_times = 10; // in startcoin, it is in config
    let delay_milliseconds_on_error = 100;

    let start_index = local_accumulator_info.get_num_leaves().saturating_sub(1);

    let event_handle = Arc::new(TaskEventCounterHandle::new());

    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    let sync_task = TaskGenerator::new(
        SyncDagAccumulatorTask::new(
            start_index.saturating_add(1),
            3,
            target_accumulator_info.num_leaves,
            fetcher.clone(),
        ),
        2,
        max_retry_times,
        delay_milliseconds_on_error,
        SyncDagAccumulatorCollector::new(
            MerkleAccumulator::new_with_info(local_accumulator_info, accumulator_store.clone()),
            accumulator_snapshot.clone(),
            target_accumulator_info,
            start_index,
        ),
        event_handle.clone(),
        ext_error_handle,
    )
    .generate();
    let (fut, handle) = sync_task.with_handle();
    match fut.await {
        anyhow::Result::Ok((start_index, full_accumulator)) => {
            return anyhow::Result::Ok((start_index, full_accumulator));
        }
        Err(error) => {
            return Err(anyhow!(error));
        }
    }

    // TODO: we need to talk about this
    // .and_then(|sync_accumulator_result, event_handle| {
    //     let sync_dag_accumulator_task = TaskGenerator::new(
    //         SyncDagBlockTask::new(),
    //         2,
    //         max_retry_times,
    //         delay_milliseconds_on_error,
    //         SyncDagAccumulatorCollector::new(),
    //         event_handle.clone(),
    //         ext_error_handle,
    //     );
    //     Ok(sync_dag_accumulator_task)
    // });
    // return Ok(async_std::task::block_on(sync));
    // match async_std::task::block_on(sync) {
    //     std::result::Result::Ok((index, accumulator)) => {
    //         debug!("sync accumulator success, target accumulator info's leaf count = {}, root hash = {}, begin index = {}",
    //             accumulator.get_info().get_num_leaves(), accumulator.get_info().get_accumulator_root(), index);
    //         return Ok((index, accumulator));
    //     }
    //     Err(error) => {
    //         println!("sync accumulator error: {}", error.to_string());
    //         Err(error.into())
    //     }
    // }
}

fn get_start_block_id(
    accumulator: &MerkleAccumulator,
    start_index: u64,
    local_store: Arc<dyn Store>,
) -> anyhow::Result<HashValue> {
    let last_block_id = accumulator
        .get_leaf(start_index)?
        .expect("last block id should not be None");

    let mut snapshot = local_store
        .query_by_hash(last_block_id)?
        .expect("tips should not be None");
    snapshot.child_hashes.sort();
    Ok(snapshot
        .child_hashes
        .iter()
        .last()
        .expect("last block id should not be None")
        .clone())
}

async fn sync_dag_block<H, N>(
    start_index: u64,
    accumulator: MerkleAccumulator,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    local_store: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    block_event_handle: H,
    network: N,
    skip_pow_verify_when_sync: bool,
    dag: Arc<Mutex<BlockDAG>>,
    block_chain_service: ServiceRef<BlockConnectorService>,
    vm_metrics: Option<VMMetrics>,
) -> anyhow::Result<BlockChain>
where
    H: BlockConnectedEventHandle + Sync + 'static,
    N: PeerProvider + Clone + 'static,
{
    let max_retry_times = 10; // in startcoin, it is in config
    let delay_milliseconds_on_error = 100;
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    let start_block_id = get_start_block_id(&accumulator, start_index, local_store.clone())
        .map_err(|err| TaskError::BreakError(anyhow!(err)));
    let chain = BlockChain::new(
        time_service.clone(),
        start_block_id?,
        local_store.clone(),
        vm_metrics,
    )
    .map_err(|err| TaskError::BreakError(anyhow!(err)));

    let leaf = accumulator
        .get_leaf(start_index)
        .expect(format!("index: {} must be valid", start_index).as_str())
        .expect(format!("index: {} should not be None", start_index).as_str());

    let mut snapshot = accumulator_snapshot
        .get(leaf)
        .expect(format!("index: {} must be valid for getting snapshot", start_index).as_str())
        .expect(
            format!(
                "index: {} should not be None for getting snapshot",
                start_index
            )
            .as_str(),
        );

    snapshot.child_hashes.sort();
    let last_chain_block = snapshot
        .child_hashes
        .iter()
        .last()
        .expect("block id should not be None")
        .clone();

    let current_block_info = local_store
        .get_block_info(last_chain_block)?
        .ok_or_else(|| format_err!("Can not find block info by id: {}", last_chain_block))
        .map_err(|err| TaskError::BreakError(anyhow!(err)));

    let accumulator_info = accumulator.get_info();
    let accumulator_root = accumulator.root_hash();
    let sync_task = TaskGenerator::new(
        SyncDagBlockTask::new(
            accumulator,
            start_index.saturating_add(1),
            accumulator_info,
            fetcher.clone(),
            accumulator_snapshot.clone(),
            local_store.clone(),
        ),
        2,
        max_retry_times,
        delay_milliseconds_on_error,
        BlockCollector::new_with_handle(
            current_block_info?.clone(),
            None,
            chain?,
            block_event_handle.clone(),
            network.clone(),
            skip_pow_verify_when_sync,
            accumulator_root,
            Some(dag.clone()),
        ),
        event_handle.clone(),
        ext_error_handle,
    )
    .generate();
    let (fut, handle) = sync_task.with_handle();
    match fut.await {
        anyhow::Result::Ok(block_chain) => {
            return anyhow::Result::Ok(block_chain);
        }
        Err(error) => {
            return Err(anyhow!(error));
        }
    };
}

pub fn sync_dag_full_task(
    local_accumulator_info: AccumulatorInfo,
    target_accumulator_info: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_store: Arc<dyn AccumulatorTreeStore>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    local_store: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    vm_metrics: Option<VMMetrics>,
    connector_service: ServiceRef<BlockConnectorService>,
    network: NetworkServiceRef,
    skip_pow_verify_when_sync: bool,
    dag: Arc<Mutex<BlockDAG>>,
    block_chain_service: ServiceRef<BlockConnectorService>,
) -> anyhow::Result<(
    BoxFuture<'static, anyhow::Result<BlockChain, TaskError>>,
    TaskHandle,
    Arc<TaskEventCounterHandle>,
)> {
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let task_event_handle = event_handle.clone();
    let all_fut = async move {
        let ancestor = find_dag_ancestor_task(
            local_accumulator_info.clone(),
            target_accumulator_info.clone(),
            fetcher.clone(),
            accumulator_store.clone(),
            accumulator_snapshot.clone(),
            task_event_handle.clone(),
        )
        .await
        .map_err(|err| TaskError::BreakError(anyhow!(err)))?;

        let (start_index, accumulator) = sync_accumulator(
            ancestor,
            target_accumulator_info,
            fetcher.clone(),
            accumulator_store.clone(),
            accumulator_snapshot.clone(),
        )
        .await
        .map_err(|err| TaskError::BreakError(anyhow!(err)))?;

        let block_chain = sync_dag_block(
            start_index,
            accumulator,
            fetcher.clone(),
            accumulator_snapshot.clone(),
            local_store.clone(),
            time_service.clone(),
            connector_service.clone(),
            network,
            skip_pow_verify_when_sync,
            dag.clone(),
            block_chain_service.clone(),
            vm_metrics,
        )
        .await
        .map_err(|err| TaskError::BreakError(anyhow!(err)))?;
        return anyhow::Result::Ok(block_chain);
    };

    let task = TaskFuture::new(all_fut.boxed());
    let (fut, handle) = task.with_handle();
    Ok((fut, handle, event_handle))
}
