// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::{async_trait, RpcResult};
use network_api::PeerStrategy;
use network_p2p_types::peer_id::PeerId;
use starcoin_rpc_api::{sync_manager::SyncManagerApiServer, types::SyncStatusView};
use starcoin_sync_api::{PeerScoreResponse, SyncAsyncService, SyncProgressReport};

pub struct SyncManagerRpcImpl<S>
where
    S: SyncAsyncService + 'static,
{
    service: S,
}

impl<S> SyncManagerRpcImpl<S>
where
    S: SyncAsyncService,
{
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

#[async_trait]
impl<S> SyncManagerApiServer for SyncManagerRpcImpl<S>
where
    S: SyncAsyncService,
{
    async fn status(&self) -> RpcResult<SyncStatusView> {
        let service = self.service.clone();
        let result = service
            .status()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(result.into())
    }

    async fn cancel(&self) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .cancel()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> RpcResult<()> {
        let service = self.service.clone();
        service
            .start(force, peers, skip_pow_verify, strategy)
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(())
    }

    async fn progress(&self) -> RpcResult<Option<SyncProgressReport>> {
        let service = self.service.clone();
        let result = service
            .progress()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(result)
    }

    async fn peer_score(&self) -> RpcResult<PeerScoreResponse> {
        let service = self.service.clone();
        let result = service
            .sync_peer_score()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(result)
    }
}
