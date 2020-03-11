// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod miner_config;
mod network_config;
mod rpc_config;
mod vm_config;
mod storage_config;

pub use miner_config::{MinerConfig, PacemakerStrategy};
pub use network_config::NetworkConfig;
pub use rpc_config::RpcConfig;
pub use vm_config::VMConfig;
pub use storage_config::StorageConfig;

use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::{test_utils::KeyPair, Uniform};
use rand::prelude::*;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use dirs;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub base: BaseConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub rpc: RpcConfig,
    #[serde(default)]
    pub vm: VMConfig,
    #[serde(default)]
    pub miner: MinerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
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
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct BaseConfig {
    #[serde(default)]
    pub data_dir: PathBuf,
}
impl Default for BaseConfig {
    fn default() -> BaseConfig {
        let home_dir: PathBuf = dirs::home_dir().expect("should get home dir");
        let default_data_dir = home_dir.join(".starcoin/starcoin");
        BaseConfig {
            data_dir: default_data_dir,
        }
    }
}

pub fn save_config<T, P>(c: &T, output_file: P) -> Result<()>  where T: Serialize + DeserializeOwned, P: AsRef<Path> {
    let contents = toml::to_vec(c)?;
    let mut file = File::create(output_file)?;
    file.write_all(&contents)?;
    Ok(())
}

pub fn gen_keypair() -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng0: StdRng = SeedableRng::from_seed(seed_buf);
    let account_keypair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> =
        Arc::new(KeyPair::generate_for_testing(&mut rng0));
    account_keypair
}

pub fn get_available_port() -> u16 {
    const MAX_PORT_RETRIES: u32 = 1000;

    for _ in 0..MAX_PORT_RETRIES {
        if let Ok(port) = get_ephemeral_port() {
            return port;
        }
    }

    panic!("Error: could not find an available port");
}

fn get_ephemeral_port() -> ::std::io::Result<u16> {
    use std::net::{TcpListener, TcpStream};

    // Request a random available port from the OS
    let listener = TcpListener::bind(("localhost", 0))?;
    let addr = listener.local_addr()?;

    // Create and accept a connection (which we'll promptly drop) in order to force the port
    // into the TIME_WAIT state, ensuring that the port will be reserved from some limited
    // amount of time (roughly 60s on some Linux systems)
    let _sender = TcpStream::connect(addr)?;
    let _incoming = listener.accept()?;

    Ok(addr.port())
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
