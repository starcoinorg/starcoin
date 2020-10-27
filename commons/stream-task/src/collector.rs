// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{TaskError, TaskEventHandle};
use anyhow::{Error, Result};
use futures::task::{Context, Poll};
use futures::Sink;
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub enum CollectorState {
    /// Collector is enough, do not feed more item.
    Enough,
    /// Collector is need more item.
    Need,
}

pub trait TaskResultCollector<Item>: std::marker::Send + Unpin {
    type Output: std::marker::Send;

    fn collect(self: Pin<&mut Self>, item: Item) -> Result<CollectorState>;
    fn finish(self) -> Result<Self::Output>;
}

impl<Item, F> TaskResultCollector<Item> for F
where
    F: FnMut(Item) -> Result<()>,
    F: std::marker::Send + Unpin,
{
    type Output = ();

    fn collect(self: Pin<&mut Self>, item: Item) -> Result<CollectorState> {
        self.get_mut()(item)?;
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

    fn collect(self: Pin<&mut Self>, item: Item) -> Result<CollectorState> {
        self.get_mut().push(item);
        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self)
    }
}

#[derive(Clone, Default)]
pub struct CounterCollector {
    counter: Arc<AtomicU64>,
}

impl CounterCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_counter(counter: Arc<AtomicU64>) -> Self {
        Self { counter }
    }
}

impl<Item> TaskResultCollector<Item> for CounterCollector
where
    Item: std::marker::Send + Unpin,
{
    type Output = u64;

    fn collect(self: Pin<&mut Self>, _item: Item) -> Result<CollectorState, Error> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(CollectorState::Need)
    }

    fn finish(self) -> Result<Self::Output> {
        Ok(self.counter.load(Ordering::SeqCst))
    }
}

#[derive(Error, Debug)]
pub(crate) enum SinkError {
    #[error("{0:?}")]
    StreamTaskError(TaskError),
    #[error("{0:?}")]
    CollectorError(anyhow::Error),
    #[error("Collector is enough.")]
    CollectorEnough,
}

#[pin_project]
pub(crate) struct FutureTaskSink<C> {
    #[pin]
    collector: C,
    event_handle: Arc<dyn TaskEventHandle>,
}

impl<C> FutureTaskSink<C> {
    pub fn new<Item>(collector: C, event_handle: Arc<dyn TaskEventHandle>) -> Self
    where
        C: TaskResultCollector<Item>,
    {
        Self {
            collector,
            event_handle,
        }
    }

    pub fn into_collector(self) -> C {
        self.collector
    }
}

impl<C, Item> Sink<Item> for FutureTaskSink<C>
where
    C: TaskResultCollector<Item>,
{
    type Error = SinkError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        let this = self.project();
        this.event_handle.on_item();
        let collector_state = this
            .collector
            .collect(item)
            .map_err(SinkError::CollectorError)?;
        match collector_state {
            CollectorState::Enough => Err(SinkError::CollectorEnough),
            CollectorState::Need => Ok(()),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
