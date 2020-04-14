// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_config::NodeConfig;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

pub struct CliState {
    config: Arc<NodeConfig>,
    client: RpcClient,
    join_handle: Option<NodeHandle>,
}

impl CliState {
    pub fn new(
        config: Arc<NodeConfig>,
        client: RpcClient,
        join_handle: Option<NodeHandle>,
    ) -> CliState {
        Self {
            config,
            client,
            join_handle,
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn into_inner(self) -> (Arc<NodeConfig>, RpcClient, Option<NodeHandle>) {
        (self.config, self.client, self.join_handle)
    }
}
