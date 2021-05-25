// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as MinerClient;
use crate::types::MintedBlockView;
use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_types::system_events::MintBlockEvent;

#[rpc]
pub trait MinerApi {
    /// submit mining seal
    #[rpc(name = "mining.submit")]
    fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> FutureResult<MintedBlockView>;
    /// get current mining job
    #[rpc(name = "mining.get_job")]
    fn get_job(&self) -> FutureResult<Option<MintBlockEvent>>;
}
