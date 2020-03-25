// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_config::NodeConfig;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct CliState {
    config: Arc<NodeConfig>,
    client: RpcClient,
    join_handler: Option<JoinHandle<()>>,
}

impl CliState {
    pub fn new(
        config: Arc<NodeConfig>,
        client: RpcClient,
        join_handler: Option<JoinHandle<()>>,
    ) -> CliState {
        Self {
            config,
            client,
            join_handler,
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn into_inner(self) -> (Arc<NodeConfig>, RpcClient, Option<JoinHandle<()>>) {
        (self.config, self.client, self.join_handler)
    }
}
