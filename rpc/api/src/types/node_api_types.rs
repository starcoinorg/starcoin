// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_config::ChainNetworkID;
use std::str::FromStr;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChainId {
    pub name: String,
    pub id: u8,
}

impl From<&ChainNetworkID> for ChainId {
    fn from(id: &ChainNetworkID) -> Self {
        match id {
            ChainNetworkID::Builtin(t) => Self {
                name: t.chain_name(),
                id: t.chain_id().id(),
            },
            ChainNetworkID::Custom(t) => Self {
                name: t.chain_name().to_string(),
                id: t.chain_id().id(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum FactoryAction {
    Status,
    Stop,
    Start,
}
impl FromStr for FactoryAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "stop" => Self::Stop,
            "start" => Self::Start,
            _ => Self::Status, //default is status action
        })
    }
}
