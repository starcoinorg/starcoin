// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_vault_config::AccountVaultConfig;
use crate::sync_config::SyncConfig;
use anyhow::{ensure, format_err, Result};
use git_version::git_version;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::keygen::KeyGen;
use starcoin_logger::prelude::*;
use std::convert::TryFrom;
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use structopt::clap::crate_version;
use structopt::StructOpt;

mod account_vault_config;
mod api_config;
mod api_quota;
mod available_port;
mod logger_config;
mod metrics_config;
mod miner_config;
mod network_config;
mod rpc_config;
mod storage_config;
mod sync_config;
mod txpool_config;

pub use api_config::{Api, ApiSet};
pub use api_quota::{ApiQuotaConfig, QuotaDuration};
pub use available_port::{
    get_available_port_from, get_random_available_port, get_random_available_ports,
};
pub use diem_temppath::TempPath;
pub use logger_config::LoggerConfig;
pub use metrics_config::MetricsConfig;
pub use miner_config::{MinerClientConfig, MinerConfig};
use names::{Generator, Name};
pub use network_config::{NetworkConfig, NetworkRpcQuotaConfiguration};
pub use rpc_config::{
    ApiQuotaConfiguration, HttpConfiguration, IpcConfiguration, RpcConfig, TcpConfiguration,
    WsConfiguration,
};
pub use starcoin_crypto::ed25519::genesis_key_pair;
pub use starcoin_vm_types::genesis_config::{
    BuiltinNetworkID, ChainNetwork, ChainNetworkID, ConsensusStrategy, GenesisConfig,
    StdlibVersion, DEV_CONFIG, HALLEY_CONFIG, MAIN_CONFIG, PROXIMA_CONFIG,
};
pub use starcoin_vm_types::time::{MockTimeService, RealTimeService, TimeService};
pub use storage_config::{RocksdbConfig, StorageConfig};
pub use txpool_config::TxPoolConfig;

pub static CRATE_VERSION: &str = crate_version!();
pub static GIT_VERSION: &str = git_version!(
    args = ["--tags", "--dirty", "--always"],
    fallback = "unknown"
);

pub static APP_NAME: &str = "starcoin";
pub static APP_VERSION: Lazy<String> = Lazy::new(|| {
    if GIT_VERSION != "unknown" {
        format!("{} (build:{})", CRATE_VERSION, GIT_VERSION)
    } else {
        CRATE_VERSION.to_string()
    }
});

pub static APP_NAME_WITH_VERSION: Lazy<String> =
    Lazy::new(|| format!("{}/{}", APP_NAME, APP_VERSION.clone()));

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

/// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn std::error::Error>>
where
    T: std::str::FromStr,
    T::Err: Into<Box<dyn std::error::Error + 'static>>,
    U: std::str::FromStr,
    U::Err: Into<Box<dyn std::error::Error + 'static>>,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((
        s[..pos].parse().map_err(Into::into)?,
        s[pos + 1..].parse().map_err(Into::into)?,
    ))
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

#[derive(Clone, Debug, StructOpt, Default, Serialize, Deserialize)]
#[structopt(name = "starcoin", about = "Starcoin")]
pub struct StarcoinOpt {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long, short = "c")]
    /// Connect and attach to a node
    pub connect: Option<Connect>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long, short = "d", parse(from_os_str))]
    /// Path to data dir
    pub data_dir: Option<PathBuf>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long, short = "n", help = OPT_NET_HELP)]
    pub net: Option<ChainNetworkID>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "watch-timeout")]
    /// Watch timeout in seconds
    pub watch_timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "genesis-config")]
    /// Init chain by a custom genesis config. if want to reuse builtin network config, just pass a builtin network name.
    /// This option only work for node init start.
    pub genesis_config: Option<String>,

    #[structopt(flatten)]
    pub rpc: RpcConfig,
    #[structopt(flatten)]
    pub logger: LoggerConfig,
    #[structopt(flatten)]
    pub metrics: MetricsConfig,
    #[structopt(flatten)]
    pub miner: MinerConfig,
    #[structopt(flatten)]
    pub network: NetworkConfig,
    #[structopt(flatten)]
    pub txpool: TxPoolConfig,
    #[structopt(flatten)]
    pub storage: StorageConfig,
    #[structopt(flatten)]
    pub sync: SyncConfig,
}

impl std::fmt::Display for StarcoinOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
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
        matches!(self, DataDirPath::TempPath(_))
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
    /// Init the skip field or overwrite config by global command line option.
    fn merge_with_opt(&mut self, _opt: &StarcoinOpt, _base: Arc<BaseConfig>) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(skip)]
    base: Option<Arc<BaseConfig>>,
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

impl std::fmt::Display for NodeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_e| std::fmt::Error)?
        )
    }
}

impl NodeConfig {
    pub fn random_for_test() -> Self {
        let opt = StarcoinOpt {
            net: Some(BuiltinNetworkID::Test.into()),
            ..StarcoinOpt::default()
        };
        Self::load_with_opt(&opt).expect("Auto generate test config should success.")
    }

    pub fn config_path(&self) -> PathBuf {
        self.base().data_dir().join(CONFIG_FILE_PATH)
    }

    pub fn load_with_opt(opt: &StarcoinOpt) -> Result<Self> {
        let base = Arc::new(BaseConfig::default_with_opt(opt)?);
        let data_dir = base.data_dir();
        ensure!(data_dir.is_dir(), "please pass in a dir as data_dir");

        let config_file_path = data_dir.join(CONFIG_FILE_PATH);
        let config = if !config_file_path.exists() {
            info!(
                "Config file not exist, generate default config to: {:?}",
                config_file_path
            );
            // generate default config and merge with opt, the init opt item will persistence to config
            let mut config = NodeConfig::default();
            config.merge_with_opt(opt, base.clone())?;
            save_config(&config, &config_file_path)?;
            config
        } else {
            // if config file exist, load the config, and overwrite the config with option in memory, do not persistence to config again.
            info!("Load config from: {:?}", config_file_path);
            let mut config: NodeConfig = load_config(&config_file_path)?;
            config.merge_with_opt(opt, base.clone())?;
            config
        };
        info!("Final config: {}", config);
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

    pub fn node_name(&self) -> String {
        self.network.node_name()
    }
}

impl NodeConfig {
    pub fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base.clone());
        self.network.merge_with_opt(opt, base.clone())?;
        self.rpc.merge_with_opt(opt, base.clone())?;
        self.miner.merge_with_opt(opt, base.clone())?;
        self.storage.merge_with_opt(opt, base.clone())?;
        self.tx_pool.merge_with_opt(opt, base.clone())?;
        self.sync.merge_with_opt(opt, base.clone())?;
        self.vault.merge_with_opt(opt, base.clone())?;
        self.metrics.merge_with_opt(opt, base.clone())?;
        self.logger.merge_with_opt(opt, base)?;
        Ok(())
    }
}

pub(crate) fn save_config<T, P>(c: &T, output_file: P) -> Result<()>
where
    T: Serialize + DeserializeOwned,
    P: AsRef<Path>,
{
    let mut file = File::create(output_file)?;
    file.write_all(&to_toml(c)?.as_bytes())?;
    Ok(())
}

fn to_toml<T>(c: &T) -> Result<String>
where
    T: Serialize + DeserializeOwned,
{
    // fix toml table problem, see https://github.com/alexcrichton/toml-rs/issues/142
    let c = toml::value::Value::try_from(c)?;
    Ok(toml::to_string(&c)?)
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
    let mut file = open_key_file(output_file)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub(crate) fn decode_key(hex_str: &str) -> Result<(Ed25519PrivateKey, Ed25519PublicKey)> {
    let bytes_out: Vec<u8> = hex::decode(hex_str)?;
    let pri_key = Ed25519PrivateKey::try_from(bytes_out.as_slice())?;
    let pub_key = Ed25519PublicKey::from(&pri_key);
    Ok((pri_key, pub_key))
}

pub(crate) fn load_key<P: AsRef<Path>>(path: P) -> Result<(Ed25519PrivateKey, Ed25519PublicKey)> {
    let content = fs::read_to_string(path)?;
    decode_key(content.as_str())
}

pub(crate) fn gen_keypair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
    let mut gen = KeyGen::from_os_rng();
    gen.generate_keypair()
}

/// Opens a file containing a secret key in write mode.
#[cfg(unix)]
fn open_key_file<P>(path: P) -> io::Result<fs::File>
where
    P: AsRef<Path>,
{
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

/// Opens a file containing a secret key in write mode.
#[cfg(not(unix))]
fn open_key_file<P>(path: P) -> Result<fs::File, io::Error>
where
    P: AsRef<Path>,
{
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
}

const NODE_NAME_MAX_LENGTH: usize = 64;
/// Generate a valid random name for the node
fn generate_node_name() -> String {
    loop {
        let node_name = Generator::with_naming(Name::Numbered)
            .next()
            .expect("RNG is available on all supported platforms; qed");
        let count = node_name.chars().count();

        if count < NODE_NAME_MAX_LENGTH {
            return node_name;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use network_p2p_types::MultiaddrWithPeerId;

    #[test]
    fn test_generate_and_load() -> Result<()> {
        for net in BuiltinNetworkID::networks() {
            let mut opt = StarcoinOpt::default();
            let temp_path = temp_path();
            opt.net = Some(net.into());
            opt.data_dir = Some(temp_path.path().to_path_buf());
            let config = NodeConfig::load_with_opt(&opt)?;
            let config2 = NodeConfig::load_with_opt(&opt)?;
            assert_eq!(
                to_toml(&config)?,
                to_toml(&config2)?,
                "test config for network {} fail.",
                net
            );
        }
        Ok(())
    }

    #[test]
    fn test_custom_chain_genesis() -> Result<()> {
        let net = ChainNetworkID::from_str("test1:123")?;
        let temp_path = temp_path();
        let opt = StarcoinOpt {
            net: Some(net),
            data_dir: Some(temp_path.path().to_path_buf()),
            genesis_config: Some(BuiltinNetworkID::Test.to_string()),
            ..StarcoinOpt::default()
        };
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

    #[test]
    fn test_example_config_compact() -> Result<()> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let example_dir = path.join("example");
        for net in BuiltinNetworkID::networks() {
            let mut opt = StarcoinOpt {
                net: Some(net.into()),
                data_dir: Some(example_dir.clone()),
                ..StarcoinOpt::default()
            };
            opt.network.seeds = Some(vec![MultiaddrWithPeerId::from_str(
                "/ip4/198.51.100.19/tcp/30333/p2p/QmSk5HQbn6LhUwDiNMseVUjuRYhEtYj4aUZ6WfWoGURpdV",
            )?]);

            let config = NodeConfig::load_with_opt(&opt)?;
            let config2 = NodeConfig::load_with_opt(&opt)?;
            assert_eq!(
                to_toml(&config)?,
                to_toml(&config2)?,
                "test config for network {} fail.",
                net
            );
        }
        Ok(())
    }
}
