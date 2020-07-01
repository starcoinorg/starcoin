mod block_connector;
pub mod block_sync;
mod download;
pub mod helper;
mod process;
pub mod state_sync;
mod sync;
mod sync_metrics;
mod sync_task;
mod txn_sync;
pub use download::Downloader;
pub use process::ProcessActor;
use std::time::Duration;
pub use sync::SyncActor;

use crypto::HashValue;
use dyn_clone::DynClone;

pub(crate) const DELAY_TIME: u64 = 15;

pub(crate) fn do_duration(delay: u64) -> Duration {
    Duration::from_secs(delay)
}

#[async_trait::async_trait]
pub trait StateSyncReset: DynClone + Send + Sync {
    async fn reset(&self, state_root: HashValue, block_accumulator_root: HashValue);
}
