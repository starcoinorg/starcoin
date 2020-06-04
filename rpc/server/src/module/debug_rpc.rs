// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::to_invalid_param_err;
use jsonrpc_core::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::LevelFilter;
use starcoin_logger::LoggerHandle;
use starcoin_rpc_api::debug::DebugApi;
use std::str::FromStr;
use std::sync::Arc;

pub struct DebugRpcImpl {
    config: Arc<NodeConfig>,
    log_handle: Arc<LoggerHandle>,
}

impl DebugRpcImpl {
    pub fn new(config: Arc<NodeConfig>, log_handle: Arc<LoggerHandle>) -> Self {
        Self { config, log_handle }
    }
}

impl DebugApi for DebugRpcImpl {
    fn set_log_level(&self, logger_name: Option<String>, level: String) -> Result<()> {
        let logger_name = logger_name.and_then(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });
        let level = LevelFilter::from_str(level.as_str()).map_err(to_invalid_param_err)?;
        match logger_name {
            None => self.log_handle.update_level(level),
            Some(n) => self.log_handle.set_log_level(n, level),
        }

        Ok(())
    }

    fn panic(&self) -> Result<()> {
        if !self.config.net().is_dev() {
            return Err(jsonrpc_core::Error::invalid_request());
        }
        panic!("DebugApi.panic")
    }
}
