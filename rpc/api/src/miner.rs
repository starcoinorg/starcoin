// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type MinerClient = jsonrpsee::async_client::Client;
use crate::types::MintedBlockView;
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use starcoin_types::system_events::MintBlockEvent;
use std::sync::Arc;
pub trait MinerApi {
    /// submit mining seal
    fn submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> FutureResult<MintedBlockView>;
    /// get current mining job
    fn get_job(&self) -> FutureResult<Option<MintBlockEvent>>;
}

/// Build jsonrpsee methods from legacy `MinerApi`.
pub fn miner_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: MinerApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("mining.submit", |params, api, _| async move {
        let (minting_blob, nonce, extra): (String, u32, String) = params.parse()?;
        api.submit(minting_blob, nonce, extra)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("mining.get_job", |_, api, _| async move {
        api.get_job().await.map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}

