// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::module::map_err;
use futures::future::TryFutureExt;
use futures::FutureExt;
use jsonrpc_core::Result;
use network_api::NetworkService;
use starcoin_config::NodeConfig;
use starcoin_metrics;
use starcoin_network::NetworkAsyncService;
use starcoin_rpc_api::node::{NodeApi, NodeInfo};
use starcoin_rpc_api::FutureResult;
use starcoin_types::peer_info::PeerInfo;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NodeRpcImpl {
    config: Arc<NodeConfig>,
    service: Option<NetworkAsyncService>,
}

impl NodeRpcImpl {
    pub fn new(config: Arc<NodeConfig>, service: Option<NetworkAsyncService>) -> Self {
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
        let self_address = self
            .config
            .network
            .self_address
            .as_ref()
            .expect("self address must exist in runtime.")
            .to_string();
        let fut = async move {
            let peer_info = service.get_self_peer().await?;
            let node_info = NodeInfo::new(peer_info, self_address);
            Ok(node_info)
        };
        Box::new(
            fut.map_err(|e: anyhow::Error| map_err(e.into()))
                .boxed()
                .compat(),
        )
    }

    fn peers(&self) -> FutureResult<Vec<PeerInfo>> {
        let service = self.service.clone().unwrap();
        let fut = async move { service.peer_set().await };
        Box::new(
            fut.map_err(|e: anyhow::Error| map_err(e.into()))
                .boxed()
                .compat(),
        )
    }

    fn metrics(&self) -> Result<HashMap<String, String>> {
        Ok(starcoin_metrics::get_all_metrics())
    }
}
