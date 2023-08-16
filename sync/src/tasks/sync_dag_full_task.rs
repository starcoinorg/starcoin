use std::sync::Arc;

use anyhow::{bail, Ok};
use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, Accumulator, AccumulatorTreeStore, MerkleAccumulator,
};
use starcoin_chain::BlockChain;
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, Store};
use starcoin_time_service::TimeService;
use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

use crate::verified_rpc_client::VerifiedRpcClient;

use super::{
    sync_dag_accumulator_task::{SyncDagAccumulatorCollector, SyncDagAccumulatorTask},
    sync_dag_block_task::{SyncDagBlockCollector, SyncDagBlockTask},
    sync_find_ancestor_task::{AncestorCollector, FindAncestorTask},
    BlockLocalStore, ExtSyncTaskErrorHandle,
};

pub fn find_dag_ancestor_task(
    local_accumulator_info: AccumulatorInfo,
    target_accumulator_info: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_store: Arc<dyn AccumulatorTreeStore>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
) -> anyhow::Result<AccumulatorInfo> {
    let max_retry_times = 10; // in startcoin, it is in config
    let delay_milliseconds_on_error = 100;

    let event_handle = Arc::new(TaskEventCounterHandle::new());

    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    let find_ancestor_task = async_std::task::spawn(async move {
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
                println!("receive ancestor {:?}", ancestor);
                return Ok(ancestor);
            }
            Err(error) => {
                println!("an error happened: {}", error.to_string());
                return Err(error.into());
            }
        }
    });
    return async_std::task::block_on(find_ancestor_task);
}

fn sync_accumulator(
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

    let sync = async_std::task::spawn(async move {
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
                println!(
                    "start index: {}, full accumulator info is {:?}",
                    start_index,
                    full_accumulator.get_info()
                );
                return anyhow::Result::Ok((start_index, full_accumulator));
            }
            Err(error) => {
                println!("an error happened: {}", error.to_string());
                return Err(error);
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
    });
    // return Ok(async_std::task::block_on(sync));
    match async_std::task::block_on(sync) {
        std::result::Result::Ok((index, accumulator)) => {
            println!("sync accumulator success");
            return Ok((index, accumulator));
        }
        Err(error) => {
            println!("sync accumulator error: {}", error.to_string());
            Err(error.into())
        }
    }
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
        .get_last_tips()?
        .expect("tips should not be None");
    snapshot.sort();
    Ok(snapshot
        .iter()
        .last()
        .expect("last block id should not be None")
        .clone())
}

fn sync_dag_block(
    start_index: u64,
    accumulator: MerkleAccumulator,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
    local_store: Arc<dyn Store>,
    time_service: Arc<dyn TimeService>,
    vm_metrics: Option<VMMetrics>,
) -> anyhow::Result<()> {
    let max_retry_times = 10; // in startcoin, it is in config
    let delay_milliseconds_on_error = 100;
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let ext_error_handle = Arc::new(ExtSyncTaskErrorHandle::new(fetcher.clone()));

    let chain = BlockChain::new(
        time_service.clone(),
        get_start_block_id(&accumulator, start_index, local_store.clone())?,
        local_store.clone(),
        vm_metrics,
    )?;

    let sync = async_std::task::spawn(async move {
        let accumulator_info = accumulator.get_info();
        let sync_task = TaskGenerator::new(
            SyncDagBlockTask::new(
                accumulator,
                start_index.saturating_add(1),
                10,
                accumulator_info,
                fetcher.clone(),
                accumulator_snapshot.clone(),
                local_store.clone(),
            ),
            2,
            max_retry_times,
            delay_milliseconds_on_error,
            SyncDagBlockCollector::new(chain),
            event_handle.clone(),
            ext_error_handle,
        )
        .generate();
        let (fut, handle) = sync_task.with_handle();
        match fut.await {
            anyhow::Result::Ok(_) => {
                println!("finish to sync dag blocks");
                return anyhow::Result::Ok(());
            }
            Err(error) => {
                println!(
                    "an error happened when synchronizing the dag blocks: {}",
                    error.to_string()
                );
                return Err(error);
            }
        }
    });

    match async_std::task::block_on(sync) {
        std::result::Result::Ok(()) => {
            println!("sync dag blocks success");
            return Ok(());
        }
        Err(error) => {
            bail!("sync accumulator error: {}", error.to_string());
        }
    }
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
) -> anyhow::Result<()> {
    let ancestor = find_dag_ancestor_task(
        local_accumulator_info,
        target_accumulator_info.clone(),
        fetcher.clone(),
        accumulator_store.clone(),
        accumulator_snapshot.clone(),
    )?;

    match sync_accumulator(
        ancestor,
        target_accumulator_info,
        fetcher.clone(),
        accumulator_store.clone(),
        accumulator_snapshot.clone(),
    ) {
        anyhow::Result::Ok((start_index, accumultor)) => {
            return sync_dag_block(
                start_index,
                accumultor,
                fetcher.clone(),
                accumulator_snapshot.clone(),
                local_store.clone(),
                time_service.clone(),
                vm_metrics,
            );
        }
        Err(error) => {
            bail!("sync accumulator error: {}", error.to_string());
        }
    }
}
