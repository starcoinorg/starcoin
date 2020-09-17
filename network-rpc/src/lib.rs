// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rpc::NetworkRpcImpl;
use anyhow::Result;
use network_api::messages::RawRpcRequestMessage;
use network_rpc_core::server::NetworkRpcServer;
use network_rpc_core::RawRpcServer;
use starcoin_chain_service::ChainReaderService;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::gen_server::NetworkRpc;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_state_service::ChainStateService;
use starcoin_storage::{Storage, Store};
use std::sync::Arc;
use txpool::TxPoolService;

mod rpc;
#[cfg(test)]
mod tests;

pub use starcoin_network_rpc_api::gen_client;

pub struct NetworkRpcService {
    rpc_server: Arc<NetworkRpcServer>,
}

impl NetworkRpcService {
    pub fn new(
        storage: Arc<dyn Store>,
        chain_service: ServiceRef<ChainReaderService>,
        txpool_service: TxPoolService,
        state_service: ServiceRef<ChainStateService>,
    ) -> Self {
        let rpc_impl = NetworkRpcImpl::new(storage, chain_service, txpool_service, state_service);
        let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());
        Self {
            rpc_server: Arc::new(rpc_server),
        }
    }
}

impl ServiceFactory<Self> for NetworkRpcService {
    fn create(ctx: &mut ServiceContext<NetworkRpcService>) -> Result<NetworkRpcService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let chain_service = ctx.service_ref::<ChainReaderService>()?.clone();
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let state_service = ctx.service_ref::<ChainStateService>()?.clone();
        Ok(Self::new(
            storage,
            chain_service,
            txpool_service,
            state_service,
        ))
    }
}

impl ActorService for NetworkRpcService {}

impl ServiceHandler<Self, RawRpcRequestMessage> for NetworkRpcService {
    fn handle(
        &mut self,
        req_msg: RawRpcRequestMessage,
        ctx: &mut ServiceContext<NetworkRpcService>,
    ) {
        let rpc_server = self.rpc_server.clone();
        let (peer_id, rpc_path, message) = req_msg.request;
        let mut responder = req_msg.responder;
        ctx.spawn(async move {
            let result = rpc_server
                .handle_raw_request(peer_id, rpc_path, message)
                .await;
            let resp = scs::to_bytes(&result).expect("NetRpc Result must encode success.");

            if let Err(e) = responder.try_send(resp) {
                error!("Send response to rpc call failed:{:?}", e);
            }
        });
    }
}
