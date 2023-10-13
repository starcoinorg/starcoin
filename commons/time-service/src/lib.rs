// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, SystemTime},
};

// Gives the duration since the Unix epoch, notice the expect.
pub fn duration_since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is before the UNIX_EPOCH")
}

/// A generic service for providing time related operations (e.g., returning the current time and
/// sleeping).
pub trait TimeService: Send + Sync + Debug {
    ///Adjust local time by on chain time.
    fn adjust(&self, milliseconds: u64);
    /// Returns the current time since the UNIX_EPOCH in seconds as a u64.
    fn now_secs(&self) -> u64;
    /// Returns the current time since the UNIX_EPOCH in milliseconds as a u64.
    fn now_millis(&self) -> u64;
    /// Sleeps the calling thread for (at least) the specified number of milliseconds. This call may
    /// sleep longer than specified, never less.
    fn sleep(&self, millis: u64);

    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum TimeServiceType {
    RealTimeService,
    MockTimeService,
}

impl TimeServiceType {
    pub fn new_time_service(self) -> Arc<dyn TimeService> {
        match self {
            Self::RealTimeService => Arc::new(RealTimeService::new()),
            Self::MockTimeService => Arc::new(MockTimeService::new_with_value(1)),
        }
    }
}

/// A real-time TimeService
#[derive(Default)]
pub struct RealTimeService;

impl RealTimeService {
    pub fn new() -> Self {
        Self {}
    }
}

impl std::fmt::Debug for RealTimeService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.now_millis())
    }
}

impl TimeService for RealTimeService {
    fn adjust(&self, milliseconds: u64) {
        let now = self.now_millis();
        if milliseconds > now && milliseconds - now > 150000 {
            warn!(
                "Local time {} is behind on chain time {} too much.",
                now / 1000,
                milliseconds / 1000
            );
        }
    }

    fn now_secs(&self) -> u64 {
        duration_since_epoch().as_secs()
    }

    fn now_millis(&self) -> u64 {
        duration_since_epoch().as_millis() as u64
    }

    fn sleep(&self, millis: u64) {
        sleep(Duration::from_millis(millis));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A mock-time TimeService
#[derive(Clone, Default)]
pub struct MockTimeService {
    now: Arc<AtomicU64>,
}

impl std::fmt::Debug for MockTimeService {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.now_millis())
    }
}

impl MockTimeService {
    pub fn new() -> Self {
        Self::new_with_value(0)
    }

    pub fn new_with_value(init_value: u64) -> Self {
        Self {
            now: Arc::new(AtomicU64::new(init_value)),
        }
    }

    #[cfg(test)]
    pub fn increment(&self) {
        self.now.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_by(&self, value: u64) {
        self.now.fetch_add(value, Ordering::Relaxed);
    }

    pub fn set(&self, value: u64) {
        self.now.store(value, Ordering::Relaxed)
    }
}

impl TimeService for MockTimeService {
    fn adjust(&self, milliseconds: u64) {
        if milliseconds >= self.now_millis() {
            // add 1 to ensure local time is greater than on chain time.
            let time = milliseconds + 1;
            info!("Adjust MockTimeService by time: {}", time);
            self.set(time)
        }
    }

    fn now_secs(&self) -> u64 {
        self.now_millis() / 1000
    }

    fn now_millis(&self) -> u64 {
        self.now.load(Ordering::Relaxed)
    }

    fn sleep(&self, millis: u64) {
        self.increment_by(millis);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct DagBlockTimeWindowService {
    time_service: Arc<dyn TimeService>,

    time_window: u64,
}

pub enum TimeWindowResult {
    InTimeWindow,
    BeforeTimeWindow,
    AfterTimeWindow,
}

impl TimeService for DagBlockTimeWindowService {
    fn adjust(&self, milliseconds: u64) {
        self.time_service.adjust(milliseconds)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn now_secs(&self) -> u64 {
        self.time_service.now_secs()
    }

    fn now_millis(&self) -> u64 {
        self.time_service.now_millis()
    }

    fn sleep(&self, millis: u64) {
        self.time_service.sleep(millis)
    }
}

impl DagBlockTimeWindowService {
    pub fn new(time_windows: u64, time_service: Arc<dyn TimeService>) -> Self {
        Self {
            time_service: time_service.clone(),
            time_window: time_windows,
        }
    }

    pub fn is_in_time_window(&self, block_timestamp: u64) -> TimeWindowResult {
        let now = self.time_service.now_millis();
        let start_time = now - now % self.time_window;
        let end_time = start_time + self.time_window;
        if (start_time..end_time).contains(&block_timestamp) {
            TimeWindowResult::InTimeWindow
        } else if block_timestamp < start_time {
            TimeWindowResult::BeforeTimeWindow
        } else {
            TimeWindowResult::AfterTimeWindow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_real_time() {
        test_time_service(&RealTimeService::new());
    }

    #[test]
    fn verify_mock_time() {
        let service = MockTimeService::new();

        assert_eq!(service.now_millis(), 0);
        service.increment();
        assert_eq!(service.now_millis(), 1);
    }

    #[test]
    fn test_sleep() {
        // This time shouldn't be too large because it actually sleeps the testing thread when
        // using the RealTimeService!
        let sleep_time = 1;

        // Test real service
        let service = RealTimeService::new();
        verify_sleep(&service, sleep_time);

        // Test mock service
        let service = MockTimeService::new();
        verify_sleep(&service, sleep_time);
    }

    fn verify_sleep<T: TimeService>(service: &T, sleep_time: u64) {
        let current_time = service.now_millis();
        service.sleep(sleep_time);

        assert!(service.now_millis() >= current_time + sleep_time);
    }

    fn test_time_service<T: TimeService>(service: &T) {
        service.now_secs();
    }
}
