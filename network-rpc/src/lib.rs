// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::rpc::NetworkRpcImpl;
use anyhow::Result;
use api_limiter::{ApiLimiters, Quota};
use config::ApiQuotaConfig;
use config::NetworkRpcQuotaConfiguration;
use config::NodeConfig;
use config::QuotaDuration;

use network_p2p_types::{OutgoingResponse, ProtocolRequest};
use network_rpc_core::server::NetworkRpcServer;
use network_rpc_core::{NetRpcError, RawRpcServer, RpcErrorCode};
use starcoin_chain_service::ChainReaderService;
use starcoin_logger::prelude::*;
pub use starcoin_network_rpc_api::gen_client;
use starcoin_network_rpc_api::gen_server::NetworkRpc;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_state_service::ChainStateService;
use starcoin_storage::{Storage, Store};
use starcoin_types::peer_info::{PeerId, RpcInfo};
use std::sync::Arc;
use txpool::TxPoolService;

mod rpc;
#[cfg(test)]
mod tests;

struct QuotaWrapper(Quota);

impl From<ApiQuotaConfig> for QuotaWrapper {
    fn from(c: ApiQuotaConfig) -> Self {
        let q = match c.duration {
            QuotaDuration::Second => Quota::per_second(c.max_burst),
            QuotaDuration::Minute => Quota::per_minute(c.max_burst),
            QuotaDuration::Hour => Quota::per_hour(c.max_burst),
        };
        QuotaWrapper(q)
    }
}

pub struct NetworkRpcService {
    rpc_server: Arc<NetworkRpcServer>,
    rpc_limiters: Arc<ApiLimiters<String, PeerId>>,
}

impl NetworkRpcService {
    pub fn new(
        storage: Arc<dyn Store>,
        chain_service: ServiceRef<ChainReaderService>,
        txpool_service: TxPoolService,
        state_service: ServiceRef<ChainStateService>,
        quotas: NetworkRpcQuotaConfiguration,
    ) -> Self {
        let rpc_impl = NetworkRpcImpl::new(storage, chain_service, txpool_service, state_service);
        let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());

        let limiters = ApiLimiters::new(
            Into::<QuotaWrapper>::into(quotas.default_global_api_quota()).0,
            quotas
                .custom_global_api_quota()
                .into_iter()
                .map(|(k, v)| (k, Into::<QuotaWrapper>::into(v).0))
                .collect(),
            Into::<QuotaWrapper>::into(quotas.default_user_api_quota()).0,
            quotas
                .custom_user_api_quota()
                .into_iter()
                .map(|(k, v)| (k, Into::<QuotaWrapper>::into(v).0))
                .collect(),
        );
        Self {
            rpc_server: Arc::new(rpc_server),
            rpc_limiters: Arc::new(limiters),
        }
    }
}

impl ServiceFactory<Self> for NetworkRpcService {
    fn create(ctx: &mut ServiceContext<NetworkRpcService>) -> Result<NetworkRpcService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let chain_service = ctx.service_ref::<ChainReaderService>()?.clone();
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let state_service = ctx.service_ref::<ChainStateService>()?.clone();
        let node_config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let quotas = node_config.network.network_rpc_quotas.clone();
        Ok(Self::new(
            storage,
            chain_service,
            txpool_service,
            state_service,
            quotas,
        ))
    }
}

impl ActorService for NetworkRpcService {}

impl EventHandler<Self, ProtocolRequest> for NetworkRpcService {
    fn handle_event(&mut self, msg: ProtocolRequest, ctx: &mut ServiceContext<Self>) {
        let rpc_server = self.rpc_server.clone();
        let api_limiters = self.rpc_limiters.clone();
        ctx.spawn(async move {
            let protocol = msg.protocol;
            let rpc_path =
                RpcInfo::rpc_path(protocol).expect("get rpc path from protocol must success.");
            let peer = msg.request.peer.into();
            let result = match api_limiters.check(&rpc_path, Some(&peer)) {
                Err(e) => Err(NetRpcError::new(RpcErrorCode::RateLimited, e.to_string())),
                Ok(_) => {
                    rpc_server
                        .handle_raw_request(peer, rpc_path.into(), msg.request.payload)
                        .await
                }
            };

            let resp = bcs_ext::to_bytes(&result).expect("NetRpc Result must encode success.");
            //TODO: update reputation_changes
            if let Err(e) = msg.request.pending_response.send(OutgoingResponse {
                result: Ok(resp),
                reputation_changes: vec![],
            }) {
                error!("Send response to rpc call failed:{:?}", e);
            }
        });
    }
}
