mod download;
mod get_txns_handler;
pub mod helper;
mod process;
pub mod state_sync;
mod sync;
mod sync_metrics;
mod txn_sync;

pub use download::Downloader;
pub use process::ProcessActor;
use std::time::Duration;
pub use sync::SyncActor;

pub(crate) const DELAY_TIME: u64 = 15;

pub(crate) fn do_duration(delay: u64) -> Duration {
    Duration::from_secs(delay)
}
