// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service_registry::ServiceInfo;
use actix::prelude::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum NodeRequest {
    ListService,
    StartService(String),
    StopService(String),
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
