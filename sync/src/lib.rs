mod block_connector;
pub mod block_sync;
mod download;
pub mod helper;
pub mod state_sync;
mod sync;
mod sync_metrics;
mod sync_task;
mod txn_sync;
pub use download::Downloader;
pub use sync::SyncActor;

use crypto::HashValue;
use dyn_clone::DynClone;

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(
        &self,
        state_root: HashValue,
        block_accumulator_root: HashValue,
        pivot_id: HashValue,
    );
}
