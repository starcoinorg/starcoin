// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::types::ConnectLocal;
use anyhow::Result;
use jsonrpc_core_client::RpcChannel;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};

pub trait RpcAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    fn connect_local(&self) -> impl std::future::Future<Output = Result<RpcChannel>> + Send;
}

impl<S> RpcAsyncService for ServiceRef<S>
where
    S: ActorService,
    S: ServiceHandler<S, ConnectLocal>,
{
    async fn connect_local(&self) -> Result<RpcChannel> {
        self.send(ConnectLocal).await
    }
}
