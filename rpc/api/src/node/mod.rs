// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use self::gen_client::Client as NodeClient;

#[rpc]
pub trait NodeApi {
    #[rpc(name = "node.status")]
    fn status(&self) -> Result<bool>;
}
