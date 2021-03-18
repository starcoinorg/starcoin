// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{TaskError, TaskEventHandle, TaskState};
use anyhow::{Error, Result};
use futures::{
    future::BoxFuture,
    task::{Context, Poll},
    FutureExt, Stream,
};
use futures_retry::{ErrorHandler, FutureRetry, RetryPolicy};
use log::debug;
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub trait CustomErrorHandle: Send + Sync {
    fn handle(&self, error: Error) {
        debug!("default custom error handle: {:?}", error);
    }
}

pub struct DefaultCustomErrorHandle;

impl CustomErrorHandle for DefaultCustomErrorHandle {}

#[derive(Clone)]
pub struct TaskErrorHandle {
    event_handle: Arc<dyn TaskEventHandle>,
    max_retry_times: u64,
    delay_milliseconds: u64,
    custom_error_handle: Arc<dyn CustomErrorHandle>,
}

impl TaskErrorHandle {
    pub fn new(
        event_handle: Arc<dyn TaskEventHandle>,
        max_retry_times: u64,
        delay_milliseconds: u64,
        custom_error_handle: Arc<dyn CustomErrorHandle>,
    ) -> Self {
        Self {
            event_handle,
            max_retry_times,
            delay_milliseconds,
            custom_error_handle,
        }
    }
}

impl ErrorHandler<anyhow::Error> for TaskErrorHandle {
    type OutError = TaskError;

    fn handle(&mut self, attempt: usize, error: Error) -> RetryPolicy<Self::OutError> {
        if attempt > 1 {
            self.event_handle.on_retry();
        } else {
            self.event_handle.on_error();
        }
        match error.downcast::<TaskError>() {
            Ok(task_err) => match task_err {
                TaskError::BreakError(e) => RetryPolicy::ForwardError(TaskError::BreakError(e)),
                TaskError::RetryLimitReached(attempt, error) => {
                    RetryPolicy::ForwardError(TaskError::RetryLimitReached(attempt + 1, error))
                }
                TaskError::Canceled => RetryPolicy::ForwardError(TaskError::Canceled),
            },
            Err(err) => {
                debug!("Task error: {:?}, attempt: {}", err, attempt);
                if attempt as u64 > self.max_retry_times {
                    RetryPolicy::ForwardError(TaskError::RetryLimitReached(attempt, err))
                } else {
                    self.custom_error_handle.handle(err);
                    if self.delay_milliseconds == 0 {
                        RetryPolicy::Repeat
                    } else {
                        RetryPolicy::WaitRetry(Duration::from_millis(
                            self.delay_milliseconds * attempt as u64,
                        ))
                    }
                }
            }
        }
    }

    fn ok(&mut self, _attempt: usize) {
        self.event_handle.on_ok()
    }
}

#[pin_project]
pub struct FutureTaskStream<S>
where
    S: TaskState,
{
    state: Option<S>,
    max_retry_times: u64,
    delay_milliseconds: u64,
    event_handle: Arc<dyn TaskEventHandle>,
    custom_error_handle: Arc<dyn CustomErrorHandle>,
}

impl<S> FutureTaskStream<S>
where
    S: TaskState,
{
    pub fn new(
        state: S,
        max_retry_times: u64,
        delay_milliseconds: u64,
        event_handle: Arc<dyn TaskEventHandle>,
        custom_error_handle: Arc<dyn CustomErrorHandle>,
    ) -> Self {
        Self {
            state: Some(state),
            max_retry_times,
            delay_milliseconds,
            event_handle,
            custom_error_handle,
        }
    }
}

impl<S> Stream for FutureTaskStream<S>
where
    S: TaskState + 'static,
{
    type Item = BoxFuture<'static, Result<Vec<S::Item>, TaskError>>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.state {
            Some(state) => {
                this.event_handle.on_sub_task();
                let error_action = TaskErrorHandle::new(
                    this.event_handle.clone(),
                    *this.max_retry_times,
                    *this.delay_milliseconds,
                    this.custom_error_handle.clone(),
                );
                let state_to_factory = state.clone();

                let fut = async move {
                    let retry_fut = FutureRetry::new(
                        move || state_to_factory.clone().new_sub_task(),
                        error_action,
                    );
                    retry_fut
                        .map(|result| match result {
                            Ok((item, _attempt)) => Ok(item),
                            Err((e, _attempt)) => Err(e),
                        })
                        .await
                }
                .boxed();
                *this.state = state.next();
                Poll::Ready(Some(fut))
            }
            None => Poll::Ready(None),
        }
    }
}
