use std::{sync::atomic::AtomicU64, time::Duration};

use anyhow::bail;
use starcoin_logger::prelude::debug;
use tokio::sync::RwLock;

#[derive(Debug)]
enum WorkerSchedulerState {
    Inactive,
    Active,
}

#[derive(Debug)]
pub struct WorkerScheduler {
    state: RwLock<WorkerSchedulerState>,
    worker_count: AtomicU64,
}

impl Default for WorkerScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerScheduler {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(WorkerSchedulerState::Inactive),
            worker_count: AtomicU64::new(0),
        }
    }

    pub async fn tell_worker_to_stop(&self) {
        let mut state = self.state.write().await;
        *state = WorkerSchedulerState::Inactive;
    }

    pub async fn tell_worker_to_start(&self) {
        let mut state = self.state.write().await;
        *state = WorkerSchedulerState::Active;
    }

    pub async fn check_if_stop(&self) -> bool {
        let state = self.state.read().await;
        match *state {
            WorkerSchedulerState::Inactive => true,
            WorkerSchedulerState::Active => false,
        }
    }

    pub fn check_worker_count(&self) -> u64 {
        self.worker_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub fn worker_exits(&self) {
        self.worker_count
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn worker_start(&self) {
        self.worker_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub async fn wait_for_worker(&self) -> anyhow::Result<()> {
        const MAX_ATTEMPTS: u32 = 150;
        const INITIAL_DELAY: Duration = Duration::from_secs(30);
        let mut delay = INITIAL_DELAY;
        let mut attempts: u32 = 0;
        loop {
            if 0 == self.check_worker_count() {
                break;
            }
            attempts = attempts.saturating_add(1);
            if attempts >= MAX_ATTEMPTS {
                bail!("Timeout waiting for workers to exit");
            }
            tokio::task::yield_now().await;
            debug!("waiting for worker to exit, attempt {}", attempts);
            tokio::time::sleep(delay).await;
            delay = std::cmp::min(delay.saturating_mul(2), Duration::from_secs(60 * 60 * 2));
        }

        anyhow::Ok(())
    }
}
