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
}
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
