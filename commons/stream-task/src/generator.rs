// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::collector::{FutureTaskSink, SinkError};
use crate::task_stream::FutureTaskStream;
use crate::{TaskError, TaskEventHandle, TaskResultCollector, TaskState};
use anyhow::Result;
use futures::task::{Context, Poll};
use futures::{
    future::{abortable, AbortHandle, BoxFuture},
    stream::{self},
    Future, FutureExt, SinkExt, StreamExt, TryFutureExt, TryStreamExt,
};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct TaskHandle {
    inner: AbortHandle,
    is_done: Arc<AtomicBool>,
}

impl TaskHandle {
    pub(crate) fn new(inner: AbortHandle, is_done: Arc<AtomicBool>) -> Self {
        Self { inner, is_done }
    }

    pub fn cancel(&self) {
        self.inner.abort()
    }

    pub fn is_done(&self) -> bool {
        self.is_done.load(Ordering::SeqCst)
    }
}

pub struct TaskFuture<Output> {
    fut: BoxFuture<'static, Result<Output, TaskError>>,
}

impl<Output> TaskFuture<Output>
where
    Output: Send + 'static,
{
    pub fn new(fut: BoxFuture<'static, Result<Output, TaskError>>) -> Self {
        Self { fut }
    }

    pub fn with_handle(self) -> (BoxFuture<'static, Result<Output, TaskError>>, TaskHandle) {
        let (abortable_fut, handle) = abortable(self.fut);
        let is_done = Arc::new(AtomicBool::new(false));
        let fut_is_done = is_done.clone();
        (
            abortable_fut
                .map(move |result| {
                    fut_is_done.store(true, Ordering::SeqCst);
                    match result {
                        Ok(result) => result,
                        Err(_aborted) => Err(TaskError::Canceled),
                    }
                })
                .boxed(),
            TaskHandle::new(handle, is_done),
        )
    }
}

impl<Output> Future for TaskFuture<Output> {
    type Output = Result<Output, TaskError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.fut.as_mut()).poll(cx)
    }
}

pub trait Generator: Send {
    type State: TaskState;
    type Output: std::marker::Send;
    fn generate(self) -> TaskFuture<Self::Output>;

    fn get_event_handle(&self) -> Arc<dyn TaskEventHandle>;

    fn and_then<S, C, M>(
        self,
        buffer_size: usize,
        max_retry_times: u64,
        delay_milliseconds: u64,
        collector: C,
        init_state_map: M,
    ) -> AndThenGenerator<Self, C, S, M>
    where
        Self: Sized + 'static,
        C: TaskResultCollector<S::Item> + 'static,
        S: TaskState + 'static,
        M: FnOnce(Self::Output) -> Result<S> + Send + 'static,
    {
        AndThenGenerator::new(
            self,
            buffer_size,
            max_retry_times,
            delay_milliseconds,
            collector,
            init_state_map,
        )
    }
}

pub struct TaskGenerator<S, C>
where
    S: TaskState,
    C: TaskResultCollector<S::Item>,
{
    init_state: S,
    buffer_size: usize,
    max_retry_times: u64,
    delay_milliseconds: u64,
    collector: C,
    event_handle: Arc<dyn TaskEventHandle>,
}

impl<S, C> TaskGenerator<S, C>
where
    S: TaskState + 'static,
    C: TaskResultCollector<S::Item> + 'static,
{
    pub fn new(
        init_state: S,
        buffer_size: usize,
        max_retry_times: u64,
        delay_milliseconds_on_error: u64,
        collector: C,
        event_handle: Arc<dyn TaskEventHandle>,
    ) -> Self {
        Self {
            init_state,
            buffer_size,
            max_retry_times,
            delay_milliseconds: delay_milliseconds_on_error,
            collector,
            event_handle,
        }
    }
}

impl<S, C> Generator for TaskGenerator<S, C>
where
    S: TaskState + 'static,
    C: TaskResultCollector<S::Item> + 'static,
{
    type State = S;
    type Output = C::Output;

    fn generate(self) -> TaskFuture<C::Output> {
        let fut = async move {
            let task_name = S::task_name();
            let total_item = self.init_state.total_items();
            let event_handle = self.event_handle;
            event_handle.on_start(task_name.to_string(), total_item);
            let stream = FutureTaskStream::new(
                self.init_state,
                self.max_retry_times,
                self.delay_milliseconds,
                event_handle.clone(),
            );
            let mut buffered_stream = stream
                .buffered(self.buffer_size)
                .map(|result| {
                    let items = match result {
                        Ok(items) => items.into_iter().map(Ok).collect(),
                        Err(e) => vec![Err(e)],
                    };
                    stream::iter(items)
                })
                .flatten()
                .map_err(SinkError::StreamTaskError);
            let mut sink = FutureTaskSink::new(self.collector, event_handle.clone());
            let sink_result = sink.send_all(&mut buffered_stream).await;
            if let Err(sink_err) = sink_result {
                match sink_err {
                    SinkError::StreamTaskError(e) => return Err(e),
                    SinkError::CollectorError(e) => return Err(TaskError::CollectorError(e)),
                    SinkError::CollectorEnough => {
                        //continue
                    }
                }
            }
            let collector = sink.into_collector();
            event_handle.on_finish(task_name.to_string());
            collector.finish().map_err(TaskError::CollectorError)
        }
        .boxed();

        TaskFuture::new(fut)
    }

    fn get_event_handle(&self) -> Arc<dyn TaskEventHandle> {
        self.event_handle.clone()
    }
}

pub struct AndThenGenerator<G, C, S, M> {
    g1: G,
    buffer_size: usize,
    max_retry_times: u64,
    delay_milliseconds: u64,
    collector: C,
    init_state_map: M,
    init_state: PhantomData<S>,
}

impl<G, C, S, M> AndThenGenerator<G, C, S, M>
where
    G: Generator + 'static,
    S: TaskState + 'static,
    C: TaskResultCollector<S::Item> + 'static,
    M: FnOnce(G::Output) -> Result<S> + Send + 'static,
{
    pub(crate) fn new(
        g1: G,
        buffer_size: usize,
        max_retry_times: u64,
        delay_milliseconds: u64,
        collector: C,
        init_state_map: M,
    ) -> Self {
        Self {
            g1,
            buffer_size,
            max_retry_times,
            delay_milliseconds,
            collector,
            init_state_map,
            init_state: PhantomData,
        }
    }
}

impl<G, C, S, M> Generator for AndThenGenerator<G, C, S, M>
where
    G: Generator + 'static,
    S: TaskState + 'static,
    C: TaskResultCollector<S::Item> + 'static,
    M: FnOnce(G::Output) -> Result<S> + Send + 'static,
{
    type State = S;
    type Output = C::Output;

    fn generate(self) -> TaskFuture<Self::Output> {
        let Self {
            g1,
            buffer_size,
            max_retry_times,
            delay_milliseconds,
            collector,
            init_state_map,
            init_state: _,
        } = self;
        let event_handle = g1.get_event_handle();
        let first_task = g1.generate();
        let then_fut = first_task
            .and_then(|output| async move {
                (init_state_map)(output).map_err(TaskError::CollectorError)
            })
            .and_then(move |init_state| {
                TaskGenerator::new(
                    init_state,
                    buffer_size,
                    max_retry_times,
                    delay_milliseconds,
                    collector,
                    event_handle,
                )
                .generate()
            })
            .boxed();
        TaskFuture::new(then_fut)
    }

    fn get_event_handle(&self) -> Arc<dyn TaskEventHandle> {
        self.g1.get_event_handle()
    }
}
