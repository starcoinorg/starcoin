// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rpc::NetworkRpcImpl;
use anyhow::Result;
use network_rpc_core::server::NetworkRpcServer;
use network_rpc_core::RawRpcServer;
use starcoin_chain_service::ChainReaderService;
use starcoin_logger::prelude::*;
use starcoin_network_rpc_api::gen_server::NetworkRpc;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_state_service::ChainStateService;
use starcoin_storage::{Storage, Store};
use std::sync::Arc;
use txpool::TxPoolService;

mod rpc;
#[cfg(test)]
mod tests;

use network_p2p_types::ProtocolRequest;
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

impl EventHandler<Self, ProtocolRequest> for NetworkRpcService {
    fn handle_event(&mut self, msg: ProtocolRequest, ctx: &mut ServiceContext<Self>) {
        let rpc_server = self.rpc_server.clone();
        ctx.spawn(async move {
            //TODO use Cow to replace String.
            let result = rpc_server
                .handle_raw_request(
                    msg.request.peer.into(),
                    msg.protocol.to_string(),
                    msg.request.payload,
                )
                .await;
            let resp = scs::to_bytes(&result).expect("NetRpc Result must encode success.");

            if let Err(e) = msg.request.pending_response.send(resp) {
                //TODO change log level
                warn!("Send response to rpc call failed:{:?}", e);
            }
        });
    }
}
