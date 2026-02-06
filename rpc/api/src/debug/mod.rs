// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use openrpc_derive::openrpc;
use starcoin_logger::LogPattern;
use std::sync::Arc;

pub use self::gen_client::Client as DebugClient;
use crate::types::FactoryAction;
#[openrpc]
pub trait DebugApi {
    /// Update log level, if logger_name is none, update global log level.
    #[rpc(name = "debug.set_log_level")]
    fn set_log_level(&self, logger_name: Option<String>, level: String) -> Result<()>;

    /// Set log pattern
    #[rpc(name = "debug.set_log_pattern")]
    fn set_log_pattern(&self, pattern: LogPattern) -> Result<()>;

    ///Trigger the node panic, only work for dev network.
    #[rpc(name = "debug.panic")]
    fn panic(&self) -> Result<()>;

    ///Only can used under dev net.
    #[rpc(name = "debug.sleep")]
    fn sleep(&self, time: u64) -> Result<()>;

    /// Get and set txn factory status.
    #[rpc(name = "txfactory.status")]
    fn txfactory_status(&self, action: FactoryAction) -> Result<bool>;

    /// Update vm concurrency level, level = min(level, num_cpus::get)
    #[rpc(name = "debug.set_concurrency_level")]
    fn set_concurrency_level(&self, level: usize) -> Result<()>;

    /// Get vm concurrency level
    #[rpc(name = "debug.get_concurrency_level")]
    fn get_concurrency_level(&self) -> Result<usize>;

    /// Update logger balance amount
    #[rpc(name = "debug.set_logger_balance_amount")]
    fn set_logger_balance_amount(&self, balance_amount: u64) -> Result<()>;

    /// Get logger balance amount
    #[rpc(name = "debug.get_logger_balance_amount")]
    fn get_logger_balance_amount(&self) -> Result<u64>;
}

/// Build jsonrpsee methods from legacy `DebugApi`.
pub fn debug_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: DebugApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_method("debug.set_log_level", |params, api, _| {
        let (logger_name, level): (Option<String>, String) = params.parse()?;
        api.set_log_level(logger_name, level)
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_method("debug.set_log_pattern", |params, api, _| {
        let pattern: LogPattern = params.one()?;
        api.set_log_pattern(pattern).map_err(crate::map_jsonrpc_err)
    })?;

    module.register_method("debug.panic", |_, api, _| api.panic().map_err(crate::map_jsonrpc_err))?;
    module.register_method("debug.sleep", |params, api, _| {
        let time: u64 = params.one()?;
        api.sleep(time).map_err(crate::map_jsonrpc_err)
    })?;
    module.register_method("txfactory.status", |params, api, _| {
        let action: FactoryAction = params.one()?;
        api.txfactory_status(action).map_err(crate::map_jsonrpc_err)
    })?;
    module.register_method("debug.set_concurrency_level", |params, api, _| {
        let level: usize = params.one()?;
        api.set_concurrency_level(level)
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_method("debug.get_concurrency_level", |_, api, _| {
        api.get_concurrency_level().map_err(crate::map_jsonrpc_err)
    })?;
    module.register_method("debug.set_logger_balance_amount", |params, api, _| {
        let balance_amount: u64 = params.one()?;
        api.set_logger_balance_amount(balance_amount)
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_method("debug.get_logger_balance_amount", |_, api, _| {
        api.get_logger_balance_amount()
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
