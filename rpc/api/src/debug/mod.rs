// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use openrpc_derive::openrpc;
use starcoin_logger::LogPattern;

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

    /// Set minimum pending txn threshold (dev only)
    #[rpc(name = "debug.set_min_pending_txn_threshold")]
    fn set_min_pending_txn_threshold(&self, threshold: usize) -> Result<()>;

    /// Get minimum pending txn threshold (dev only)
    #[rpc(name = "debug.get_min_pending_txn_threshold")]
    fn get_min_pending_txn_threshold(&self) -> Result<usize>;

    /// Get logger balance amount
    #[rpc(name = "debug.get_logger_balance_amount")]
    fn get_logger_balance_amount(&self) -> Result<u64>;
}
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
