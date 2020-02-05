// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};

mod network_config;

pub use network_config::NetworkConfig;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub network: NetworkConfig,
}

impl NodeConfig {
    pub fn load<P: AsRef<Path>>(input_path: P) -> Result<Self> {
        let mut file = File::open(&input_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::deserialize(&contents)
    }

    pub fn deserialize(serialized: &str) -> Result<Self> {
        Ok(toml::from_str(&serialized)?)
    }

    pub fn serialize(&self) -> Result<String> {
        let config_str = toml::to_string(self)?;
        return Ok(config_str);
    }

    pub fn load_or_default(config_path: Option<&Path>) -> Self {
        // Load the config
        let node_config = match config_path {
            Some(path) => {
                println!("Loading node config from: {}", path.display());
                NodeConfig::load(path).expect("Failed to load node config.")
            }
            None => {
                println!("Loading test configs");
                NodeConfig::default()
            }
        };

        println!("Using node config {:?}", &node_config);

        node_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serialize() {
        let config = NodeConfig::default();
        let config_str = config.serialize().expect("config serialize must success.");
        let config2 =
            NodeConfig::deserialize(config_str.as_str()).expect("config deserialize must success.");
        assert_eq!(config, config2);
    }
}
