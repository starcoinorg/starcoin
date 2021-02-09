// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{TaskError, TaskEventHandle};
use anyhow::{Error, Result};
use async_std::task::JoinHandle;
use futures::channel::mpsc::{channel, Sender};
use futures::task::{Context, Poll};
use futures::{Sink, StreamExt};
use log::debug;
use pin_project::pin_project;
use pin_utils::core_reexport::option::Option::Some;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub enum CollectorState {
    /// Collector is enough, do not feed more item, finish task.
    Enough,
    /// Collector is need more item.
    Need,
}

pub trait TaskResultCollector<Item>: std::marker::Send + Unpin {
    type Output: std::marker::Send;

    fn collect(&mut self, item: Item) -> Result<CollectorState>;
    fn finish(self) -> Result<Self::Output>;
}

impl<Item, F> TaskResultCollector<Item> for F
where
    F: FnMut(Item) -> Result<()>,
    F: std::marker::Send + Unpin,
{
    type Output = ();

    fn collect(&mut self, item: Item) -> Result<CollectorState> {
        (self)(item)?;
        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(())
    }
}

impl<Item> TaskResultCollector<Item> for Vec<Item>
where
    Item: std::marker::Send + Unpin,
{
    type Output = Self;

    fn collect(&mut self, item: Item) -> Result<CollectorState> {
        self.push(item);
        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self)
    }
}

#[derive(Clone)]
pub struct CounterCollector {
    counter: Arc<AtomicU64>,
    max: u64,
}

impl Default for CounterCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterCollector {
    pub fn new() -> Self {
        Self::new_with_counter(Arc::new(AtomicU64::default()))
    }

    pub fn new_with_counter(counter: Arc<AtomicU64>) -> Self {
        Self {
            counter,
            max: u64::max_value(),
        }
    }

    pub fn new_with_max(max: u64) -> Self {
        Self {
            counter: Arc::new(AtomicU64::default()),
            max,
        }
    }
}

impl<Item> TaskResultCollector<Item> for CounterCollector
where
    Item: std::marker::Send + Unpin,
{
    type Output = u64;

    fn collect(&mut self, _item: Item) -> Result<CollectorState, Error> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        let count = self.counter.load(Ordering::SeqCst);
        debug!("collect item, count: {}", count);
        if count >= self.max {
            Ok(CollectorState::Enough)
        } else {
            Ok(CollectorState::Need)
        }
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self.counter.load(Ordering::SeqCst))
    }
}

#[derive(Error, Debug)]
pub(crate) enum SinkError {
    #[error("{0:?}")]
    StreamTaskError(TaskError),
    #[error("Collector is enough.")]
    CollectorEnough,
}

impl SinkError {
    pub fn map_result(result: Result<(), SinkError>) -> Result<(), TaskError> {
        match result {
            Err(err) => match err {
                SinkError::StreamTaskError(err) => Err(err),
                SinkError::CollectorEnough => Ok(()),
            },
            Ok(()) => Ok(()),
        }
    }
}

#[pin_project]
pub(crate) struct FutureTaskSink<Item, Output> {
    #[pin]
    sender: Sender<Item>,
    #[pin]
    task_handle: JoinHandle<Result<Output, TaskError>>,
}

impl<Item, Output> FutureTaskSink<Item, Output> {
    pub fn new<C>(
        mut collector: C,
        buffer_size: usize,
        event_handle: Arc<dyn TaskEventHandle>,
    ) -> Self
    where
        Item: Send + 'static,
        Output: Send + 'static,
        C: TaskResultCollector<Item, Output = Output> + 'static,
    {
        let (sender, receiver) = channel(buffer_size);
        let task_handle = async_std::task::spawn(async move {
            let mut receiver = receiver.fuse();
            while let Some(item) = receiver.next().await {
                event_handle.on_item();
                let collector_state = collector.collect(item).map_err(TaskError::map)?;
                match collector_state {
                    CollectorState::Enough => break,
                    CollectorState::Need => {
                        //continue
                    }
                }
            }
            collector.finish().map_err(TaskError::map)
        });
        Self {
            sender,
            task_handle,
        }
    }

    pub async fn wait_output(self) -> Result<Output, TaskError> {
        self.task_handle.await
    }
}

impl<Item, Output> Sink<Item> for FutureTaskSink<Item, Output> {
    type Error = SinkError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.project();
        //if the sender is disconnect, means the task is finished, so map error to CollectorEnough, and close the sink.
        this.sender
            .poll_ready(cx)
            .map_err(|_| SinkError::CollectorEnough)
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();
        //ignore sender error, because if send error, may bean task is finished
        this.sender
            .start_send(item)
            .map_err(|_| SinkError::CollectorEnough)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.as_mut().project();
        this.sender
            .poll_flush(cx)
            .map_err(|_| SinkError::CollectorEnough)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        debug!("FutureTaskSink poll_close");
        let this = self.as_mut().project();
        this.sender
            .poll_close(cx)
            .map_err(|_| SinkError::CollectorEnough)
    }
}
