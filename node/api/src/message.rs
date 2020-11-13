// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
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
}

#[derive(Debug)]
pub enum NodeResponse {
    Services(Vec<ServiceInfo>),
    Result(Result<()>),
    ServiceStatus(ServiceStatus),
}

impl ServiceRequest for NodeRequest {
    type Response = Result<NodeResponse>;
}
