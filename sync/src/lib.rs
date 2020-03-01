mod download;
mod pool;
mod process;
mod sync;
#[cfg(test)]
mod tests;

pub use download::DownloadActor;
pub use process::ProcessActor;
use std::time::Duration;
pub use sync::SyncActor;

pub(crate) const DELAY_TIME: u64 = 5;

pub(crate) fn do_duration(delay: u64) -> Duration {
    Duration::from_secs(delay)
}
