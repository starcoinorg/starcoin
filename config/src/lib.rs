// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_vault_config::AccountVaultConfig;
use crate::sync_config::SyncConfig;
use anyhow::{ensure, format_err, Result};
use libp2p::core::Multiaddr;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use starcoin_logger::prelude::*;
use std::convert::TryFrom;
use std::fs::create_dir_all;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

mod account_vault_config;
mod available_port;
mod logger_config;
mod metrics_config;
mod miner_config;
mod network_config;
mod rpc_config;
mod storage_config;
mod sync_config;
mod txpool_config;

pub use available_port::{
    get_available_port_from, get_random_available_port, get_random_available_ports,
};

pub use libra_temppath::TempPath;
pub use logger_config::LoggerConfig;
pub use metrics_config::MetricsConfig;
pub use miner_config::{MinerClientConfig, MinerConfig};
pub use network_config::NetworkConfig;
pub use rpc_config::RpcConfig;
pub use starcoin_crypto::ed25519::genesis_key_pair;
pub use starcoin_vm_types::genesis_config::{
    BuiltinNetworkID, ChainNetwork, ChainNetworkID, ConsensusStrategy, GenesisConfig,
    StdlibVersion, DEV_CONFIG, HALLEY_CONFIG, MAIN_CONFIG, PROXIMA_CONFIG,
};
pub use starcoin_vm_types::time::{MockTimeService, RealTimeService, TimeService};
pub use storage_config::StorageConfig;
pub use sync_config::SyncMode;
pub use txpool_config::TxPoolConfig;

/// Default data dir
pub static DEFAULT_BASE_DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("read home dir should ok")
        .join(".starcoin")
});
pub static CONFIG_FILE_PATH: &str = "config.toml";
pub static GENESIS_CONFIG_FILE_NAME: &str = "genesis_config.json";

pub fn load_config_with_opt(opt: &StarcoinOpt) -> Result<NodeConfig> {
    NodeConfig::load_with_opt(opt)
}

pub fn temp_path() -> DataDirPath {
    let temp_path = TempPath::new();
    temp_path.create_as_dir().expect("Create temp dir fail.");
    DataDirPath::TempPath(Arc::from(temp_path))
}

pub fn temp_path_with_dir(dir: PathBuf) -> DataDirPath {
    let temp_path = TempPath::new_with_temp_dir(dir);
    temp_path.create_as_dir().expect("Create temp dir fail.");
    DataDirPath::TempPath(Arc::from(temp_path))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Connect {
    /// Connect by ipc file path, if Path is absent, use default ipc file.
    IPC(Option<PathBuf>),
    /// Connect by json rpc address.
    WebSocket(String),
}

impl Default for Connect {
    fn default() -> Self {
        Connect::IPC(None)
    }
}

impl FromStr for Connect {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::default());
        }
        if s.starts_with("ws://") || s.starts_with("wss://") {
            Ok(Connect::WebSocket(s.to_string()))
        } else {
            Ok(Connect::IPC(Some(PathBuf::from_str(s)?)))
        }
    }
}

static OPT_NET_HELP: &str = r#"Chain Network 
    Builtin network: test,dev,halley,proxima,main
    Custom network format: chain_name:chain_id
    Such as:  
    my_chain:123 will init a new chain with id `123`. 
    Custom network first start should also set the `genesis-config` option.
    Use starcoin_generator command to generate a genesis config."#;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin", about = "Starcoin")]
pub struct StarcoinOpt {
    #[structopt(long, short = "c")]
    /// Connect and attach to a node
    pub connect: Option<Connect>,

    #[structopt(long, short = "d", parse(from_os_str))]
    /// Path to data dir
    pub data_dir: Option<PathBuf>,

    #[structopt(long, short = "n", help = OPT_NET_HELP)]
    pub net: Option<ChainNetworkID>,

    #[structopt(long)]
    /// P2P network seed, if want add more seeds, please edit config file.
    pub seed: Option<Multiaddr>,

    #[structopt(long = "node-key")]
    /// Node network private key, only work for first init.
    pub node_key: Option<String>,

    #[structopt(long = "node-key-file", parse(from_os_str), conflicts_with("node-key"))]
    /// Node network private key file, only work for first init.
    pub node_key_file: Option<PathBuf>,

    #[structopt(long = "sync-mode", short = "s")]
    /// Sync mode. Included value(full, fast, light).
    pub sync_mode: Option<SyncMode>,

    #[structopt(long = "rpc-address")]
    /// Rpc address, default is 127.0.0.1
    pub rpc_address: Option<String>,

    #[structopt(long = "miner-thread")]
    /// Miner thread number, not work for dev network, default is 1
    pub miner_thread: Option<u16>,

    #[structopt(long = "disable-std-log")]
    /// Disable std error log output.
    pub disable_std_log: bool,

    #[structopt(long = "disable-file-log")]
    /// Disable std error log output.
    pub disable_file_log: bool,

    #[structopt(long = "disable-metrics")]
    /// Disable metrics.
    pub disable_metrics: bool,

    #[structopt(long = "disable-miner-client")]
    /// Don't start a miner client in node.
    pub disable_miner_client: bool,

    #[structopt(long = "disable-seed")]
    /// Disable seed for seed node.
    pub disable_seed: bool,

    #[structopt(long = "disable-mint-empty-block")]
    /// Do not mint empty block, default is true in Dev network.
    pub disable_mint_empty_block: Option<bool>,

    #[structopt(long = "watch-timeout")]
    /// Watch timeout in seconds
    pub watch_timeout: Option<u64>,

    #[structopt(long = "genesis-config")]
    /// Init chain by a custom genesis config. if want to reuse builtin network config, just pass a builtin network name.
    /// This option only work for node init start.
    pub genesis_config: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataDirPath {
    PathBuf(PathBuf),
    TempPath(Arc<TempPath>),
}

impl DataDirPath {
    pub fn path(&self) -> &Path {
        self.as_ref()
    }
    pub fn is_temp(&self) -> bool {
        match self {
            DataDirPath::TempPath(_) => true,
            _ => false,
        }
    }
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

#[derive(Clone, Debug, PartialEq)]
pub struct BaseConfig {
    net: ChainNetwork,
    base_data_dir: DataDirPath,
    data_dir: PathBuf,
}

impl BaseConfig {
    pub fn default_with_opt(opt: &StarcoinOpt) -> Result<Self> {
        let id = opt.net.clone().unwrap_or_default();
        let base_data_dir = opt.data_dir.clone();
        let base_data_dir = match base_data_dir {
            Some(base_data_dir) => DataDirPath::PathBuf(base_data_dir),
            None => {
                if id.is_dev() || id.is_test() {
                    temp_path()
                } else {
                    DataDirPath::PathBuf(DEFAULT_BASE_DATA_DIR.to_path_buf())
                }
            }
        };
        let data_dir = base_data_dir.as_ref().join(id.dir_name());
        if !data_dir.exists() {
            create_dir_all(data_dir.as_path())?;
        }
        let genesis_config = Self::load_genesis_config_by_opt(
            id.clone(),
            data_dir.as_path(),
            opt.genesis_config.clone(),
        )?;
        let net = ChainNetwork::new(id, genesis_config);
        Ok(Self {
            net,
            base_data_dir,
            data_dir,
        })
    }

    fn load_genesis_config_by_opt(
        id: ChainNetworkID,
        data_dir: &Path,
        genesis_config_name: Option<String>,
    ) -> Result<GenesisConfig> {
        let config_path = data_dir.join(GENESIS_CONFIG_FILE_NAME);
        let config_in_file = if config_path.exists() {
            Some(GenesisConfig::load(config_path.as_path())?)
        } else {
            None
        };
        let genesis_config = match (config_in_file, id) {
            (Some(config_in_file), ChainNetworkID::Builtin(net)) => {
                ensure!(
                    &config_in_file == net.genesis_config(),
                    "GenesisConfig in file:{:?} is not same with builtin config: {:?}",
                    config_path.as_path(),
                    net
                );
                config_in_file
            }
            (Some(config_in_file), ChainNetworkID::Custom(_net)) => config_in_file,
            (None, ChainNetworkID::Builtin(net)) => {
                //write genesis config to data_dir
                let genesis_config = net.genesis_config().clone();
                genesis_config.save(config_path.as_path())?;
                genesis_config
            }
            (None, ChainNetworkID::Custom(_net)) => {
                let config_name_or_path = genesis_config_name.ok_or_else(||format_err!("Can not load genesis config from {:?}, please set `genesis-config` cli option.", config_path))?;
                let genesis_config = match BuiltinNetworkID::from_str(config_name_or_path.as_str())
                {
                    Ok(net) => net.genesis_config().clone(),
                    Err(_) => {
                        let path = Path::new(config_name_or_path.as_str());
                        GenesisConfig::load(path)?
                    }
                };
                genesis_config.save(config_path.as_path())?;
                genesis_config
            }
        };
        Ok(genesis_config)
    }

    pub fn net(&self) -> &ChainNetwork {
        &self.net
    }
    pub fn data_dir(&self) -> &Path {
        self.data_dir.as_path()
    }
    pub fn base_data_dir(&self) -> DataDirPath {
        self.base_data_dir.clone()
    }
}

pub trait ConfigModule: Sized {
    /// Generate default config by the global command line option.
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self>;
    /// Init config after load config from file.
    /// Init the skip files or load external config from file, or overwrite config by global command line option.
    fn after_load(&mut self, _opt: &StarcoinOpt, _base: &BaseConfig) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(skip)]
    base: Option<BaseConfig>,
    pub network: NetworkConfig,
    pub rpc: RpcConfig,
    pub miner: MinerConfig,
    pub storage: StorageConfig,
    pub tx_pool: TxPoolConfig,
    pub sync: SyncConfig,
    pub vault: AccountVaultConfig,
    pub metrics: MetricsConfig,
    pub logger: LoggerConfig,
}

impl NodeConfig {
    pub fn random_for_test() -> Self {
        let mut opt = StarcoinOpt::default();
        opt.net = Some(BuiltinNetworkID::Test.into());
        Self::load_with_opt(&opt).expect("Auto generate test config should success.")
    }

    pub fn load_with_opt(opt: &StarcoinOpt) -> Result<Self> {
        let base = BaseConfig::default_with_opt(opt)?;
        let data_dir = base.data_dir();
        ensure!(data_dir.is_dir(), "please pass in a dir as data_dir");

        let config_file_path = data_dir.join(CONFIG_FILE_PATH);
        if !config_file_path.exists() {
            info!(
                "Config file not exist, generate default config to: {:?}",
                config_file_path
            );
            let config = NodeConfig::default_with_opt(opt, &base)?;
            save_config(&config, &config_file_path)?;
        }
        info!("Load config from: {:?}", config_file_path);
        let mut config: NodeConfig = match load_config(&config_file_path) {
            Ok(config) => config,
            Err(e) => {
                if base.net.is_dev() || base.net.is_test() || base.net.is_halley() {
                    info!("Load config error: {:?}, use default config.", e);
                    NodeConfig::default_with_opt(opt, &base)?
                } else {
                    return Err(e);
                }
            }
        };

        config.after_load(opt, &base)?;
        save_config(&config, &config_file_path)?;
        Ok(config)
    }

    pub fn data_dir(&self) -> &Path {
        self.base().data_dir()
    }

    pub fn net(&self) -> &ChainNetwork {
        self.base().net()
    }

    pub fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Base must exist after init.")
    }
}

impl NodeConfig {
    pub fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        Ok(Self {
            base: Some(base.clone()),
            network: NetworkConfig::default_with_opt(opt, &base)?,
            rpc: RpcConfig::default_with_opt(opt, &base)?,
            miner: MinerConfig::default_with_opt(opt, &base)?,
            storage: StorageConfig::default_with_opt(opt, &base)?,
            tx_pool: TxPoolConfig::default_with_opt(opt, &base)?,
            sync: SyncConfig::default_with_opt(opt, &base)?,
            vault: AccountVaultConfig::default_with_opt(opt, &base)?,
            metrics: MetricsConfig::default_with_opt(opt, &base)?,
            logger: LoggerConfig::default_with_opt(opt, &base)?,
        })
    }

    pub fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
        self.base = Some(base.clone());
        self.network.after_load(opt, base)?;
        self.rpc.after_load(opt, base)?;
        self.miner.after_load(opt, base)?;
        self.storage.after_load(opt, base)?;
        self.tx_pool.after_load(opt, base)?;
        self.sync.after_load(opt, base)?;
        self.vault.after_load(opt, base)?;
        self.metrics.after_load(opt, base)?;
        self.logger.after_load(opt, base)?;
        Ok(())
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

//TODO remove this method and remove KeyPair dependency.
pub fn gen_keypair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let mut gen = KeyGen::from_os_rng();
    let (private_key, public_key) = gen.generate_keypair();
    KeyPair {
        private_key,
        public_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_load() -> Result<()> {
        for net in BuiltinNetworkID::networks() {
            let mut opt = StarcoinOpt::default();
            let temp_path = temp_path();
            opt.net = Some(net.into());
            opt.data_dir = Some(temp_path.path().to_path_buf());

            let config = NodeConfig::load_with_opt(&opt)?;
            let config2 = NodeConfig::load_with_opt(&opt)?;
            assert_eq!(config, config2, "test config for network {} fail.", net);
        }
        Ok(())
    }

    #[test]
    fn test_custom_chain_genesis() -> Result<()> {
        let mut opt = StarcoinOpt::default();
        let net = ChainNetworkID::from_str("test1:123")?;
        let temp_path = temp_path();
        opt.net = Some(net);
        opt.data_dir = Some(temp_path.path().to_path_buf());
        opt.genesis_config = Some(BuiltinNetworkID::Test.to_string());
        let config = NodeConfig::load_with_opt(&opt)?;
        let config2 = NodeConfig::load_with_opt(&opt)?;
        assert_eq!(
            config, config2,
            "test config for network {:?} fail.",
            opt.net
        );
        Ok(())
    }

    #[test]
    fn test_genesis_config_save_and_load() -> Result<()> {
        let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
        genesis_config.timestamp = 1000;
        let temp_path = temp_path();
        let file_path = temp_path.path().join(GENESIS_CONFIG_FILE_NAME);
        genesis_config.save(file_path.as_path())?;
        let genesis_config2 = GenesisConfig::load(file_path.as_path())?;
        assert_eq!(genesis_config, genesis_config2);
        Ok(())
    }
}
