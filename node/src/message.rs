// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::prelude::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum NodeCommand {
    StartMiner(),
    StopMiner(),
}

impl Message for NodeCommand {
    type Result = Result<()>;
}
