// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::format_err;
use log::info;
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time is before the UNIX_EPOCH")
        .as_secs()
}

pub trait TaskEventHandle: Send + Sync {
    fn on_start(&self, task_name: String, total_items: Option<u64>);

    fn on_sub_task(&self);

    fn on_error(&self);

    fn on_ok(&self);

    fn on_retry(&self);

    fn on_item(&self);

    fn on_finish(&self, task_name: String);
}

#[derive(Debug)]
pub struct TaskEventCounter {
    start_seconds: u64,
    task_name: String,
    total_items: Option<u64>,
    sub_task_counter: AtomicU64,
    error_counter: AtomicU64,
    ok_counter: AtomicU64,
    retry_counter: AtomicU64,
    item_counter: AtomicU64,
}

impl TaskEventCounter {
    pub fn new(task_name: String, total_items: Option<u64>) -> Self {
        Self {
            start_seconds: now_seconds(),
            task_name,
            total_items,
            sub_task_counter: AtomicU64::new(0),
            error_counter: AtomicU64::new(0),
            ok_counter: AtomicU64::new(0),
            retry_counter: AtomicU64::new(0),
            item_counter: AtomicU64::new(0),
        }
    }

    pub fn sub_task(&self) -> u64 {
        self.sub_task_counter.load(Ordering::Relaxed)
    }

    pub fn error(&self) -> u64 {
        self.error_counter.load(Ordering::Relaxed)
    }

    pub fn ok(&self) -> u64 {
        self.ok_counter.load(Ordering::Relaxed)
    }

    pub fn retry(&self) -> u64 {
        self.retry_counter.load(Ordering::Relaxed)
    }

    pub fn processed_items(&self) -> u64 {
        self.item_counter.load(Ordering::Relaxed)
    }

    pub fn use_seconds(&self) -> u64 {
        now_seconds() - self.start_seconds
    }

    pub fn get_report(&self) -> TaskProgressReport {
        TaskProgressReport::new(
            self.task_name.clone(),
            self.sub_task(),
            self.error(),
            self.ok(),
            self.retry(),
            self.total_items,
            self.processed_items(),
            self.use_seconds(),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TaskProgressReport {
    pub task_name: String,
    pub sub_task: u64,
    pub error: u64,
    pub ok: u64,
    pub retry: u64,
    pub total_items: Option<u64>,
    pub processed_items: u64,
    pub use_seconds: u64,
    pub percent: Option<f64>,
}

impl TryFrom<Vec<TaskProgressReport>> for TaskProgressReport {
    type Error = anyhow::Error;
    fn try_from(reports: Vec<TaskProgressReport>) -> Result<Self, Self::Error> {
        let task_name = reports
            .as_slice()
            .first()
            .ok_or_else(|| format_err!("Reports can not be empty.",))?
            .task_name
            .clone();
        let mut sub_task: u64 = 0;
        let mut error: u64 = 0;
        let mut ok: u64 = 0;
        let mut retry: u64 = 0;
        let mut total_items: u64 = 0;
        let mut processed_items: u64 = 0;
        let mut use_seconds: u64 = 0;
        for report in reports.into_iter() {
            sub_task += report.sub_task;
            error += report.error;
            ok += report.ok;
            retry += report.retry;
            if let Some(items) = report.total_items {
                total_items += items;
            }
            processed_items += report.processed_items;
            use_seconds += report.use_seconds;
        }
        Ok(TaskProgressReport {
            task_name,
            sub_task,
            error,
            ok,
            retry,
            total_items: if total_items == 0 {
                None
            } else {
                Some(total_items)
            },
            processed_items,
            use_seconds,
            percent: None,
        })
    }
}

impl TaskProgressReport {
    pub fn new(
        task_name: String,
        sub_task: u64,
        error: u64,
        ok: u64,
        retry: u64,
        total_items: Option<u64>,
        processed_items: u64,
        use_seconds: u64,
    ) -> Self {
        Self {
            task_name,
            sub_task,
            error,
            ok,
            retry,
            total_items,
            processed_items,
            use_seconds,
            percent: total_items
                .map(|total_items| (processed_items as f64 / total_items as f64) * 100f64),
        }
    }

    pub fn fix_percent(&mut self, total: u64) {
        self.total_items = Some(total);
        self.percent = Some((self.processed_items as f64 / total as f64) * 100f64);
    }
}
impl std::fmt::Display for TaskProgressReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task {} sub_task:{}, error:{}, ok:{}, retry:{}, processed items:{}, use_seconds: {}",
            self.task_name,
            self.sub_task,
            self.error,
            self.ok,
            self.retry,
            self.processed_items,
            self.use_seconds,
        )?;
        if let (Some(total_items), Some(percent)) = (self.total_items, self.percent) {
            write!(
                f,
                ", total_items: {}, percent: {:.2}%",
                total_items, percent
            )?;
        }
        writeln!(f)
    }
}

#[derive(Default, Debug)]
pub struct TaskEventCounterHandle {
    current_counter: Mutex<Option<TaskEventCounter>>,
    previous_counters: Mutex<Vec<TaskEventCounter>>,
}

impl TaskEventCounterHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_reports(&self) -> Vec<TaskProgressReport> {
        let mut reports = self
            .previous_counters
            .lock()
            .iter()
            .map(|counter| counter.get_report())
            .collect::<Vec<_>>();
        if let Some(counter) = self.current_counter.lock().as_ref().take() {
            reports.push(counter.get_report());
        }
        reports
    }

    pub fn total_report(&self) -> Option<TaskProgressReport> {
        if let Some(current_counter) = self.current_counter.lock().as_ref().take() {
            let task_name = current_counter.task_name.clone();
            let mut reports = self
                .previous_counters
                .lock()
                .iter()
                .filter(|task| task.task_name == task_name)
                .map(|counter| counter.get_report())
                .collect::<Vec<_>>();
            reports.push(current_counter.get_report());
            if let Ok(report) = TaskProgressReport::try_from(reports) {
                return Some(report);
            }
        };
        None
    }

    pub fn get_report(&self) -> Option<TaskProgressReport> {
        self.current_counter
            .lock()
            .as_ref()
            .map(|counter| counter.get_report())
    }
}

impl TaskEventHandle for TaskEventCounterHandle {
    fn on_start(&self, name: String, total_items: Option<u64>) {
        info!("{} started", name);
        let pre_counter = self
            .current_counter
            .lock()
            .replace(TaskEventCounter::new(name, total_items));
        if let Some(pre_counter) = pre_counter {
            self.previous_counters.lock().push(pre_counter);
        }
    }

    fn on_sub_task(&self) {
        if let Some(counter) = self.current_counter.lock().as_ref() {
            counter.sub_task_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_error(&self) {
        if let Some(counter) = self.current_counter.lock().as_ref() {
            counter.error_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_ok(&self) {
        if let Some(counter) = self.current_counter.lock().as_ref() {
            counter.ok_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_retry(&self) {
        if let Some(counter) = self.current_counter.lock().as_ref() {
            counter.retry_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_item(&self) {
        if let Some(counter) = self.current_counter.lock().as_ref() {
            counter.item_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_finish(&self, task_name: String) {
        if let Some(current_counter) = self.current_counter.lock().as_ref() {
            info!(
                "{} finished, report: {}",
                task_name,
                current_counter.get_report()
            );
        }
    }
}
