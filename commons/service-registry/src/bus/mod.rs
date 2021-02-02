// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ActorService, EventHandler, EventNotifier, ServiceRef, ServiceRequest};
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::mpsc::TrySendError;

mod service;
mod sys_bus;

pub use service::BusService;
pub use sys_bus::SysBus;

#[derive(Clone, Debug)]
pub struct SubscribeRequest<M>
where
    M: Send + Clone + Debug + 'static,
{
    pub notifier: EventNotifier<M>,
}

impl<M> ServiceRequest for SubscribeRequest<M>
where
    M: Send + Clone + Debug,
{
    type Response = ();
}

#[derive(Clone, Debug)]
pub struct UnsubscribeRequest<M>
where
    M: Send + Clone + Debug + 'static,
{
    pub target_service: &'static str,
    msg: PhantomData<M>,
}

impl<M> UnsubscribeRequest<M>
where
    M: Send + Clone + Debug,
{
    pub fn new(target_service: &'static str) -> Self {
        Self {
            target_service,
            msg: PhantomData,
        }
    }
}

impl<M> ServiceRequest for UnsubscribeRequest<M>
where
    M: Send + Clone + Debug,
{
    type Response = ();
}

#[derive(Debug, Default)]
pub struct ChannelRequest<M>
where
    M: Send + Clone + Debug + 'static,
{
    m: PhantomData<M>,
}

impl<M> ChannelRequest<M>
where
    M: Send + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> ServiceRequest for ChannelRequest<M>
where
    M: Send + Clone + Debug,
{
    type Response = Result<mpsc::UnboundedReceiver<M>>;
}

#[derive(Debug, Default)]
pub struct OneshotRequest<M>
where
    M: Send + Clone + Debug + 'static,
{
    m: PhantomData<M>,
}

impl<M> OneshotRequest<M>
where
    M: Send + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> ServiceRequest for OneshotRequest<M>
where
    M: Send + Clone + Debug,
{
    type Response = Result<oneshot::Receiver<M>>;
}

#[derive(Debug, Clone)]
pub struct BroadcastRequest<M>
where
    M: Send + Clone + Debug + 'static,
{
    pub msg: M,
}

impl<M> BroadcastRequest<M>
where
    M: Send + Clone + Debug,
{
    pub fn new(msg: M) -> Self {
        Self { msg }
    }

    pub fn into_inner(self) -> M {
        self.msg
    }
}

impl<M> ServiceRequest for BroadcastRequest<M>
where
    M: Send + Clone + Debug,
{
    type Response = ();
}

#[async_trait::async_trait]
pub trait Bus {
    async fn subscribe<M>(&self, notifier: EventNotifier<M>) -> Result<()>
    where
        M: Send + Clone + Debug + 'static;

    async fn unsubscribe<S, M>(&self) -> Result<()>
    where
        S: ActorService + EventHandler<S, M>,
        M: Send + Clone + Debug + 'static;

    async fn channel<M>(&self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Send + Clone + Debug + 'static;

    async fn oneshot<M>(&self) -> Result<oneshot::Receiver<M>>
    where
        M: Send + Clone + Debug + 'static;

    fn broadcast<M: 'static>(&self, msg: M) -> Result<(), TrySendError<M>>
    where
        M: Send + Clone + Debug;
}

#[async_trait::async_trait]
impl Bus for ServiceRef<BusService> {
    async fn subscribe<M>(&self, notifier: EventNotifier<M>) -> Result<()>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(SubscribeRequest { notifier })
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    async fn unsubscribe<S, M>(&self) -> Result<()>
    where
        S: ActorService + EventHandler<S, M>,
        M: Send + Clone + Debug + 'static,
    {
        self.send(UnsubscribeRequest::<M>::new(S::service_name()))
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    async fn channel<M>(&self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(ChannelRequest::<M>::new())
            .await
            .map_err(Into::<anyhow::Error>::into)?
    }

    async fn oneshot<M>(&self) -> Result<oneshot::Receiver<M>>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(OneshotRequest::<M>::new())
            .await
            .map_err(Into::<anyhow::Error>::into)?
    }

    fn broadcast<M>(&self, msg: M) -> Result<(), TrySendError<M>>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.try_send(BroadcastRequest { msg })
            .map_err(|e| match e {
                TrySendError::Full(m) => TrySendError::Full(m.into_inner()),
                TrySendError::Disconnected(m) => TrySendError::Disconnected(m.into_inner()),
            })
    }
}

#[cfg(test)]
mod tests;
