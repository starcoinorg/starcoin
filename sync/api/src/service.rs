// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{SyncCancelRequest, SyncProgressRequest, SyncStartRequest, SyncStatusRequest};
use anyhow::Result;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::sync_status::SyncStatus;
use stream_task::TaskProgressReport;

#[async_trait::async_trait]
pub trait SyncAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn status(&self) -> Result<SyncStatus>;
    async fn progress(&self) -> Result<Option<TaskProgressReport>>;
    async fn cancel(&self) -> Result<()>;
    async fn start(&self, force: bool) -> Result<()>;
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

    async fn progress(&self) -> Result<Option<TaskProgressReport>> {
        self.send(SyncProgressRequest).await
    }

    async fn cancel(&self) -> Result<()> {
        self.send(SyncCancelRequest).await
    }

    async fn start(&self, force: bool) -> Result<()> {
        self.send(SyncStartRequest { force }).await?
    }
}
