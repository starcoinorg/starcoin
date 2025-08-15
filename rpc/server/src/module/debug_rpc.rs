// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::to_invalid_param_err;
use crate::module::txfactory_rpc::TxFactoryStatusHandle;
use jsonrpc_core::Result;
use log::warn;
use pprof::ProfilerGuard;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::LevelFilter;
use starcoin_logger::structured_log::set_slog_level;
use starcoin_logger::{LogPattern, LoggerHandle};
use starcoin_rpc_api::debug::DebugApi;
use starcoin_rpc_api::types::FactoryAction;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_types::system_events::GenerateBlockEvent;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use std::fs::File;
use std::str::FromStr;
use std::sync::{Arc, OnceLock, RwLock};
use std::thread::sleep;
use std::time::Duration;

static PPROF_PROFILER: OnceLock<Arc<RwLock<Option<ProfilerGuard<'static>>>>> = OnceLock::new();

fn get_pprof_guard() -> Arc<RwLock<Option<ProfilerGuard<'static>>>> {
    PPROF_PROFILER
        .get_or_init(|| {
            let freq = if cfg!(target_os = "macos") { 50 } else { 100 };
            let blocklist = if cfg!(target_os = "macos") {
                vec![
                    "libc",
                    "libgcc",
                    "pthread",
                    "vdso",
                    "libsystem_kernel.dylib, libsystem_pthread.dylib",
                ]
            } else {
                vec!["libc", "libgcc", "pthread", "vdso"]
            };
            Arc::new(RwLock::new(
                pprof::ProfilerGuardBuilder::default()
                    .frequency(freq)
                    .blocklist(&blocklist)
                    .build()
                    .ok(),
            ))
        })
        .clone()
}

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
            .broadcast(GenerateBlockEvent::new(true, true))
            .map_err(|_e| jsonrpc_core::Error::internal_error())?;
        Ok(())
    }

    fn txfactory_status(&self, action: FactoryAction) -> Result<bool> {
        Ok(TxFactoryStatusHandle::handle_action(action))
    }

    fn set_concurrency_level(&self, level: usize) -> Result<()> {
        StarcoinVM::set_concurrency_level_once(level);
        Ok(())
    }

    fn get_concurrency_level(&self) -> Result<usize> {
        let guard = get_pprof_guard();
        // take and drop the guard to avoid holding the lock for too long.
        let guard = if let Ok(mut guard) = guard.write() {
            guard.take()
        } else {
            None
        };

        if let Some(g) = guard {
            std::thread::spawn(move || {
                sleep(Duration::from_secs(60));
                let Ok(report) = g.report().build() else {
                    warn!("Failed to build pprof report.");
                    return;
                };
                let Ok(file) = File::create("flamegraph.svg") else {
                    warn!("Failed to create flamegraph file.");
                    return;
                };
                if report.flamegraph(file).is_err() {
                    warn!("Failed to generate flamegraph report.");
                }
            });
        }
        Ok(StarcoinVM::get_concurrency_level())
    }

    fn set_logger_balance_amount(&self, balance_amount: u64) -> Result<()> {
        starcoin_executor::set_logger_balance_amount_once(balance_amount);
        Ok(())
    }

    fn get_logger_balance_amount(&self) -> Result<u64> {
        Ok(starcoin_executor::get_logger_balance_amount())
    }
}
