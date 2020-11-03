// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use log::info;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

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

    pub fn item(&self) -> u64 {
        self.item_counter.load(Ordering::Relaxed)
    }

    pub fn percent(&self) -> Option<f64> {
        self.total_items
            .map(|total_items| (self.item() as f64 / total_items as f64) * 100f64)
    }

    pub fn get_report(&self) -> CounterReport {
        CounterReport {
            task_name: self.task_name.clone(),
            sub_task: self.sub_task(),
            error: self.error(),
            ok: self.ok(),
            retry: self.retry(),
            item: self.item(),
            percent: self.percent(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CounterReport {
    pub task_name: String,
    pub sub_task: u64,
    pub error: u64,
    pub ok: u64,
    pub retry: u64,
    pub item: u64,
    pub percent: Option<f64>,
}

impl std::fmt::Display for CounterReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task {} sub_task:{}, error:{}, ok:{}, retry:{}, processed items:{}",
            self.task_name, self.sub_task, self.error, self.ok, self.retry, self.item
        )?;
        if let Some(percent) = self.percent {
            write!(f, ", percent: {:.2}%", percent)?;
        }
        writeln!(f)
    }
}

#[derive(Default)]
pub struct TaskEventCounterHandle {
    current_counter: Mutex<Option<TaskEventCounter>>,
    previous_counters: Mutex<Vec<TaskEventCounter>>,
}

impl TaskEventCounterHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_reports(&self) -> Vec<CounterReport> {
        let mut reports = self
            .previous_counters
            .lock()
            .unwrap()
            .iter()
            .map(|counter| counter.get_report())
            .collect::<Vec<_>>();
        if let Some(counter) = self.current_counter.lock().unwrap().take() {
            reports.push(counter.get_report());
        }
        reports
    }

    pub fn get_report(&self) -> Option<CounterReport> {
        self.current_counter
            .lock()
            .unwrap()
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
            .unwrap()
            .replace(TaskEventCounter::new(name, total_items));
        if let Some(pre_counter) = pre_counter {
            self.previous_counters.lock().unwrap().push(pre_counter);
        }
    }

    fn on_sub_task(&self) {
        if let Some(counter) = self.current_counter.lock().unwrap().as_ref() {
            counter.sub_task_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_error(&self) {
        if let Some(counter) = self.current_counter.lock().unwrap().as_ref() {
            counter.error_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_ok(&self) {
        if let Some(counter) = self.current_counter.lock().unwrap().as_ref() {
            counter.ok_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_retry(&self) {
        if let Some(counter) = self.current_counter.lock().unwrap().as_ref() {
            counter.retry_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_item(&self) {
        if let Some(counter) = self.current_counter.lock().unwrap().as_ref() {
            counter.item_counter.fetch_add(1, Ordering::Release);
        }
    }

    fn on_finish(&self, task_name: String) {
        info!("{} finished.", task_name);
        if let Some(current_counter) = self.current_counter.lock().unwrap().as_ref() {
            info!("{} ", current_counter.get_report());
        }
    }
}
