// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service::{ActorService, ServiceHandler};
use crate::service_actor::{EventMessage, ServiceActor, ServiceMessage};
use crate::{
    EventHandler, ServiceCmd, ServiceQuery, ServiceQueryResult, ServiceRequest, ServiceStatus,
};
use actix::{Addr, MailboxError};
use anyhow::Result;
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::warn;
use std::fmt::Debug;
use std::time::Duration;

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

impl<S> ServiceRef<S>
where
    S: ActorService,
{
    pub fn new(addr: Addr<ServiceActor<S>>) -> Self {
        Self { addr }
    }

    pub(crate) fn exec_service_cmd(&self, cmd: ServiceCmd) -> Result<()> {
        block_on(async move {
            self.addr
                .send(cmd)
                .timeout(Duration::from_millis(2000))
                .await
                .map_err(anyhow::Error::new)
        })?
    }

    pub fn start_service(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Start)
    }

    pub fn stop_service(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Stop)
    }

    pub fn restart_service(&self) -> Result<()> {
        self.exec_service_cmd(ServiceCmd::Restart)
    }

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

    /// Send service a request and ignore response and error.
    pub fn do_send<R>(&self, request: R)
    where
        R: ServiceRequest + 'static,
        S: ServiceHandler<S, R>,
    {
        self.addr.do_send(ServiceMessage::new(request))
    }

    /// Notify service a event msg.
    pub fn notify<M>(&self, msg: M)
    where
        S: EventHandler<S, M>,
        M: Clone + Debug + Send + 'static,
    {
        self.addr.do_send(EventMessage { msg })
    }

    /// Get self service status
    pub fn self_status(&self) -> ServiceStatus {
        match block_on(async move {
            self.addr
                .send(ServiceQuery::Status)
                .timeout(Duration::from_millis(500))
                .await
        }) {
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

    pub fn is_started(&self) -> bool {
        match self.self_status() {
            ServiceStatus::Started => true,
            _ => false,
        }
    }

    pub fn is_stopped(&self) -> bool {
        match self.self_status() {
            ServiceStatus::Stopped => true,
            _ => false,
        }
    }
}
