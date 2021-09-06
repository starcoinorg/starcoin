// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::channel::oneshot::Receiver;
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ServiceInfo, ServiceRequest, ServiceStatus};

#[derive(Debug, Clone)]
pub enum NodeRequest {
    ListService,
    StartService(String),
    CheckService(String),
    StopService(String),
    StopPacemaker,
    StartPacemaker,
    ShutdownSystem,
    ResetNode(HashValue),
    ReExecuteBlock(HashValue),
    DeleteBlock(HashValue),
    DeleteFailedBlock(HashValue),
}

#[derive(Debug)]
pub enum NodeResponse {
    Services(Vec<ServiceInfo>),
    Result(Result<()>),
    AsyncResult(Receiver<Result<()>>),
    ServiceStatus(ServiceStatus),
}

impl ServiceRequest for NodeRequest {
    type Response = Result<NodeResponse>;
}
