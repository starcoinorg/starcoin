// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::to_invalid_param_err;
use jsonrpc_core::Result;
use starcoin_logger::prelude::LevelFilter;
use starcoin_logger::LoggerHandle;
use starcoin_rpc_api::debug::DebugApi;
use std::str::FromStr;
use std::sync::Arc;

pub struct DebugRpcImpl {
    log_handle: Arc<LoggerHandle>,
}

impl DebugRpcImpl {
    pub fn new(log_handle: Arc<LoggerHandle>) -> Self {
        Self { log_handle }
    }
}

impl DebugApi for DebugRpcImpl {
    fn set_log_level(&self, level: String) -> Result<()> {
        self.log_handle
            .update_level(LevelFilter::from_str(level.as_str()).map_err(to_invalid_param_err)?);
        Ok(())
    }
}
