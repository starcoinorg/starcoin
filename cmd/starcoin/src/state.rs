// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_config::NodeConfig;
use starcoin_logger::LoggerHandle;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::thread::JoinHandle;

pub struct CliState {
    config: Arc<NodeConfig>,
    client: RpcClient,
    logger_handle: LoggerHandle,
    join_handler: Option<JoinHandle<()>>,
}

impl CliState {
    pub fn new(
        config: Arc<NodeConfig>,
        client: RpcClient,
        logger_handle: LoggerHandle,
        join_handler: Option<JoinHandle<()>>,
    ) -> CliState {
        Self {
            config,
            client,
            logger_handle,
            join_handler,
        }
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn client(&self) -> &RpcClient {
        &self.client
    }

    pub fn logger(&self) -> &LoggerHandle {
        &self.logger_handle
    }

    pub fn into_inner(
        self,
    ) -> (
        Arc<NodeConfig>,
        RpcClient,
        LoggerHandle,
        Option<JoinHandle<()>>,
    ) {
        (
            self.config,
            self.client,
            self.logger_handle,
            self.join_handler,
        )
    }
}
