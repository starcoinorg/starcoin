// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::SyncStatusView;
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use network_api::PeerStrategy;
use network_p2p_types::peer_id::PeerId;
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
#[rpc(client, server, namespace = "sync", namespace_separator = ".")]
pub trait SyncManagerApi {
    #[method(name = "status")]
    async fn status(&self) -> RpcResult<SyncStatusView>;

    #[method(name = "cancel")]
    async fn cancel(&self) -> RpcResult<()>;

    /// if `force` is true, will cancel current task and start a new task.
    /// if peers is not empty, will try sync with the special peers.
    #[method(name = "start")]
    async fn start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> RpcResult<()>;

    #[method(name = "progress")]
    async fn progress(&self) -> RpcResult<Option<SyncProgressReport>>;

    #[method(name = "score")]
    async fn peer_score(&self) -> RpcResult<PeerScoreResponse>;
}

pub use SyncManagerApiClient as SyncManagerApiRpcClient;
pub use SyncManagerApiServer as SyncManagerApiRpcServer;

/// Build jsonrpsee methods from legacy `SyncManagerApi`.
pub fn sync_manager_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: SyncManagerApiServer + Send + Sync + 'static,
{
    Ok(SyncManagerApiServer::into_rpc(api).into())
}
