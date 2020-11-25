// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    SyncCancelRequest, SyncProgressReport, SyncProgressRequest, SyncStartRequest, SyncStatusRequest,
};
use anyhow::Result;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::peer_info::PeerId;
use starcoin_types::sync_status::SyncStatus;

#[async_trait::async_trait]
pub trait SyncAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn status(&self) -> Result<SyncStatus>;
    async fn progress(&self) -> Result<Option<SyncProgressReport>>;
    async fn cancel(&self) -> Result<()>;
    /// if `force` is true, will cancel current task and start a new task.
    /// if peers is not empty, will try sync with the special peers.
    async fn start(&self, force: bool, peers: Vec<PeerId>) -> Result<()>;
}

pub trait SyncServiceHandler:
    ActorService
    + ServiceHandler<Self, SyncStatusRequest>
    + ServiceHandler<Self, SyncProgressRequest>
    + ServiceHandler<Self, SyncCancelRequest>
    + ServiceHandler<Self, SyncStartRequest>
{
}

#[async_trait::async_trait]
impl<S> SyncAsyncService for ServiceRef<S>
where
    S: SyncServiceHandler,
{
    async fn status(&self) -> Result<SyncStatus> {
        self.send(SyncStatusRequest).await
    }

    async fn progress(&self) -> Result<Option<SyncProgressReport>> {
        self.send(SyncProgressRequest).await
    }

    async fn cancel(&self) -> Result<()> {
        self.send(SyncCancelRequest).await
    }

    async fn start(&self, force: bool, peers: Vec<PeerId>) -> Result<()> {
        self.send(SyncStartRequest { force, peers }).await?
    }
}
