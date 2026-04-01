// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_logger::LogPattern;

use crate::types::FactoryAction;
#[rpc(client, server)]
pub trait DebugApi {
    /// Update log level, if logger_name is none, update global log level.
    #[method(name = "debug.set_log_level")]
    fn set_log_level(&self, logger_name: Option<String>, level: String) -> RpcResult<()>;

    /// Set log pattern
    #[method(name = "debug.set_log_pattern")]
    fn set_log_pattern(&self, pattern: LogPattern) -> RpcResult<()>;

    ///Trigger the node panic, only work for dev network.
    #[method(name = "debug.panic")]
    fn panic(&self) -> RpcResult<()>;

    ///Only can used under dev net.
    #[method(name = "debug.sleep")]
    fn sleep(&self, time: u64) -> RpcResult<()>;

    /// Get and set txn factory status.
    #[method(name = "txfactory.status")]
    fn txfactory_status(&self, action: FactoryAction) -> RpcResult<bool>;

    /// Update vm concurrency level, level = min(level, num_cpus::get)
    #[method(name = "debug.set_concurrency_level")]
    fn set_concurrency_level(&self, level: usize) -> RpcResult<()>;

    /// Get vm concurrency level
    #[method(name = "debug.get_concurrency_level")]
    fn get_concurrency_level(&self) -> RpcResult<usize>;

    /// Update logger balance amount
    #[method(name = "debug.set_logger_balance_amount")]
    fn set_logger_balance_amount(&self, balance_amount: u64) -> RpcResult<()>;

    /// Get logger balance amount
    #[method(name = "debug.get_logger_balance_amount")]
    fn get_logger_balance_amount(&self) -> RpcResult<u64>;
}

pub use DebugApiClient as DebugApiRpcClient;
pub use DebugApiServer as DebugApiRpcServer;

/// Build jsonrpsee methods from legacy `DebugApi`.
pub fn debug_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: DebugApiServer + Send + Sync + 'static,
{
    Ok(DebugApiServer::into_rpc(api).into())
}
