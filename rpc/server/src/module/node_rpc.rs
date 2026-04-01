// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{error::INTERNAL_ERROR_CODE, ErrorObjectOwned},
};
use network_api::PeerProvider;
use starcoin_config::NodeConfig;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_api::node::{NodeApiServer, NodeInfo};
use starcoin_rpc_api::types::PeerInfoView;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeRpcImpl {
    config: Arc<NodeConfig>,
    service: Option<NetworkServiceRef>,
}

impl NodeRpcImpl {
    pub fn new(config: Arc<NodeConfig>, service: Option<NetworkServiceRef>) -> Self {
        Self { config, service }
    }

    fn network_service(&self) -> RpcResult<NetworkServiceRef> {
        self.service.clone().ok_or_else(|| {
            ErrorObjectOwned::owned(
                INTERNAL_ERROR_CODE,
                "Network service unavailable",
                None::<()>,
            )
        })
    }
}

#[async_trait]
impl NodeApiServer for NodeRpcImpl {
    fn status(&self) -> RpcResult<bool> {
        //TODO check service status.
        Ok(true)
    }

    async fn info(&self) -> RpcResult<NodeInfo> {
        let service = self.network_service()?;
        let self_address = self.config.network.self_address().to_string();
        let net = self.config.net().clone();
        let peer_info = service
            .get_self_peer()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        //TODO read consensus_strategy from Epoch.
        let consensus_strategy = net.genesis_config2().consensus();
        let node_info = NodeInfo::new(
            peer_info.into(),
            self_address,
            net.id().clone(),
            consensus_strategy,
            net.time_service().now_secs(),
        );
        Ok(node_info)
    }

    async fn peers(&self) -> RpcResult<Vec<PeerInfoView>> {
        let service = self.network_service()?;
        let peers = service
            .peer_set()
            .await
            .map_err(crate::module::map_jsonrpc_err)?;
        Ok(peers
            .into_iter()
            .map(PeerInfoView::from)
            .collect::<Vec<_>>())
    }

    fn metrics(&self) -> RpcResult<HashMap<String, String>> {
        if let Some(registry) = self.config.metrics.registry() {
            Ok(starcoin_metrics::get_all_metrics(registry))
        } else {
            Ok(HashMap::new())
        }
    }
}
