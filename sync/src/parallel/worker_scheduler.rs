use std::{sync::atomic::AtomicU64, time::Duration};

use tokio::sync::RwLock;

enum WorkerSchedulerState {
    Inactive,
    Active,
}

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

    pub async fn check_worker_count(&self) -> u64 {
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

    pub async fn wait_for_worker(&self) {
        loop {
            if 0 == self.check_worker_count().await {
                break;
            } else {
                tokio::task::yield_now().await;
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}
