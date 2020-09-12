// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::Message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
        match self {
            Self::Stopped => true,
            _ => false,
        }
    }

    pub(crate) fn is_started(self) -> bool {
        match self {
            Self::Started => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
pub enum ServiceQuery {
    Status,
}

pub enum ServiceQueryResult {
    Status(ServiceStatus),
    Err(Error),
}

impl Message for ServiceQuery {
    type Result = ServiceQueryResult;
}

pub trait ServiceRequest: Send {
    type Response: Send;
}
