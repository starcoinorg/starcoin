// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use network_api::PeerStrategy;
use starcoin_rpc_api::sync_manager::SyncManagerApi;
use starcoin_rpc_api::FutureResult;
use starcoin_sync_api::{PeerScoreResponse, SyncAsyncService, SyncProgressReport};
use starcoin_types::peer_info::PeerId;
use starcoin_types::sync_status::SyncStatus;

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

impl<S> SyncManagerApi for SyncManagerRpcImpl<S>
where
    S: SyncAsyncService,
{
    fn status(&self) -> FutureResult<SyncStatus> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.status().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn cancel(&self) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service.cancel().await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> FutureResult<()> {
        let service = self.service.clone();
        let fut = async move {
            service
                .start(force, peers, skip_pow_verify, strategy)
                .await?;
            Ok(())
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn progress(&self) -> FutureResult<Option<SyncProgressReport>> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.progress().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }

    fn peer_score(&self) -> FutureResult<PeerScoreResponse> {
        let service = self.service.clone();
        let fut = async move {
            let result = service.sync_peer_score().await?;
            Ok(result)
        }
        .map_err(map_err);
        Box::pin(fut.boxed())
    }
}
