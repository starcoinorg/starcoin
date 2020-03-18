// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures::future::{FutureExt, TryFutureExt};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

pub use self::gen_client::Client as StatusClient;

#[rpc]
pub trait StatusApi {
    #[rpc(name = "status")]
    fn status(&self) -> Result<String>;
}
