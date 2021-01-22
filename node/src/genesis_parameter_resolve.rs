// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetworkID;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_types::genesis_config::{
    FutureBlockParameter, FutureBlockParameterResolver, GenesisBlockParameter,
};
use std::time::Duration;

const WAIT_CONFORM_BLOCK: u64 = 6;

pub struct RpcFutureBlockParameterResolver {
    network: ChainNetworkID,
}

impl RpcFutureBlockParameterResolver {
    pub fn new(network: ChainNetworkID) -> Self {
        Self { network }
    }
}

impl FutureBlockParameterResolver for RpcFutureBlockParameterResolver {
    fn resolve(&self, parameter: &FutureBlockParameter) -> Result<GenesisBlockParameter> {
        let ws_rpc_url = format!("ws://{}:{}", parameter.network.boot_nodes_domain(), 9870);
        info!("Connect to {} for get genesis block parameter.", ws_rpc_url);
        let rpc_client: RpcClient = RpcClient::connect_websocket(ws_rpc_url.as_str())?;
        loop {
            match rpc_client.chain_info() {
                Ok(chain_info) => {
                    let block_number = chain_info.head.number.0;
                    info!(
                        "{}'s latest block number: {}",
                        parameter.network, block_number
                    );
                    if block_number >= parameter.block_number {
                        info!(
                            "{}'s latest block is match expect {} launch number:  {}",
                            parameter.network, self.network, parameter.block_number
                        );
                        if block_number < parameter.block_number + WAIT_CONFORM_BLOCK {
                            info!(
                                "Waiting {} blocks conform.",
                                (parameter.block_number + WAIT_CONFORM_BLOCK) - block_number
                            );
                        } else {
                            match rpc_client.chain_get_block_by_number(parameter.block_number) {
                                Ok(Some(block)) => {
                                    let genesis_parameter = GenesisBlockParameter {
                                        parent_hash: block.header.block_hash,
                                        timestamp: block.header.timestamp.0,
                                        difficulty: block.header.difficulty,
                                    };
                                    info!(
                                        "{} network ready to launch with parameter: {:?}",
                                        self.network, genesis_parameter
                                    );
                                    return Ok(genesis_parameter);
                                }
                                Ok(None) => {
                                    warn!(
                                        "Can not get block by number:{}, retry.",
                                        parameter.block_number
                                    )
                                }
                                Err(e) => {
                                    warn!(
                                        "Get block by number:{}, return error:{:?}, retry.",
                                        parameter.block_number, e
                                    )
                                }
                            }
                        }
                    } else {
                        //TODO read onchain block_time_target to estimate time.
                        let wait_milli_seconds = (parameter.block_number - block_number)
                            * parameter
                                .network
                                .genesis_config()
                                .consensus_config
                                .base_block_time_target;
                        let duration = Duration::from_millis(wait_milli_seconds);
                        info!(
                            "Waiting to {}'s block {}, about: {:?}",
                            parameter.network, parameter.block_number, duration
                        )
                    }
                }
                Err(e) => {
                    error!("Get {}'s chain_info error: {:?}", parameter.network, e);
                }
            }
            std::thread::sleep(Duration::from_secs(5))
        }
    }
}
