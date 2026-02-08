// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type SyncManagerClient = jsonrpsee::async_client::Client;
use crate::{types::SyncStatusView, FutureResult};
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use network_api::PeerStrategy;
use network_p2p_types::peer_id::PeerId;
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
use std::sync::Arc;
pub trait SyncManagerApi {
    fn status(&self) -> FutureResult<SyncStatusView>;
    fn cancel(&self) -> FutureResult<()>;
    /// if `force` is true, will cancel current task and start a new task.
    /// if peers is not empty, will try sync with the special peers.
    fn start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> FutureResult<()>;
    fn progress(&self) -> FutureResult<Option<SyncProgressReport>>;
    fn peer_score(&self) -> FutureResult<PeerScoreResponse>;
}

/// Build jsonrpsee methods from legacy `SyncManagerApi`.
pub fn sync_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: SyncManagerApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("sync.status", |_, api, _| async move {
        api.status().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("sync.cancel", |_, api, _| async move {
        api.cancel().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("sync.start", |params, api, _| async move {
        let (force, peers, skip_pow_verify, strategy): (bool, Vec<PeerId>, bool, Option<PeerStrategy>) =
            params.parse()?;
        api.start(force, peers, skip_pow_verify, strategy)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("sync.progress", |_, api, _| async move {
        api.progress().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("sync.score", |_, api, _| async move {
        api.peer_score().await.map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
