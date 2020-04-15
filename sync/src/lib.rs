mod download;
mod download_body;
mod download_header;
mod get_txns_handler;
pub mod helper;
mod pool;
mod process;
pub mod state_sync;
mod sync;
mod txn_sync;

use std::time::Duration;
pub use sync::SyncActor;

pub(crate) const DELAY_TIME: u64 = 15;

pub(crate) fn do_duration(delay: u64) -> Duration {
    Duration::from_secs(delay)
}
