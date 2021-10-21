// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use jsonrpc_core::Result;
use network_api::PeerProvider;
use starcoin_config::NodeConfig;
use starcoin_network::NetworkServiceRef;
use starcoin_rpc_api::node::{NodeApi, NodeInfo};
use starcoin_rpc_api::types::PeerInfoView;
use starcoin_rpc_api::FutureResult;
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
}

impl NodeApi for NodeRpcImpl {
    fn status(&self) -> Result<bool> {
        //TODO check service status.
        Ok(true)
    }

    fn info(&self) -> FutureResult<NodeInfo> {
        let service = self.service.clone().unwrap();
        let self_address = self.config.network.self_address().to_string();
        let net = self.config.net().clone();
        let fut = async move {
            let peer_info = service.get_self_peer().await?;
            //TODO read consensus_strategy from Epoch.
            let consensus_strategy = net.genesis_config().consensus();
            let node_info = NodeInfo::new(
                peer_info.into(),
                self_address,
                net.id().clone(),
                consensus_strategy,
                net.time_service().now_secs(),
            );
            Ok(node_info)
        };
        Box::pin(fut.map_err(map_err).boxed())
    }

    fn peers(&self) -> FutureResult<Vec<PeerInfoView>> {
        let service = self.service.clone().unwrap();
        let fut = async move {
            let peers = service.peer_set().await?;
            Ok(peers
                .into_iter()
                .map(PeerInfoView::from)
                .collect::<Vec<_>>())
        };
        Box::pin(fut.map_err(map_err).boxed())
    }

    fn metrics(&self) -> Result<HashMap<String, String>> {
        if let Some(registry) = self.config.metrics.registry() {
            Ok(starcoin_metrics::get_all_metrics(registry))
        } else {
            Ok(HashMap::new())
        }
    }
}
