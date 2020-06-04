// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use self::gen_client::Client as DebugClient;

#[rpc]
pub trait DebugApi {
    /// update log level, if logger_name is none, update global log level.
    #[rpc(name = "debug.set_log_level")]
    fn set_log_level(&self, logger_name: Option<String>, level: String) -> Result<()>;

    ///Trigger the node panic, only work for dev network.
    #[rpc(name = "debug.panic")]
    fn panic(&self) -> Result<()>;
}
