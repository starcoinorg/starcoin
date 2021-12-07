// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::to_invalid_param_err;
use crate::module::txfactory_rpc::TxFactoryStatusHandle;
use jsonrpc_core::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::LevelFilter;
use starcoin_logger::structured_log::set_slog_level;
use starcoin_logger::{LogPattern, LoggerHandle};
use starcoin_rpc_api::debug::DebugApi;
use starcoin_rpc_api::types::FactoryAction;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_types::system_events::GenerateBlockEvent;
use std::str::FromStr;
use std::sync::Arc;

pub struct DebugRpcImpl {
    config: Arc<NodeConfig>,
    log_handle: Arc<LoggerHandle>,
    bus: ServiceRef<BusService>,
}

impl DebugRpcImpl {
    pub fn new(
        config: Arc<NodeConfig>,
        log_handle: Arc<LoggerHandle>,
        bus: ServiceRef<BusService>,
    ) -> Self {
        Self {
            config,
            log_handle,
            bus,
        }
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
        set_slog_level(level.as_str());
        match logger_name {
            None => self.log_handle.update_level(level),
            Some(n) => self.log_handle.set_log_level(n, level),
        }

        Ok(())
    }

    fn set_log_pattern(&self, pattern: LogPattern) -> Result<()> {
        self.log_handle.set_log_pattern(pattern);
        Ok(())
    }

    fn panic(&self) -> Result<()> {
        if !self.config.net().is_test() || self.config.net().is_dev() {
            return Err(jsonrpc_core::Error::invalid_request());
        }
        panic!("DebugApi.panic")
    }

    fn sleep(&self, time: u64) -> Result<()> {
        if !self.config.net().is_test() && !self.config.net().is_dev() {
            return Err(jsonrpc_core::Error::invalid_request());
        }
        self.config.net().time_service().sleep(time);
        self.bus
            .broadcast(GenerateBlockEvent::new(true))
            .map_err(|_e| jsonrpc_core::Error::internal_error())?;
        Ok(())
    }

    fn txfactory_status(&self, action: FactoryAction) -> Result<bool> {
        Ok(TxFactoryStatusHandle::handle_action(action))
    }
}
