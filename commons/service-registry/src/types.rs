// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use anyhow::{Error, Result};
use futures::Stream;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ServiceStatus {
    /// Actor thread and Service is started and running.
    Started,
    /// Service is stopped
    Stopped,
    /// Actor thread is stopped
    Shutdown,
    /// Get status timeout, unknown status.
    Unavailable,
}

impl ServiceStatus {
    pub(crate) fn is_stopped(self) -> bool {
        matches!(self, Self::Stopped)
    }

    pub(crate) fn is_started(self) -> bool {
        matches!(self, Self::Started)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
}

#[derive(Clone, Debug)]
pub enum ServiceCmd {
    Start,
    Stop,
    Restart,
}

impl Message for ServiceCmd {
    type Result = Result<()>;
}

#[derive(Clone, Debug)]
pub(crate) enum ServiceQuery {
    Status,
}

pub enum ServiceQueryResult {
    Status(ServiceStatus),
    Err(Error),
}

impl Message for ServiceQuery {
    type Result = ServiceQueryResult;
}

#[derive(Clone, Debug)]
pub(crate) struct ServicePing;

impl Message for ServicePing {
    type Result = ();
}

#[derive(Clone, Debug)]
pub(crate) struct ServiceEventStream<Fut>
where
    Fut: Stream + 'static,
{
    pub stream: Fut,
}

impl<Fut> Message for ServiceEventStream<Fut>
where
    Fut: Stream,
{
    type Result = ();
}

pub trait ServiceRequest: Send + Debug {
    type Response: Send;
}
