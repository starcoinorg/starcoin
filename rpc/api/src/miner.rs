// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as MinerClient;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

#[rpc]
pub trait MinerApi {
    /// submit mining seal
    #[rpc(name = "mining.submit")]
    fn submit(&self, minting_blob: Vec<u8>, nonce: u32) -> Result<()>;
}
