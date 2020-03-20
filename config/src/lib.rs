// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::ensure;
use anyhow::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::TryFrom;
mod miner_config;
mod network_config;
mod rpc_config;
mod storage_config;
mod txpool_config;
mod vm_config;

pub use miner_config::{MinerConfig, PacemakerStrategy};
pub use network_config::NetworkConfig;
pub use rpc_config::RpcConfig;
pub use storage_config::StorageConfig;
pub use txpool_config::TxPoolConfig;
pub use vm_config::VMConfig;

use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::{test_utils::KeyPair, Uniform};
use dirs;
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::env;
use std::fs::create_dir;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default data dir
pub static DEFAULT_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("read home dir should ok")
        .join(".starcoin")
});
pub static CONFIG_FILE_PATH: &str = "config.toml";

pub fn load_config_from_dir<P>(data_dir: P) -> Result<NodeConfig>
where
    P: AsRef<Path>,
{
    NodeConfig::load(data_dir)
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(skip)]
    pub data_dir: PathBuf,
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
    #[serde(default)]
    pub tx_pool: TxPoolConfig,
}

impl NodeConfig {
    pub fn random_for_test() -> Self {
        let mut config = NodeConfig::default();
        config.data_dir = env::temp_dir();
        config.network = NetworkConfig::random_for_test();
        config.tx_pool = TxPoolConfig::random_for_test();
        config.rpc = RpcConfig::random_for_test();
        config.miner = MinerConfig::random_for_test();
        config
    }

    pub fn load<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        if !data_dir.as_ref().exists() {
            create_dir(data_dir.as_ref())?;
        }
        ensure!(
            data_dir.as_ref().is_dir(),
            "pelase pass in a dir as data_dir"
        );

        let base_dir = PathBuf::from(data_dir.as_ref());
        let config_file_path = base_dir.join(CONFIG_FILE_PATH);

        let mut node_config: NodeConfig = if config_file_path.exists() {
            load_config(&config_file_path)?
        } else {
            let default_config = NodeConfig::default();
            save_config(&default_config, &config_file_path)?;
            default_config
        };
        node_config.data_dir = base_dir.clone();
        //TODO every config should know the data_dir self.
        node_config.network.load(&base_dir)?;
        node_config.tx_pool.load()?;
        node_config.miner.load(&base_dir)?;
        // NOTICE: if there is more load case, make it here.
        // such as: node_config.storage.load(&base_dir)?;
        Ok(node_config)
    }

    pub fn load_or_random(config_path: Option<&Path>) -> Self {
        // Load the config
        let node_config = match config_path {
            Some(path) => NodeConfig::load(path).expect("Failed to load node config."),
            None => NodeConfig::random_for_test(),
        };
        node_config
    }
}

pub(crate) fn save_config<T, P>(c: &T, output_file: P) -> Result<()>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
{
    let contents = toml::to_vec(c)?;
    let mut file = File::create(output_file)?;
    file.write_all(&contents)?;
    Ok(())
}
pub(crate) fn load_config<T, P>(path: P) -> Result<T>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
{
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse(&contents)
}

fn parse<T>(serialized: &str) -> Result<T>
where
    T: Serialize + DeserializeOwned,
{
    Ok(toml::from_str(&serialized)?)
}

pub(crate) fn save_key<P>(key: &[u8], output_file: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let contents: String = hex::encode(key);
    let mut file = File::create(output_file)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub(crate) fn load_key<P: AsRef<Path>>(
    path: P,
) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    let content = std::fs::read_to_string(path)?;
    let bytes_out: Vec<u8> = hex::decode(&content)?;
    let pri_key = Ed25519PrivateKey::try_from(bytes_out.as_slice())?;
    Ok(KeyPair::from(pri_key))
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
    const MAX_PORT_RETRIES: u32 = 30000;
    const MIN_PORT_RETRIES: u32 = 1024;

    for _ in MIN_PORT_RETRIES..MAX_PORT_RETRIES {
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
