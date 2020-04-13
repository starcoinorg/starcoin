// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use self::gen_client::Client as DebugClient;

#[rpc]
pub trait DebugApi {
    #[rpc(name = "debug.set_log_level")]
    fn set_log_level(&self, level: String) -> Result<()>;
}
