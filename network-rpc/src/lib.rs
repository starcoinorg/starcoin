// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rpc::NetworkRpcImpl;
use actix::Addr;
use anyhow::Result;
use chain::ChainActorRef;
use futures::channel::mpsc;
use network_api::messages::RawRpcRequestMessage;
use network_rpc_core::server::NetworkRpcServer;
use starcoin_network_rpc_api::gen_server::NetworkRpc;
use state_api::ChainStateAsyncService;
use std::sync::Arc;
use storage::Store;
use txpool::TxPoolService;

mod rpc;
#[cfg(test)]
mod tests;

pub use starcoin_network_rpc_api::gen_client;

pub fn start_network_rpc_server<S>(
    rpc_rx: mpsc::UnboundedReceiver<RawRpcRequestMessage>,
    chain: ChainActorRef,
    storage: Arc<dyn Store>,
    state_service: S,
    txpool: TxPoolService,
) -> Result<Addr<NetworkRpcServer>>
where
    S: ChainStateAsyncService + 'static,
{
    let rpc_impl = NetworkRpcImpl::new(chain, txpool, state_service, storage);
    NetworkRpcServer::start(rpc_rx, rpc_impl.to_delegate())
}
