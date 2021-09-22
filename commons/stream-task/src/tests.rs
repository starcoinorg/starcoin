use super::*;
use crate::task_stream::FutureTaskStream;
use anyhow::format_err;
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;
use log::debug;
use pin_utils::core_reexport::time::Duration;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

struct MockTestConfig {
    max: u64,
    batch_size: u64,
    delay_time: u64,
    error_per_task: u64,
    break_at: Option<u64>,
    error_times: Mutex<HashMap<u64, AtomicU64>>,
}

impl MockTestConfig {
    pub fn new(
        max: u64,
        batch_size: u64,
        delay_time: u64,
        error_per_task: u64,
        break_at: Option<u64>,
    ) -> Self {
        Self {
            max,
            batch_size,
            delay_time,
            error_per_task,
            break_at,
            error_times: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_with_delay(max: u64, delay_time: u64) -> Self {
        Self::new(max, 1, delay_time, 0, None)
    }

    pub fn new_with_error(max: u64, error_per_task: u64) -> Self {
        Self::new(max, 1, 1, error_per_task, None)
    }

    pub fn new_with_max(max: u64) -> Self {
        Self::new(max, 1, 1, 0, None)
    }

    pub fn new_with_break(max: u64, error_per_task: u64, break_at: u64) -> Self {
        Self::new(max, 1, 1, error_per_task, Some(break_at))
    }

    pub fn new_with_batch(max: u64, batch_size: u64) -> Self {
        Self::new(max, batch_size, 1, 0, None)
    }
}

#[derive(Clone)]
struct MockTaskState {
    state: u64,
    config: Arc<MockTestConfig>,
}

impl MockTaskState {
    pub fn new(config: MockTestConfig) -> Self {
        Self {
            state: 0,
            config: Arc::new(config),
        }
    }
}

impl TaskState for MockTaskState {
    type Item = u64;

    fn task_name() -> &'static str {
        "MockTask"
    }

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            if let Some(break_at) = self.config.break_at {
                if self.state >= break_at {
                    return Err(TaskError::BreakError(format_err!(
                        "Break error at: {}",
                        self.state
                    ))
                    .into());
                }
            }
            if self.config.delay_time > 0 {
                Delay::new(Duration::from_millis(self.config.delay_time)).await;
            }
            if self.config.error_per_task > 0 {
                let mut error_times = self.config.error_times.lock().unwrap();
                let current_state_error_counter = error_times
                    .entry(self.state)
                    .or_insert_with(|| AtomicU64::new(0));
                let current_state_error_times =
                    current_state_error_counter.fetch_add(1, Ordering::Relaxed);
                if current_state_error_times <= self.config.error_per_task {
                    return Err(format_err!(
                        "return error for state: {}, error_times: {}",
                        self.state,
                        current_state_error_times
                    ));
                }
            }
            Ok((self.state..self.state + self.config.batch_size)
                .filter(|i| *i < self.config.max)
                .map(|i| i * 2)
                .collect())
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        if self.state >= self.config.max - 1 {
            None
        } else {
            let next = self.state + self.config.batch_size;
            Some(MockTaskState {
                state: next,
                config: self.config.clone(),
            })
        }
    }

    fn total_items(&self) -> Option<u64> {
        Some(self.config.max)
    }
}

#[stest::test]
async fn test_task_stream() {
    let max = 100;
    let config = MockTestConfig::new_with_max(max);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let task = FutureTaskStream::new(
        mock_state,
        0,
        0,
        event_handle,
        Arc::new(DefaultCustomErrorHandle),
    );
    let results = task.buffered(10).collect::<Vec<_>>().await;
    assert_eq!(results.len() as u64, max);
}

#[stest::test]
async fn test_counter_collector() {
    let max = 100;
    let config = MockTestConfig::new_with_max(max);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let result = TaskGenerator::new(
        mock_state.clone(),
        10,
        0,
        0,
        CounterCollector::new(),
        event_handle,
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate()
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), max);
}

#[stest::test]
async fn test_stream_task_batch() {
    let max = 100;
    for batch in 1..max {
        let config = MockTestConfig::new_with_batch(max, batch);
        let mock_state = MockTaskState::new(config);
        let event_handle = Arc::new(TaskEventCounterHandle::new());
        let result = TaskGenerator::new(
            mock_state.clone(),
            10,
            0,
            0,
            CounterCollector::new(),
            event_handle,
            Arc::new(DefaultCustomErrorHandle),
        )
        .generate()
        .await;
        assert!(result.is_ok(), "assert test batch {} fail.", batch);
        assert_eq!(result.unwrap(), max, "test batch {} fail.", batch);
    }
}

#[stest::test]
async fn test_vec_collector() {
    let max = 100;
    let config = MockTestConfig::new_with_max(max);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let result = TaskGenerator::new(
        mock_state,
        10,
        0,
        0,
        vec![],
        event_handle,
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate()
    .await;
    //println!("{:?}", result);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len() as u64, max);
}

#[stest::test]
async fn test_task_cancel() {
    let max = 100;
    let delay_time = 10;
    let config = MockTestConfig::new_with_delay(max, delay_time);
    let mock_state = MockTaskState::new(config);
    let counter = Arc::new(AtomicU64::new(0));
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let fut = TaskGenerator::new(
        mock_state.clone(),
        5,
        0,
        0,
        CounterCollector::new_with_counter(counter.clone()),
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let (fut, task_handle) = fut.with_handle();
    let join_handle = async_std::task::spawn(fut);
    Delay::new(Duration::from_millis(delay_time * 5)).await;
    assert!(!task_handle.is_done());
    task_handle.cancel();
    let result = join_handle.await;
    assert!(result.is_err());

    assert!(task_handle.is_done());

    let task_err = result.err().unwrap();
    assert!(task_err.is_canceled());
    let processed_messages = counter.load(Ordering::SeqCst);
    debug!("processed_messages before cancel: {}", processed_messages);
    assert!(processed_messages > 0 && processed_messages < max);

    let report = event_handle.get_reports().pop().unwrap();
    debug!("{}", report);
    assert!(report.processed_items > 0);
    assert!(report.processed_items < max);
}

#[stest::test]
async fn test_task_retry() {
    let max = 100;
    let max_retry_times = 5;
    let config = MockTestConfig::new_with_error(max, max_retry_times - 1);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let fut = TaskGenerator::new(
        mock_state.clone(),
        10,
        max_retry_times,
        1,
        CounterCollector::new(),
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let counter = fut.await.unwrap();
    assert_eq!(counter, max);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("{}", report);
    assert_eq!(report.processed_items, max);
}

#[stest::test]
async fn test_task_retry_fail() {
    let max = 100;
    let max_retry_times = 5;
    let counter = Arc::new(AtomicU64::new(0));

    let config = MockTestConfig::new_with_error(max, max_retry_times);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let fut = TaskGenerator::new(
        mock_state.clone(),
        10,
        max_retry_times,
        1,
        CounterCollector::new_with_counter(counter.clone()),
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let result = fut.await;
    assert!(result.is_err());
    let task_err = result.err().unwrap();
    assert!(task_err.is_retry_limit_reached());
    assert_eq!(counter.load(Ordering::SeqCst), 0);
    let report = event_handle.get_reports().pop().unwrap();
    debug!("{}", report);
    assert_eq!(report.ok, 0);
    assert_eq!(report.processed_items, 0);
}

#[stest::test]
async fn test_collector_error() {
    let max = 100;
    let config = MockTestConfig::new_with_max(max);
    let mock_state = MockTaskState::new(config);
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let result = TaskGenerator::new(
        mock_state,
        10,
        0,
        0,
        |item| {
            //println!("collect error for: {:?}", item);
            Err(format_err!("collect error for: {:?}", item))
        },
        event_handle,
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate()
    .await;
    assert!(result.is_err());
    let task_err = result.err().unwrap();
    assert!(task_err.is_break_error());
}

#[stest::test]
async fn test_break_error() {
    let max = 100;
    let break_at = 31;
    let max_retry_times = 5;
    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let counter = Arc::new(AtomicU64::new(0));
    let config = MockTestConfig::new_with_break(max, max_retry_times - 1, break_at);
    let mock_state = MockTaskState::new(config);

    let fut = TaskGenerator::new(
        mock_state.clone(),
        10,
        max_retry_times,
        1,
        CounterCollector::new_with_counter(counter.clone()),
        event_handle.clone(),
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate();
    let result = fut.await;
    assert!(result.is_err());
    let task_err = result.err().unwrap();
    assert!(task_err.is_break_error());
    assert!(
        (break_at as i32)
            .saturating_sub(counter.load(Ordering::SeqCst) as i32)
            .abs()
            <= 1,
        " break_at: {}, counter: {}",
        break_at,
        counter.load(Ordering::SeqCst)
    );

    let report = event_handle.get_reports().pop().unwrap();
    debug!("{}", report);
    assert!(report.processed_items > 0);
    assert!(report.processed_items < max);
}

#[stest::test]
async fn test_collect_enough() {
    let max = 100;
    let collector_max = 50;
    let config = MockTestConfig::new_with_max(max);
    let mock_state = MockTaskState::new(config);

    let event_handle = Arc::new(TaskEventCounterHandle::new());
    let result = TaskGenerator::new(
        mock_state.clone(),
        10,
        0,
        0,
        CounterCollector::new_with_max(collector_max),
        event_handle,
        Arc::new(DefaultCustomErrorHandle),
    )
    .generate()
    .await;
    //assert!(result.is_ok());
    assert_eq!(result.unwrap(), collector_max);
}
