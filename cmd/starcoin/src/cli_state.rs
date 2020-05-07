// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_config::ChainNetwork;
use starcoin_node::NodeHandle;
use starcoin_rpc_client::RpcClient;

pub struct CliState {
    net: ChainNetwork,
    client: RpcClient,
    join_handle: Option<NodeHandle>,
}

impl CliState {
    pub fn new(net: ChainNetwork, client: RpcClient, join_handle: Option<NodeHandle>) -> CliState {
        Self {
            net,
            client,
            join_handle,
        }
    }

    pub fn net(&self) -> ChainNetwork {
        self.net
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn into_inner(self) -> (ChainNetwork, RpcClient, Option<NodeHandle>) {
        (self.net, self.client, self.join_handle)
    }
}
