// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::future::BoxFuture;
use std::any::type_name;
use std::fmt::Debug;
use thiserror::Error;

mod collector;
mod event;
mod generator;
mod task_stream;

pub use collector::{CollectorState, CounterCollector, TaskResultCollector};
pub use event::{TaskEventCounter, TaskEventCounterHandle, TaskEventHandle, TaskProgressReport};
pub use generator::{AndThenGenerator, Generator, TaskFuture, TaskGenerator, TaskHandle};
pub use task_stream::{CustomErrorHandle, DefaultCustomErrorHandle};

#[derive(Error, Debug)]
pub enum TaskError {
    /// directly break the task, do not retry.
    #[error("Task failed because break error: {0:?}")]
    BreakError(anyhow::Error),
    #[error(
        "Task failed because maximum number of retry attempts({0}) reached, last error: {1:?}"
    )]
    RetryLimitReached(usize, anyhow::Error),
    #[error("Task has been canceled.")]
    Canceled,
}

impl TaskError {
    pub fn map(error: anyhow::Error) -> Self {
        match error.downcast::<Self>() {
            Ok(task_err) => task_err,
            Err(err) => TaskError::BreakError(err),
        }
    }

    pub fn is_canceled(&self) -> bool {
        matches!(self, Self::Canceled)
    }

    pub fn is_break_error(&self) -> bool {
        matches!(self, Self::BreakError(_))
    }

    pub fn is_retry_limit_reached(&self) -> bool {
        matches!(self, Self::RetryLimitReached(_, _))
    }
}

pub trait TaskState: Sized + Clone + std::marker::Unpin + std::marker::Send {
    type Item: Debug + std::marker::Send;

    fn task_name() -> &'static str {
        type_name::<Self>()
    }
    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>>;
    fn next(&self) -> Option<Self>;
    fn total_items(&self) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests;
