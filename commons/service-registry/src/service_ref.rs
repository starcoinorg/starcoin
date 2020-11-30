// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::{ActorService, ServiceHandler};
use crate::service_actor::{EventMessage, ServiceActor, ServiceMessage};
use crate::{
    EventHandler, ServiceCmd, ServiceEventStream, ServiceQuery, ServiceQueryResult, ServiceRequest,
    ServiceStatus,
};
use actix::dev::SendError;
use actix::{Addr, MailboxError, Recipient};
use anyhow::Result;
use futures::channel::mpsc::channel;
use futures::future::BoxFuture;
use futures::{FutureExt, Stream};
use log::warn;
use std::any::type_name;
use std::fmt::Debug;
use std::sync::mpsc::TrySendError;
use std::time::Duration;

#[derive(Clone)]
pub struct EventNotifier<M>
where
    M: Send + Debug,
{
    // target service name.
    target_service: &'static str,
    recipient: Recipient<EventMessage<M>>,
}

impl<M> EventNotifier<M>
where
    M: Send + Debug,
{
    pub fn target_service(&self) -> &'static str {
        self.target_service
    }

    pub fn notify(&self, msg: M) -> Result<(), TrySendError<M>> {
        self.recipient
            .try_send(EventMessage::new(msg))
            .map_err(|e| match e {
                SendError::Full(m) => TrySendError::Full(m.into_inner()),
                SendError::Closed(m) => TrySendError::Disconnected(m.into_inner()),
            })
    }

    pub fn is_closed(&self) -> bool {
        !self.recipient.connected()
    }
}

impl<S, M> From<ServiceRef<S>> for EventNotifier<M>
where
    S: ActorService + EventHandler<S, M>,
    M: Send + Debug + 'static,
{
    fn from(service_ref: ServiceRef<S>) -> Self {
        Self {
            target_service: S::service_name(),
            recipient: service_ref.addr.recipient::<EventMessage<M>>(),
        }
    }
}

impl<M> Debug for EventNotifier<M>
where
    M: Send + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_to_{}_EventNotifier",
            type_name::<M>(),
            self.target_service
        )
    }
}

pub struct ServiceRef<S>
where
    S: ActorService + 'static,
{
    pub(crate) addr: Addr<ServiceActor<S>>,
}

impl<S> Clone for ServiceRef<S>
where
    S: ActorService,
{
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
        }
    }
}

impl<S> From<Addr<ServiceActor<S>>> for ServiceRef<S>
where
    S: ActorService + 'static,
{
    fn from(addr: Addr<ServiceActor<S>>) -> Self {
        Self { addr }
    }
}

pub const DEFAULT_TIMEOUT_MILLIS: u64 = 5000;

impl<S> ServiceRef<S>
where
    S: ActorService,
{
    pub fn new(addr: Addr<ServiceActor<S>>) -> Self {
        Self { addr }
    }

    pub(crate) fn exec_service_cmd(&self, cmd: ServiceCmd) -> Result<()> {
        self.addr.try_send(cmd).map_err(anyhow::Error::new)
    }

    pub fn start_self(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Start)
    }

    pub fn stop_self(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Stop)
    }

    pub fn restart_self(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Restart)
    }

    /// Returns whether the actor is still alive.
    pub fn connected(&self) -> bool {
        self.addr.connected()
    }

    /// Send a request to target service and wait response by default timeout.
    pub fn send<R>(&self, request: R) -> BoxFuture<Result<<R as ServiceRequest>::Response>>
    where
        R: ServiceRequest + 'static,
        S: ServiceHandler<S, R>,
    {
        async move {
            self.addr
                .send(ServiceMessage::new(request))
                .await
                .map_err(anyhow::Error::new)?
        }
        .boxed()
    }

    /// Send a request to target service and ignore response and error.
    pub fn do_send<R>(&self, request: R)
    where
        R: ServiceRequest + 'static,
        S: ServiceHandler<S, R>,
    {
        self.addr.do_send(ServiceMessage::new(request))
    }

    pub fn try_send<R>(&self, request: R) -> Result<(), TrySendError<R>>
    where
        R: ServiceRequest + 'static,
        S: ServiceHandler<S, R>,
    {
        self.addr
            .try_send(ServiceMessage::new(request))
            .map_err(|e| match e {
                SendError::Full(m) => TrySendError::Full(m.into_inner()),
                SendError::Closed(m) => TrySendError::Disconnected(m.into_inner()),
            })
    }

    /// Send a event to target service
    pub fn notify<M>(&self, msg: M) -> Result<(), TrySendError<M>>
    where
        S: EventHandler<S, M>,
        M: Clone + Debug + Send + 'static,
    {
        self.addr
            .try_send(EventMessage::new(msg))
            .map_err(|e| match e {
                SendError::Full(m) => TrySendError::Full(m.into_inner()),
                SendError::Closed(m) => TrySendError::Disconnected(m.into_inner()),
            })
    }

    /// Convert self to a single event message notifier.
    pub fn event_notifier<M>(self) -> EventNotifier<M>
    where
        S: EventHandler<S, M>,
        M: Debug + Send + 'static,
    {
        self.into()
    }

    pub fn add_event_stream<M, Fut>(&self, stream: Fut) -> Result<(), TrySendError<Fut>>
    where
        S: EventHandler<S, M>,
        Fut: Stream<Item = M> + Send + 'static,
        M: Debug + Send + 'static,
    {
        self.addr
            .try_send(ServiceEventStream { stream })
            .map_err(|e| match e {
                SendError::Full(m) => TrySendError::Full(m.stream),
                SendError::Closed(m) => TrySendError::Disconnected(m.stream),
            })
    }

    /// Create a `Sender` for special event message.
    pub fn event_sender<M>(&self, buffer: usize) -> futures::channel::mpsc::Sender<M>
    where
        S: EventHandler<S, M>,
        M: Debug + Send + 'static,
    {
        let (sender, receiver) = channel(buffer);
        // drop error, the receiver will also bean dropped, so sender will got error when try send.
        let _ = self.add_event_stream(receiver);
        sender
    }

    /// Get self service status
    pub async fn self_status(&self) -> ServiceStatus {
        match self
            .addr
            .send(ServiceQuery::Status)
            .timeout(Duration::from_millis(50))
            .await
        {
            Ok(status) => match status {
                ServiceQueryResult::Status(status) => status,
                _ => unreachable!(),
            },
            Err(e) => {
                warn!(
                    "Query {} service status error: {:?}, service is unavailable",
                    S::service_name(),
                    e
                );
                match e {
                    MailboxError::Timeout => ServiceStatus::Unavailable,
                    MailboxError::Closed => ServiceStatus::Shutdown,
                }
            }
        }
    }
}
