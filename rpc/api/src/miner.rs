// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use starcoin_crypto::HashValue;

pub use self::gen_client::Client as MinerClient;

#[rpc]
pub trait MinerApi {
    /// submit mining seal
    #[rpc(name = "mining.submit")]
    fn submit(&self, header_hash: HashValue, nonce: u64) -> Result<()>;
}
