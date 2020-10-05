// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use starcoin_service_registry::{ServiceInfo, ServiceRequest};

#[derive(Debug, Clone)]
pub enum NodeRequest {
    ListService,
    StartService(String),
    StopService(String),
    StopPacemaker,
    StartPacemaker,
    StopSystem,
}

#[derive(Debug)]
pub enum NodeResponse {
    Services(Vec<ServiceInfo>),
    Result(Result<()>),
}

impl Message for NodeRequest {
    type Result = NodeResponse;
}

impl ServiceRequest for NodeRequest {
    type Response = NodeResponse;
}
