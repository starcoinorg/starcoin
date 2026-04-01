// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::MintedBlockView;
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_types::system_events::MintBlockEvent;
#[rpc(client, server)]
pub trait MinerApi {
    /// submit mining seal
    #[method(name = "mining.submit")]
    async fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> RpcResult<MintedBlockView>;
    /// get current mining job
    #[method(name = "mining.get_job")]
    async fn get_job(&self) -> RpcResult<Option<MintBlockEvent>>;
}

pub use MinerApiClient as MinerApiRpcClient;
pub use MinerApiServer as MinerApiRpcServer;

/// Build jsonrpsee methods from legacy `MinerApi`.
pub fn miner_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: MinerApiServer + Send + Sync + 'static,
{
    Ok(MinerApiServer::into_rpc(api).into())
}
