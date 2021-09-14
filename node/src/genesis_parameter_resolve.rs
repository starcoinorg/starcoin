// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use starcoin_config::ChainNetworkID;
use starcoin_config::{
    BuiltinNetworkID, FutureBlockParameter, FutureBlockParameterResolver, GenesisBlockParameter,
};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{Params, RpcClient, StateRootOption};
use starcoin_state_api::StateReaderExt;
use starcoin_types::block::BlockNumber;
use starcoin_types::U256;
use std::fmt::Write;
use std::time::Duration;

const WAIT_CONFORM_BLOCK: u64 = 6;

pub struct RpcFutureBlockParameterResolver {
    network: ChainNetworkID,
}

impl RpcFutureBlockParameterResolver {
    pub fn new(network: ChainNetworkID) -> Self {
        Self { network }
    }

    pub fn get_latest_block_number(client: &RpcClient) -> Result<BlockNumber> {
        Ok(client.chain_info()?.head.number.0)
    }

    pub fn get_genesis_parameter(
        client: &RpcClient,
        target_network: BuiltinNetworkID,
        block_number: BlockNumber,
    ) -> Result<GenesisBlockParameter> {
        match target_network {
            BuiltinNetworkID::Proxima => {
                //let params = json!({ "number": block_number });
                let mut map = serde_json::Map::new();
                map.insert(
                    "number".to_string(),
                    serde_json::Value::Number(block_number.into()),
                );
                let response = client.call_raw_api(
                    "chain.get_block_by_number",
                    Params::Array(vec![serde_json::Value::Number(block_number.into())]),
                )?;
                debug!("chain.get_block_by_number api response: {:?}", response);
                let response = response
                    .as_object()
                    .ok_or_else(|| format_err!("api response error:{:?}", response))?;
                let header = response
                    .get("header")
                    .and_then(|header| header.as_object())
                    .ok_or_else(|| format_err!("api response error:{:?}", response))?;
                let parent_hash = HashValue::from_hex_literal(
                    header
                        .get("block_hash")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| format_err!("api response error:{:?}", response))?,
                )?;
                let timestamp: u64 = header
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format_err!("api response error:{:?}", response))?
                    .parse()?;
                let difficulty: U256 = header
                    .get("difficulty")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| format_err!("api response error:{:?}", response))?
                    .parse()?;

                Ok(GenesisBlockParameter {
                    parent_hash,
                    timestamp,
                    difficulty,
                })
            }
            _ => match client.chain_get_block_by_number(block_number, None)? {
                Some(block) => Ok(GenesisBlockParameter {
                    parent_hash: block.header.block_hash,
                    timestamp: block.header.timestamp.0,
                    difficulty: block.header.difficulty,
                }),
                None => {
                    bail!("Can not get block by number:{}, retry.", block_number)
                }
            },
        }
    }
    const HOUR: u64 = 3600;
    const DAY: u64 = 24 * Self::HOUR;
    const MINUTES: u64 = 60;

    fn fmt_duration(duration: Duration) -> Result<String> {
        let mut result = String::new();
        let days = duration.as_secs() / Self::DAY;
        let hours = (duration.as_secs() - (days * Self::DAY)) / Self::HOUR;
        let minutes = (duration.as_secs() - (days * Self::DAY) - (hours * Self::HOUR)) / 60;
        let seconds = duration.as_secs()
            - (days * Self::DAY)
            - (hours * Self::HOUR)
            - (minutes * Self::MINUTES);
        if days > 0 {
            write!(&mut result, " {} days", days)?;
        }
        if days > 0 || hours > 0 {
            write!(&mut result, " {} hours", hours)?;
        }
        if days > 0 || hours > 0 || minutes > 0 {
            write!(&mut result, " {} minutes", minutes)?;
        }
        write!(&mut result, " {} seconds", seconds)?;
        Ok(result)
    }
}

impl FutureBlockParameterResolver for RpcFutureBlockParameterResolver {
    fn resolve(&self, parameter: &FutureBlockParameter) -> Result<GenesisBlockParameter> {
        let ws_rpc_url = format!("ws://{}:{}", parameter.network.boot_nodes_domain(), 9870);
        info!("Connect to {} for get genesis block parameter.", ws_rpc_url);
        let rpc_client: RpcClient = RpcClient::connect_websocket(ws_rpc_url.as_str())?;
        let state_reader = rpc_client.state_reader(StateRootOption::Latest)?;
        loop {
            match Self::get_latest_block_number(&rpc_client) {
                Ok(block_number) => {
                    info!(
                        "{}'s latest block number is {}",
                        parameter.network, block_number
                    );
                    if block_number >= parameter.block_number {
                        info!(
                            "{}'s latest block is match expect {} launch number:  {}",
                            parameter.network, self.network, parameter.block_number
                        );
                        if block_number < parameter.block_number + WAIT_CONFORM_BLOCK {
                            info!(
                                "Waiting {} blocks to conform.",
                                (parameter.block_number + WAIT_CONFORM_BLOCK) - block_number
                            );
                        } else {
                            match Self::get_genesis_parameter(
                                &rpc_client,
                                parameter.network,
                                parameter.block_number,
                            ) {
                                Ok(genesis_parameter) => {
                                    info!(
                                        "{} network ready to launch with parameter: {:?}",
                                        self.network, genesis_parameter
                                    );
                                    return Ok(genesis_parameter);
                                }
                                Err(e) => {
                                    warn!(
                                        "Get genesis block parameter by number:{}, return error:{:?}, retry.",
                                        parameter.block_number, e
                                    )
                                }
                            }
                        }
                    } else {
                        let epoch = state_reader.get_epoch()?;
                        let wait_milli_seconds =
                            (parameter.block_number - block_number) * epoch.block_time_target();
                        let duration = chrono::Duration::milliseconds(wait_milli_seconds as i64);
                        let utc_launch_time = chrono::Utc::now()
                            + chrono::Duration::milliseconds(wait_milli_seconds as i64);
                        let local_launch_time = chrono::Local::now()
                            + chrono::Duration::milliseconds(wait_milli_seconds as i64);
                        info!(
                            "Waiting to {}'s block {}, {} network will launch at {}, local time: {}, remaining {} (The time is estimated according to the current block time target[{} millisecond] of {} network)",
                            parameter.network, parameter.block_number, self.network, utc_launch_time, local_launch_time, Self::fmt_duration(duration.to_std()?)?, epoch.block_time_target(), parameter.network,
                        )
                    }
                }
                Err(e) => {
                    error!(
                        "Get {}'s latest block number error: {:?}",
                        parameter.network, e
                    );
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(5))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[stest::test]
    fn test_genesis_parameter_resolver() {
        let resolver = RpcFutureBlockParameterResolver::new(BuiltinNetworkID::Main.into());
        let parameter = FutureBlockParameter {
            network: BuiltinNetworkID::Barnard,
            block_number: 310000,
        };
        let genesis_parameter = resolver.resolve(&parameter).unwrap();
        debug!("{:?}", genesis_parameter);
    }
}
