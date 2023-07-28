use std::sync::Arc;

use starcoin_accumulator::{
    accumulator_info::AccumulatorInfo, node::AccumulatorStoreType, AccumulatorTreeStore,
    MerkleAccumulator,
};
use starcoin_service_registry::ServiceContext;
use starcoin_storage::{flexi_dag::SyncFlexiDagSnapshotStorage, Storage, Store, SyncFlexiDagStore};
use stream_task::{Generator, TaskEventCounterHandle, TaskGenerator};

use crate::{sync::SyncService, verified_rpc_client::VerifiedRpcClient};

use super::{
    sync_find_ancestor_task::{AncestorCollector, FindAncestorTask},
    ExtSyncTaskErrorHandle,
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

pub fn sync_dag_full_task(
    local_accumulator_info: AccumulatorInfo,
    target_accumulator_info: AccumulatorInfo,
    fetcher: Arc<VerifiedRpcClient>,
    accumulator_store: Arc<dyn AccumulatorTreeStore>,
    accumulator_snapshot: Arc<SyncFlexiDagSnapshotStorage>,
) {
    async move {
        let ancestor = find_dag_ancestor_task(
            local_accumulator_info,
            target_accumulator_info,
            fetcher,
            accumulator_store,
            accumulator_snapshot,
        );
    };
}
