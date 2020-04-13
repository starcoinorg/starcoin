// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use dirs;
use libp2p::core::Multiaddr;
use once_cell::sync::Lazy;
use rand::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    Uniform,
};
use std::convert::TryFrom;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use structopt::StructOpt;

mod account_vault_config;
mod chain_config;
mod miner_config;
mod network_config;
mod rpc_config;
mod storage_config;
mod sync_config;
mod txpool_config;
mod vm_config;

use crate::account_vault_config::AccountVaultConfig;
use crate::sync_config::SyncConfig;
pub use chain_config::{ChainConfig, ChainNetwork, PreMineConfig};
pub use libra_temppath::TempPath;
pub use miner_config::{MinerConfig, PacemakerStrategy};
pub use network_config::NetworkConfig;
pub use rpc_config::RpcConfig;
pub use storage_config::StorageConfig;
pub use txpool_config::TxPoolConfig;
pub use vm_config::VMConfig;

/// Default data dir
static DEFAULT_BASE_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("read home dir should ok")
        .join(".starcoin")
});
pub static CONFIG_FILE_PATH: &str = "config.toml";

pub fn load_config_with_opt(opt: &StarcoinOpt) -> Result<NodeConfig> {
    NodeConfig::load_with_opt(opt)
}

pub fn temp_path() -> TempPath {
    let temp_path = TempPath::new();
    temp_path.create_as_dir().expect("Create temp dir fail.");
    return temp_path;
}

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin", about = "Starcoin")]
pub struct StarcoinOpt {
    #[structopt(long, short = "d", parse(from_os_str))]
    /// Path to data dir
    pub data_dir: Option<PathBuf>,

    #[structopt(long, short = "n", default_value = "dev")]
    /// Chain Network
    pub net: ChainNetwork,

    #[structopt(long)]
    /// P2P network seeds
    pub seeds: Option<Vec<Multiaddr>>,

    #[structopt(long = "dev-period", default_value = "0")]
    /// Block period in second to use in dev network mode (0 = mine only if transaction pending)
    pub dev_period: u64,

    #[structopt(long = "node-key")]
    /// Node network private key, only work for first init.
    pub node_key: Option<String>,

    #[structopt(long = "node-key-file", parse(from_os_str), conflicts_with("node-key"))]
    /// Node network private key file, only work for first init.
    pub node_key_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataDirPath {
    PathBuf(PathBuf),
    TempPath(Arc<TempPath>),
}

impl AsRef<Path> for DataDirPath {
    fn as_ref(&self) -> &Path {
        match self {
            DataDirPath::PathBuf(path) => path.as_ref(),
            DataDirPath::TempPath(path) => path.as_ref().as_ref(),
        }
    }
}

impl Default for DataDirPath {
    fn default() -> Self {
        DataDirPath::PathBuf(DEFAULT_BASE_DATA_DIR.to_path_buf())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BaseConfig {
    #[serde(default)]
    net: ChainNetwork,
    #[serde(skip)]
    base_data_dir: DataDirPath,
    #[serde(skip)]
    data_dir: PathBuf,
}

impl BaseConfig {
    pub fn new(net: ChainNetwork, base_data_dir: Option<PathBuf>) -> Self {
        let base_data_dir = match base_data_dir {
            Some(base_data_dir) => DataDirPath::PathBuf(base_data_dir),
            None => {
                if net.is_dev() {
                    DataDirPath::TempPath(Arc::new(temp_path()))
                } else {
                    DataDirPath::PathBuf(DEFAULT_BASE_DATA_DIR.to_path_buf())
                }
            }
        };
        let data_dir = base_data_dir.as_ref().join(net.to_string());
        if !data_dir.exists() {
            create_dir_all(data_dir.as_path())
                .expect(format!("Create data dir {:?} fail.", data_dir).as_str());
        }
        Self {
            net,
            base_data_dir,
            data_dir,
        }
    }

    pub fn net(&self) -> ChainNetwork {
        self.net
    }
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }
    pub fn base_data_dir(&self) -> &Path {
        self.base_data_dir.as_ref()
    }
    pub fn random_for_test() -> Self {
        Self::new(ChainNetwork::Dev, None)
    }
}

impl Default for BaseConfig {
    fn default() -> Self {
        let net = ChainNetwork::default();
        BaseConfig::new(net, None)
    }
}

pub trait ConfigModule {
    /// Generate default config by ChainNetWork
    fn default_with_net(net: ChainNetwork) -> Self;
    /// Init config with random for test, such as ports.
    fn random(&mut self, _base: &BaseConfig) {}
    /// Init config with load, read or generate additional config for file
    /// and overwrite config by global command line option.
    fn load(&mut self, _base: &BaseConfig, _opt: &StarcoinOpt) -> Result<()> {
        Ok(())
    }
}

//TODO rename NodeConfig to Config.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
    #[serde(default)]
    pub tx_pool: TxPoolConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub vault: AccountVaultConfig,
}

impl NodeConfig {
    pub fn random_for_test() -> Self {
        let mut config = NodeConfig::default_with_net(ChainNetwork::Dev);
        let base = BaseConfig::random_for_test();
        config.random(&base);
        config
    }

    pub fn load_with_opt(opt: &StarcoinOpt) -> Result<Self> {
        let base = BaseConfig::new(opt.net, opt.data_dir.clone());
        let data_dir = base.data_dir();
        ensure!(data_dir.is_dir(), "please pass in a dir as data_dir");

        let config_file_path = data_dir.join(CONFIG_FILE_PATH);

        let mut config: NodeConfig = if config_file_path.exists() {
            load_config(&config_file_path)?
        } else {
            let default_config = NodeConfig::default_with_net(base.net);
            default_config
        };
        config.load(&base, opt)?;
        save_config(&config, &config_file_path)?;
        Ok(config)
    }

    pub fn data_dir(&self) -> &Path {
        self.base.data_dir()
    }

    pub fn net(&self) -> ChainNetwork {
        self.base.net
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

impl ConfigModule for NodeConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let base = BaseConfig::new(net, None);
        Self {
            base,
            network: NetworkConfig::default_with_net(net),
            rpc: RpcConfig::default_with_net(net),
            vm: VMConfig::default_with_net(net),
            miner: MinerConfig::default_with_net(net),
            storage: StorageConfig::default_with_net(net),
            tx_pool: TxPoolConfig::default_with_net(net),
            sync: SyncConfig::default_with_net(net),
            vault: AccountVaultConfig::default_with_net(net),
        }
    }

    fn random(&mut self, base: &BaseConfig) {
        self.base = base.clone();
        self.network.random(base);
        self.rpc.random(base);
        self.vm.random(base);
        self.miner.random(base);
        self.storage.random(base);
        self.tx_pool.random(base);
        self.sync.random(base);
        self.vault.random(base);
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        self.base = base.clone();
        self.network.load(base, opt)?;
        self.rpc.load(base, opt)?;
        self.miner.load(base, opt)?;
        self.storage.load(base, opt)?;
        self.tx_pool.load(base, opt)?;
        self.sync.load(base, opt)?;
        self.vault.load(base, opt)?;
        Ok(())
    }
}

impl NodeConfig {}

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

pub(crate) fn decode_key(hex_str: &str) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    let bytes_out: Vec<u8> = hex::decode(hex_str)?;
    let pri_key = Ed25519PrivateKey::try_from(bytes_out.as_slice())?;
    Ok(KeyPair::from(pri_key))
}

pub(crate) fn load_key<P: AsRef<Path>>(
    path: P,
) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    let content = std::fs::read_to_string(path)?;
    decode_key(content.as_str())
}

pub fn gen_keypair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng0: StdRng = SeedableRng::from_seed(seed_buf);
    KeyPair::generate_for_testing(&mut rng0)
}

pub fn get_available_port() -> u16 {
    for _ in 0..3 {
        if let Ok(port) = get_ephemeral_port() {
            return port;
        }
    }
    panic!("Error: could not find an available port");
}

pub fn get_available_port_multi(num: usize) -> Vec<u16> {
    let mut ports = vec![0u16; num];

    for i in 0..num {
        let mut port = get_available_port();
        let mut retry_times = 0;
        while ports.contains(&port) {
            port = get_available_port();
            retry_times = retry_times + 1;
            if retry_times > 3 {
                panic!("Error: could not find an available port");
            }
        }
        ports[i] = port;
    }
    return ports;
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
    fn test_serialize() -> Result<()> {
        let mut opt = StarcoinOpt::default();
        let config = NodeConfig::load_with_opt(&opt)?;
        opt.data_dir = Some(config.base.base_data_dir().to_path_buf());
        let config2 = NodeConfig::load_with_opt(&opt)?;
        let config3 = NodeConfig::load_with_opt(&opt)?;
        assert_eq!(config2, config3);
        Ok(())
    }
}
