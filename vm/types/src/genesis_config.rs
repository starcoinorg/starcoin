// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::dao_config::DaoConfig;
use crate::gas_schedule::{
    AbstractMemorySize, GasAlgebra, GasCarrier, GasConstants, GasPrice, GasUnits,
};
use crate::on_chain_config::{
    ConsensusConfig, VMConfig, VMPublishingOption, Version, INITIAL_GAS_SCHEDULE,
};
use crate::time::{MockTimeService, RealTimeService, TimeService, TimeServiceType};
use crate::token::stc::STCUnit;
use crate::token::token_value::TokenValue;
use crate::transaction::{RawUserTransaction, SignedUserTransaction};
use anyhow::{bail, format_err, Result};
use libp2p::multiaddr::Multiaddr;
use move_core_types::move_resource::MoveResource;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::{ed25519::*, Genesis, HashValue, PrivateKey, ValidCryptoMaterialStringExt};
use starcoin_uint::U256;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub enum StdlibVersion {
    Latest,
    Version(VersionNumber),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct VersionNumber {
    major: u32,
    minor: u32,
}

impl StdlibVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        StdlibVersion::Version(VersionNumber { major, minor })
    }
    pub fn as_string(self) -> String {
        match self {
            StdlibVersion::Latest => "latest".to_string(),
            StdlibVersion::Version(version) => format!("{}.{}", version.major, version.minor),
        }
    }
}

impl Default for StdlibVersion {
    fn default() -> Self {
        StdlibVersion::Latest
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    IntoPrimitive,
    TryFromPrimitive,
)]
#[repr(u8)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    Dummy = 0,
    //TODO add new consensus
    Argon = 2,
    Keccak = 3,
    CryptoNight = 4,
}

impl ConsensusStrategy {
    pub fn value(self) -> u8 {
        self.into()
    }
}

impl Default for ConsensusStrategy {
    fn default() -> Self {
        ConsensusStrategy::Dummy
    }
}

impl fmt::Display for ConsensusStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsensusStrategy::Dummy => write!(f, "dummy"),
            ConsensusStrategy::Argon => write!(f, "argon"),
            ConsensusStrategy::Keccak => write!(f, "keccak"),
            ConsensusStrategy::CryptoNight => write!(f, "cryptonight"),
        }
    }
}

impl FromStr for ConsensusStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dummy" => Ok(ConsensusStrategy::Dummy),
            "argon" => Ok(ConsensusStrategy::Argon),
            "keccak" => Ok(ConsensusStrategy::Keccak),
            "cryptonight" => Ok(ConsensusStrategy::CryptoNight),
            s => Err(format_err!("Unknown ConsensusStrategy: {}", s)),
        }
    }
}

/// A static key pair to sign genesis txn
pub fn genesis_key_pair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
    let private_key = Ed25519PrivateKey::genesis();
    let public_key = private_key.public_key();
    (private_key, public_key)
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    PartialOrd,
    Ord,
    IntoPrimitive,
    TryFromPrimitive,
    Deserialize,
    Serialize,
)]
#[repr(u8)]
pub enum BuiltinNetwork {
    /// A ephemeral network just for unit test.
    Test = 255,
    /// A ephemeral network just for developer test.
    Dev = 254,
    /// Starcoin test network,
    /// The data on the chain will be cleaned up periodically。
    /// Comet Halley, officially designated 1P/Halley, is a short-period comet visible from Earth every 75–76 years.
    Halley = 3,
    /// Starcoin long-running test network,
    /// Use network upgrade strategy to upgrade chain protocol.
    /// Proxima Centauri is a small, low-mass star located 4.244 light-years (1.301 pc) away from the Sun in the southern constellation of Centaurus.
    /// Its Latin name means the "nearest [star] of Centaurus".
    Proxima = 2,
    /// Starcoin main net.
    Main = 1,
}

impl Display for BuiltinNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinNetwork::Test => write!(f, "test"),
            BuiltinNetwork::Dev => write!(f, "dev"),
            BuiltinNetwork::Halley => write!(f, "halley"),
            BuiltinNetwork::Proxima => write!(f, "proxima"),
            BuiltinNetwork::Main => write!(f, "main"),
        }
    }
}

impl FromStr for BuiltinNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test" => Ok(BuiltinNetwork::Test),
            "dev" => Ok(BuiltinNetwork::Dev),
            "halley" => Ok(BuiltinNetwork::Halley),
            "proxima" => Ok(BuiltinNetwork::Proxima),
            "main" => Ok(BuiltinNetwork::Main),
            s => Err(format_err!("Unknown network: {}", s)),
        }
    }
}

impl BuiltinNetwork {
    pub fn chain_name(self) -> String {
        self.to_string()
    }

    pub fn chain_id(self) -> ChainId {
        ChainId::new(self.into())
    }

    pub fn assert_test_or_dev(self) -> Result<()> {
        if !self.is_test_or_dev() {
            bail!("Only support test or dev network.")
        }
        Ok(())
    }

    pub fn is_test_or_dev(self) -> bool {
        match self {
            BuiltinNetwork::Test | BuiltinNetwork::Dev => true,
            _ => false,
        }
    }

    pub fn is_test(self) -> bool {
        match self {
            BuiltinNetwork::Test => true,
            _ => false,
        }
    }

    pub fn is_dev(self) -> bool {
        match self {
            BuiltinNetwork::Dev => true,
            _ => false,
        }
    }

    pub fn is_main(self) -> bool {
        match self {
            BuiltinNetwork::Main => true,
            _ => false,
        }
    }

    pub fn is_halley(self) -> bool {
        match self {
            BuiltinNetwork::Halley => true,
            _ => false,
        }
    }

    pub fn networks() -> Vec<BuiltinNetwork> {
        vec![
            BuiltinNetwork::Test,
            BuiltinNetwork::Dev,
            BuiltinNetwork::Halley,
            BuiltinNetwork::Proxima,
            BuiltinNetwork::Main,
        ]
    }

    pub fn genesis_config(self) -> &'static GenesisConfig {
        match self {
            BuiltinNetwork::Test => &TEST_CONFIG,
            BuiltinNetwork::Dev => &DEV_CONFIG,
            BuiltinNetwork::Halley => &HALLEY_CONFIG,
            BuiltinNetwork::Proxima => &PROXIMA_CONFIG,
            BuiltinNetwork::Main => &MAIN_CONFIG,
        }
    }

    pub fn boot_nodes(self) -> &'static [Multiaddr] {
        match self {
            BuiltinNetwork::Test => EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetwork::Dev => &EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetwork::Halley => &HALLEY_BOOT_NODES.as_slice(),
            BuiltinNetwork::Proxima => &PROXIMA_BOOT_NODES.as_slice(),
            BuiltinNetwork::Main => &MAIN_BOOT_NODES.as_slice(),
        }
    }
}

impl Default for BuiltinNetwork {
    fn default() -> Self {
        BuiltinNetwork::Dev
    }
}

impl From<BuiltinNetwork> for ChainNetwork {
    fn from(network: BuiltinNetwork) -> Self {
        ChainNetwork::new_builtin(network)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CustomNetwork {
    chain_name: String,
    chain_id: ChainId,
    genesis_config_name: String,
    #[serde(skip)]
    genesis_config: Option<GenesisConfig>,
}

impl Display for CustomNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.chain_name, self.chain_id, self.genesis_config_name
        )
    }
}

impl CustomNetwork {
    pub const GENESIS_CONFIG_FILE_NAME: &'static str = "genesis_config.json";

    fn new(chain_name: String, chain_id: ChainId, genesis_config_name: Option<String>) -> Self {
        Self {
            chain_name,
            chain_id,
            genesis_config_name: genesis_config_name
                .unwrap_or_else(|| Self::GENESIS_CONFIG_FILE_NAME.to_string()),
            genesis_config: None,
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn chain_name(&self) -> &str {
        self.chain_name.as_str()
    }

    pub fn genesis_config_name(&self) -> &str {
        self.genesis_config_name.as_str()
    }

    pub fn load_config(&mut self, base_dir: &Path) -> Result<()> {
        if self.genesis_config.is_some() {
            bail!("Chain config has bean loaded");
        }
        let config_name_or_path = self.genesis_config_name.as_str();
        let genesis_config = match BuiltinNetwork::from_str(config_name_or_path) {
            Ok(net) => net.genesis_config().clone(),
            Err(_) => {
                let path = Path::new(config_name_or_path);
                let config_path = if path.is_relative() {
                    base_dir.join(path)
                } else {
                    path.to_path_buf()
                };
                GenesisConfig::load(config_path)?
            }
        };
        self.genesis_config = Some(genesis_config);
        Ok(())
    }

    pub fn genesis_config(&self) -> &GenesisConfig {
        self.genesis_config
            .as_ref()
            .expect("chain config should load before get.")
    }
}

impl FromStr for CustomNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() <= 1 || parts.len() > 3 {
            bail!("Invalid Custom chain network {}, custom chain network format is: chain_name:chain_id:genesis_config_name_or_path", s);
        }
        let chain_name = parts[0].to_string();
        let chain_id = ChainId::from_str(parts[1])?;
        let genesis_config = if parts.len() == 3 {
            Some(parts[2].to_string())
        } else {
            None
        };
        Ok(Self::new(chain_name, chain_id, genesis_config))
    }
}

// ChainNetwork is a global variable and does not create many instances, so allow large enum
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ChainNetwork {
    Builtin(BuiltinNetwork),
    Custom(CustomNetwork),
}

impl Display for ChainNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Builtin(b) => b.to_string(),
            Self::Custom(c) => c.to_string(),
        };
        write!(f, "{}", name)
    }
}

impl FromStr for ChainNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinNetwork::from_str(s) {
            Ok(net) => Ok(Self::Builtin(net)),
            Err(_e) => Ok(Self::Custom(CustomNetwork::from_str(s)?)),
        }
    }
}

impl Serialize for ChainNetwork {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ChainNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Self::from_str(s.as_str()).map_err(D::Error::custom)
    }
}

impl ChainNetwork {
    pub const TEST: ChainNetwork = ChainNetwork::Builtin(BuiltinNetwork::Test);
    pub const DEV: ChainNetwork = ChainNetwork::Builtin(BuiltinNetwork::Dev);
    pub const HALLEY: ChainNetwork = ChainNetwork::Builtin(BuiltinNetwork::Halley);
    pub const PROXIMA: ChainNetwork = ChainNetwork::Builtin(BuiltinNetwork::Proxima);
    pub const MAIN: ChainNetwork = ChainNetwork::Builtin(BuiltinNetwork::Main);

    pub fn new_builtin(network: BuiltinNetwork) -> Self {
        Self::Builtin(network)
    }
    pub fn new_custom(
        chain_name: String,
        chain_id: ChainId,
        genesis_config: Option<String>,
    ) -> Result<Self> {
        for net in BuiltinNetwork::networks() {
            if net.chain_id() == chain_id {
                bail!("Chain id {} has used for builtin {}", chain_id, net);
            }
            if net.chain_name() == chain_name {
                bail!("Chain name {} has used for builtin {}", chain_name, net);
            }
        }
        Ok(Self::Custom(CustomNetwork::new(
            chain_name,
            chain_id,
            genesis_config,
        )))
    }

    pub fn load_config(&mut self, base_dir: &Path) -> Result<()> {
        match self {
            ChainNetwork::Custom(net) => net.load_config(base_dir),
            _ => Ok(()),
        }
    }

    pub fn chain_id(&self) -> ChainId {
        match self {
            Self::Builtin(b) => b.chain_id(),
            Self::Custom(c) => c.chain_id(),
        }
    }

    pub fn assert_test_or_dev(&self) -> Result<()> {
        if !self.is_test_or_dev() {
            bail!("Only support test or dev network.")
        }
        Ok(())
    }

    pub fn is_test_or_dev(&self) -> bool {
        self.is_test() || self.is_dev()
    }

    pub fn is_test(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetwork::Test) => true,
            _ => false,
        }
    }

    pub fn is_dev(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetwork::Dev) => true,
            _ => false,
        }
    }

    pub fn is_main(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetwork::Main) => true,
            _ => false,
        }
    }

    pub fn is_halley(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetwork::Halley) => true,
            _ => false,
        }
    }

    pub fn is_custom(&self) -> bool {
        match self {
            Self::Custom(_) => true,
            _ => false,
        }
    }

    /// Default data dir name of this network
    pub fn dir_name(&self) -> String {
        match self {
            Self::Builtin(net) => net.to_string(),
            Self::Custom(net) => net.chain_name().to_string(),
        }
    }

    pub fn genesis_config(&self) -> &GenesisConfig {
        match self {
            Self::Builtin(b) => b.genesis_config(),
            Self::Custom(c) => c.genesis_config(),
        }
    }

    pub fn boot_nodes(&self) -> &[Multiaddr] {
        match self {
            Self::Builtin(b) => b.boot_nodes(),
            _ => &[],
        }
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.genesis_config().consensus_config.strategy)
            .expect("consensus strategy config error.")
    }

    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.genesis_config().time_service()
    }

    pub fn as_builtin(&self) -> Option<&BuiltinNetwork> {
        match self {
            Self::Builtin(net) => Some(net),
            _ => None,
        }
    }

    pub fn as_custom(&self) -> Option<&CustomNetwork> {
        match self {
            Self::Custom(net) => Some(net),
            _ => None,
        }
    }

    pub fn builtin_networks() -> Vec<&'static ChainNetwork> {
        vec![
            &Self::TEST,
            &Self::DEV,
            &Self::HALLEY,
            &Self::PROXIMA,
            &Self::MAIN,
        ]
    }

    pub fn stdlib_version(&self) -> StdlibVersion {
        self.genesis_config().stdlib_version
    }
}

impl Default for ChainNetwork {
    fn default() -> Self {
        ChainNetwork::Builtin(BuiltinNetwork::default())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct ChainId {
    id: u8,
}

impl ChainId {
    pub fn new(id: u8) -> Self {
        Self { id }
    }

    pub fn id(self) -> u8 {
        self.id
    }

    pub fn test() -> Self {
        BuiltinNetwork::Test.chain_id()
    }

    pub fn dev() -> Self {
        BuiltinNetwork::Dev.chain_id()
    }

    pub fn net(self) -> Option<ChainNetwork> {
        if self.id() == BuiltinNetwork::Test.chain_id().id() {
            Some(ChainNetwork::TEST)
        } else if self.id() == BuiltinNetwork::Dev.chain_id().id() {
            Some(ChainNetwork::DEV)
        } else if self.id() == BuiltinNetwork::Halley.chain_id().id() {
            Some(ChainNetwork::HALLEY)
        } else if self.id() == BuiltinNetwork::Proxima.chain_id().id() {
            Some(ChainNetwork::PROXIMA)
        } else if self.id() == BuiltinNetwork::Main.chain_id().id() {
            Some(ChainNetwork::MAIN)
        } else {
            None // ToDo: handle custom network
        }
    }

    pub fn is_builtin(self) -> bool {
        self.net().is_some()
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for ChainId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u8 = s.parse()?;
        Ok(ChainId::new(id))
    }
}

impl From<u8> for ChainId {
    fn from(id: u8) -> Self {
        Self::new(id)
    }
}

impl MoveResource for ChainId {
    const MODULE_NAME: &'static str = "ChainId";
    const STRUCT_NAME: &'static str = "ChainId";
}

/// GenesisConfig is a config for initialize a chain genesis.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct GenesisConfig {
    /// Starcoin system major version for genesis.
    pub version: Version,
    /// Genesis block parent hash
    pub parent_hash: HashValue,
    /// Genesis timestamp
    pub timestamp: u64,
    /// How many block to delay before rewarding miners.
    pub reward_delay: u64,
    /// Genesis difficulty, should match consensus in different ChainNetwork.
    pub difficulty: U256,
    /// Genesis consensus nonce.
    pub nonce: u64,
    /// Pre mine STC amount to Association account.
    pub pre_mine_amount: u128,
    /// If time_mint_amount >0, Issue a LinearTimeMintKey to Association account
    /// LinearTimeMintKey's total.
    pub time_mint_amount: u128,
    /// LinearTimeMintKey's period in seconds.
    pub time_mint_period: u64,
    /// VM config for publishing_option and gas_schedule
    pub vm_config: VMConfig,
    /// Script allow list and Module publish option
    pub publishing_option: VMPublishingOption,
    /// VM gas constants config.
    pub gas_constants: GasConstants,
    pub consensus_config: ConsensusConfig,
    /// association account's key pair
    pub association_key_pair: (Option<Arc<Ed25519PrivateKey>>, Ed25519PublicKey),
    /// genesis account's key pair
    pub genesis_key_pair: Option<(Arc<Ed25519PrivateKey>, Ed25519PublicKey)>,

    pub stdlib_version: StdlibVersion,
    pub dao_config: DaoConfig,
    /// TimeService
    pub time_service_type: TimeServiceType,
    /// transaction timeout
    pub transaction_timeout: u64,
}

impl GenesisConfig {
    pub fn sign_with_association(&self, txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        if let (Some(private_key), public_key) = &self.association_key_pair {
            Ok(txn.sign(private_key, public_key.clone())?.into_inner())
        } else {
            bail!("association private_key not config at current network",)
        }
    }

    pub fn sign_with_genesis(self, txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        if let Some((private_key, public_key)) = &self.genesis_key_pair {
            Ok(txn.sign(private_key, public_key.clone())?.into_inner())
        } else {
            bail!("genesis private_key not config at current network.",)
        }
    }

    pub fn load<P>(path: P) -> Result<GenesisConfig>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let mut file = File::create(&path)?;
        let buf = serde_json::to_vec(self)?;
        file.write_all(buf.as_slice())?;
        Ok(())
    }

    pub fn time_service(&self) -> Arc<dyn TimeService> {
        match self.time_service_type {
            TimeServiceType::RealTimeService => (*REAL_TIME_SERVICE).clone(),
            TimeServiceType::MockTimeService => (*MOKE_TIME_SERVICE).clone(),
            // _ => (*MOKE_TIME_SERVICE).clone(),
        }
    }
}

static UNCLE_RATE_TARGET: u64 = 80;
static DEFAULT_BASE_BLOCK_TIME_TARGET: u64 = 10;
static DEFAULT_BASE_BLOCK_DIFF_WINDOW: u64 = 24;
static BASE_REWARD_PER_UNCLE_PERCENT: u64 = 10;
static MIN_BLOCK_TIME_TARGET: u64 = 1;
static MAX_BLOCK_TIME_TARGET: u64 = 60;
static BASE_MAX_UNCLES_PER_BLOCK: u64 = 2;

//for pre sell
static DEFAULT_PRE_MINT_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(500_000_000));
//for dev and ecosystem build.
static DEFAULT_TIME_LOCKED_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(1_500_000_000));
//three years.
static DEFAULT_TIME_LOCKED_PERIOD: u64 = 3600 * 24 * 365 * 3;

static DEFAULT_BASE_REWARD_PER_BLOCK: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(64));
//time service
pub static REAL_TIME_SERVICE: Lazy<Arc<dyn TimeService>> =
    Lazy::new(|| Arc::new(RealTimeService::new()));

pub static MOKE_TIME_SERVICE: Lazy<Arc<dyn TimeService>> =
    Lazy::new(|| Arc::new(MockTimeService::new_with_value(1)));

pub static BASE_BLOCK_GAS_LIMIT: u64 = 1_000_000;

pub static MAX_TRANSACTION_SIZE_IN_BYTES: u64 = 4096 * 10;

/// For V1 all accounts will be ~800 bytes
static DEFAULT_ACCOUNT_SIZE: Lazy<AbstractMemorySize<GasCarrier>> =
    Lazy::new(|| AbstractMemorySize::new(800));

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
static LARGE_TRANSACTION_CUTOFF: Lazy<AbstractMemorySize<GasCarrier>> =
    Lazy::new(|| AbstractMemorySize::new(600));

static DEFAULT_GAS_CONSTANTS: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: GasUnits::new(4),
        global_memory_per_byte_write_cost: GasUnits::new(9),
        min_transaction_gas_units: GasUnits::new(600),
        large_transaction_cutoff: *LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: GasUnits::new(8),
        maximum_number_of_gas_units: GasUnits::new(4_000_000),
        min_price_per_gas_unit: GasPrice::new(0),
        max_price_per_gas_unit: GasPrice::new(10_000),
        max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES, // to pass stdlib_upgrade
        gas_unit_scaling_factor: 1000,
        default_account_size: *DEFAULT_ACCOUNT_SIZE,
    }
});

pub static EMPTY_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(Vec::new);

pub const ONE_DAY: u64 = 86400;

pub static TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    GenesisConfig {
        version: Version { major: 1 },
        parent_hash: HashValue::sha3_256_of(b"starcoin_test"),
        //Test timestamp set to 0 for mock time.
        timestamp: 0,
        reward_delay: 1,
        difficulty: 1.into(),
        nonce: 0,
        pre_mine_amount: DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: 3600,
        vm_config: VMConfig {
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        publishing_option: VMPublishingOption::Open,
        gas_constants: DEFAULT_GAS_CONSTANTS.clone(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: UNCLE_RATE_TARGET,
            base_block_time_target: DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: DEFAULT_BASE_BLOCK_DIFF_WINDOW * 2,
            base_block_difficulty_window: DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: MIN_BLOCK_TIME_TARGET,
            max_block_time_target: MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::Dummy.value(),
        },
        association_key_pair: (
            Some(Arc::new(association_private_key)),
            association_public_key,
        ),
        genesis_key_pair: Some((Arc::new(genesis_private_key), genesis_public_key)),
        time_service_type: TimeServiceType::MockTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60,       // 1min
            voting_period: 60 * 60, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60, // 1h
        },
        transaction_timeout: ONE_DAY,
    }
});

pub static DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    GenesisConfig {
        version: Version { major: 1 },
        //use latest git commit version's hash
        parent_hash: HashValue::sha3_256_of(
            hex::decode("8bf9cdf5f3624db507613f7fe0cd786c8c9f8037")
                .expect("invalid hex")
                .as_slice(),
        ),
        timestamp: 1596791843,
        reward_delay: 1,
        difficulty: 1.into(),
        nonce: 0,
        pre_mine_amount: DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: 3600 * 24,
        vm_config: VMConfig {
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        publishing_option: VMPublishingOption::Open,
        gas_constants: DEFAULT_GAS_CONSTANTS.clone(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: UNCLE_RATE_TARGET,
            base_block_time_target: DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: DEFAULT_BASE_BLOCK_DIFF_WINDOW * 2,
            base_block_difficulty_window: DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: MIN_BLOCK_TIME_TARGET,
            max_block_time_target: MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::Dummy.value(),
        },
        association_key_pair: (
            Some(Arc::new(association_private_key)),
            association_public_key,
        ),
        genesis_key_pair: Some((Arc::new(genesis_private_key), genesis_public_key)),
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60,       // 1min
            voting_period: 60 * 60, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60, // 1h
        },
        transaction_timeout: ONE_DAY,
    }
});

pub static HALLEY_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(|| {
    vec!["/dns4/halley1.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"),
         "/dns4/halley2.seed.starcoin.org/tcp/9840/p2p/12D3KooWAua4KokJMiCodGPEF2n4yN42B2Q26KgwrQTntnrCDRHd".parse().expect("parse multi addr should be ok"),
         "/dns4/halley3.seed.starcoin.org/tcp/9840/p2p/12D3KooW9vHQJk9o69tZPMM2viQ3eWpgp6veDBRz8tTvDFDBejwk".parse().expect("parse multi addr should be ok"), ]
});

pub static HALLEY_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    GenesisConfig {
        version: Version { major: 1 },
        //use latest git commit hash
        parent_hash: HashValue::sha3_256_of(
            hex::decode("8bf9cdf5f3624db507613f7fe0cd786c8c9f8037")
                .expect("invalid hex")
                .as_slice(),
        ),
        timestamp: 1596791843,
        reward_delay: 3,
        difficulty: 100000.into(),
        nonce: 0,
        pre_mine_amount: DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: 3600 * 24 * 31,
        vm_config: VMConfig {
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        publishing_option: VMPublishingOption::Open,
        gas_constants: DEFAULT_GAS_CONSTANTS.clone(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: UNCLE_RATE_TARGET,
            base_block_time_target: DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
            base_block_difficulty_window: DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: MIN_BLOCK_TIME_TARGET,
            max_block_time_target: MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::Keccak.value(),
        },
        association_key_pair: (
            None,
            Ed25519PublicKey::from_encoded_string(
                "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
            )
            .expect("decode public key must success."),
        ),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60,       // 1min
            voting_period: 60 * 60, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60, // 1h
        },
        transaction_timeout: ONE_DAY,
    }
});

pub static PROXIMA_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(|| {
    vec!["/dns4/proxima1.seed.starcoin.org/tcp/9840/p2p/12D3KooW9vHQJk9o69tZPMM2viQ3eWpgp6veDBRz8tTvDFDBejwk".parse().expect("parse multi addr should be ok"),
         "/dns4/proxima2.seed.starcoin.org/tcp/9840/p2p/12D3KooWAua4KokJMiCodGPEF2n4yN42B2Q26KgwrQTntnrCDRHd".parse().expect("parse multi addr should be ok"),
         "/dns4/proxima3.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"), ]
});

pub static PROXIMA_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    version: Version { major: 1 },
    parent_hash: HashValue::sha3_256_of(
        hex::decode("6b1eddc3847bb8476f8937abd017e5833e878b60")
            .expect("invalid hex")
            .as_slice(),
    ),
    timestamp: 1596791843,
    reward_delay: 7,
    difficulty: 10.into(),
    nonce: 0,
    pre_mine_amount: DEFAULT_PRE_MINT_AMOUNT.scaling(),
    time_mint_amount: DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
    time_mint_period: DEFAULT_TIME_LOCKED_PERIOD,
    vm_config: VMConfig {
        gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
    },
    publishing_option: VMPublishingOption::Open,
    gas_constants: DEFAULT_GAS_CONSTANTS.clone(),
    consensus_config: ConsensusConfig {
        uncle_rate_target: UNCLE_RATE_TARGET,
        base_block_time_target: DEFAULT_BASE_BLOCK_TIME_TARGET,
        base_reward_per_block: DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
        epoch_block_count: DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
        base_block_difficulty_window: DEFAULT_BASE_BLOCK_DIFF_WINDOW,
        base_reward_per_uncle_percent: BASE_REWARD_PER_UNCLE_PERCENT,
        min_block_time_target: MIN_BLOCK_TIME_TARGET,
        max_block_time_target: MAX_BLOCK_TIME_TARGET,
        base_max_uncles_per_block: BASE_MAX_UNCLES_PER_BLOCK,
        base_block_gas_limit: BASE_BLOCK_GAS_LIMIT,
        strategy: ConsensusStrategy::Keccak.value(),
    },
    association_key_pair: (
        None,
        Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
        .expect("decode public key must success."),
    ),
    genesis_key_pair: None,
    time_service_type: TimeServiceType::RealTimeService,
    stdlib_version: StdlibVersion::Latest,
    dao_config: DaoConfig {
        voting_delay: 60 * 60,           // 1h
        voting_period: 60 * 60 * 24 * 2, // 2d
        voting_quorum_rate: 4,
        min_action_delay: 60 * 60 * 24, // 1d
    },
    transaction_timeout: ONE_DAY,
});

pub static MAIN_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(Vec::new);

pub static MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    version: Version { major: 1 },
    //TODO set parent_hash and timestamp
    parent_hash: HashValue::zero(),
    timestamp: 0,
    reward_delay: 7,
    difficulty: 10.into(),
    nonce: 0,
    pre_mine_amount: DEFAULT_PRE_MINT_AMOUNT.scaling(),
    time_mint_amount: DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
    time_mint_period: DEFAULT_TIME_LOCKED_PERIOD,
    vm_config: VMConfig {
        gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
    },
    publishing_option: VMPublishingOption::Open,
    gas_constants: DEFAULT_GAS_CONSTANTS.clone(),
    consensus_config: ConsensusConfig {
        uncle_rate_target: UNCLE_RATE_TARGET,
        base_block_time_target: DEFAULT_BASE_BLOCK_TIME_TARGET,
        base_reward_per_block: DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
        epoch_block_count: DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
        base_block_difficulty_window: DEFAULT_BASE_BLOCK_DIFF_WINDOW,
        base_reward_per_uncle_percent: BASE_REWARD_PER_UNCLE_PERCENT,
        min_block_time_target: MIN_BLOCK_TIME_TARGET,
        max_block_time_target: MAX_BLOCK_TIME_TARGET,
        base_max_uncles_per_block: BASE_MAX_UNCLES_PER_BLOCK,
        base_block_gas_limit: BASE_BLOCK_GAS_LIMIT,
        strategy: ConsensusStrategy::Keccak.value(),
    },
    association_key_pair: (
        None,
        Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
        .expect("decode public key must success."),
    ),
    genesis_key_pair: None,
    time_service_type: TimeServiceType::RealTimeService,
    stdlib_version: StdlibVersion::Latest,
    dao_config: DaoConfig {
        voting_delay: 60 * 60,           // 1h
        voting_period: 60 * 60 * 24 * 2, // 2d
        voting_quorum_rate: 4,
        min_action_delay: 60 * 60 * 24, // 1d
    },
    transaction_timeout: ONE_DAY,
});
