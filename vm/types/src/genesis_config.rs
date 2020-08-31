// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::{VMConfig, VMPublishingOption, Version, INITIAL_GAS_SCHEDULE};
use crate::transaction::{RawUserTransaction, SignedUserTransaction};
use anyhow::{bail, format_err, Result};
use ethereum_types::U256;
use libp2p::multiaddr::Multiaddr;
use move_core_types::move_resource::MoveResource;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::{ed25519::*, Genesis, HashValue, PrivateKey, ValidCryptoMaterialStringExt};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, IntoPrimitive,
)]
#[repr(u8)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    Dummy = 0,
    Dev = 1,
    Argon = 2,
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
            ConsensusStrategy::Dev => write!(f, "dev"),
            ConsensusStrategy::Argon => write!(f, "argon"),
        }
    }
}

impl FromStr for ConsensusStrategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dummy" => Ok(ConsensusStrategy::Dummy),
            "dev" => Ok(ConsensusStrategy::Dev),
            "argon" => Ok(ConsensusStrategy::Argon),
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
    genesis_config: String,
    #[serde(skip)]
    genesis_config_loaded: Option<GenesisConfig>,
}

impl Display for CustomNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.chain_name, self.chain_id, self.genesis_config
        )
    }
}

impl CustomNetwork {
    pub const GENESIS_CONFIG_FILE_NAME: &'static str = "genesis_config.json";

    pub fn new(chain_name: String, chain_id: ChainId, genesis_config: Option<String>) -> Self {
        Self {
            chain_name,
            chain_id,
            genesis_config: genesis_config
                .unwrap_or_else(|| Self::GENESIS_CONFIG_FILE_NAME.to_string()),
            genesis_config_loaded: None,
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn chain_name(&self) -> &str {
        self.chain_name.as_str()
    }

    pub fn load_config(&mut self, base_dir: &Path) -> Result<()> {
        if self.genesis_config_loaded.is_some() {
            bail!("Chain config has bean loaded");
        }
        let config_name_or_path = self.genesis_config.as_str();
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
        self.genesis_config_loaded = Some(genesis_config);
        Ok(())
    }

    pub fn genesis_config(&self) -> &GenesisConfig {
        self.genesis_config_loaded
            .as_ref()
            .expect("chain config should load before get.")
    }
}

impl FromStr for CustomNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() <= 1 || parts.len() > 3 {
            bail!("Invalid Custom chain network {}, custom chain network format is: chain_name:chain_id:genesis_config_name or path", s);
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

#[derive(Clone, Debug, PartialEq)]
pub enum ChainNetwork {
    Builtin(BuiltinNetwork),
    #[allow(clippy::large_enum_variant)]
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
        Self::from_str(s.as_str()).map_err(|e| D::Error::custom(e))
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
        self.genesis_config().consensus_strategy
    }

    pub fn as_builtin(&self) -> Option<BuiltinNetwork> {
        match self {
            Self::Builtin(net) => Some(*net),
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
    /// pre mine amount to Association account.
    pub pre_mine_amount: u128,
    /// VM config for publishing_option and gas_schedule
    pub vm_config: VMConfig,
    /// uncle rate target
    pub uncle_rate_target: u64,
    /// how many block as a epoch
    pub epoch_block_count: u64,
    /// init block time target for first epoch
    pub init_block_time_target: u64,
    /// block window
    pub block_difficulty_window: u64,
    /// init block reward for first epoch
    pub init_reward_per_block: u128,
    /// reward per uncle percent
    pub reward_per_uncle_percent: u64,
    /// min block time target
    pub min_block_time_target: u64,
    /// max block time target
    pub max_block_time_target: u64,
    /// max uncle block count per block
    pub max_uncles_per_block: u64,
    /// association account's key pair
    pub association_key_pair: (Option<Ed25519PrivateKey>, Ed25519PublicKey),
    /// genesis account's key pair
    pub genesis_key_pair: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,
    /// consensus strategy for chain
    pub consensus_strategy: ConsensusStrategy,

    pub global_memory_per_byte_cost: u64,
    pub global_memory_per_byte_write_cost: u64,
    pub min_transaction_gas_units: u64,
    pub large_transaction_cutoff: u64,
    pub instrinsic_gas_per_byte: u64,
    pub maximum_number_of_gas_units: u64,
    pub min_price_per_gas_unit: u64,
    pub max_price_per_gas_unit: u64,
    pub max_transaction_size_in_bytes: u64,
    pub gas_unit_scaling_factor: u64,
    pub default_account_size: u64,
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

    pub fn block_gas_limit(&self) -> u64 {
        self.vm_config.block_gas_limit
    }

    pub fn load<P>(path: P) -> Result<GenesisConfig>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }
}

pub static UNCLE_RATE_TARGET: u64 = 80;
pub static INIT_BLOCK_TIME_TARGET: u64 = 5;
pub static BLOCK_DIFF_WINDOW: u64 = 24;
pub static REWARD_PER_UNCLE_PERCENT: u64 = 10;
pub static MIN_BLOCK_TIME_TARGET: u64 = 1;
pub static MAX_BLOCK_TIME_TARGET: u64 = 60;
pub static MAX_UNCLES_PER_BLOCK: u64 = 2;
pub static INIT_REWARD_PER_BLOCK: u128 = 50 * 1_000_000;

static PRE_MINT_AMOUNT: u128 =
    INIT_REWARD_PER_BLOCK * ((3600 * 24 * 30) / INIT_BLOCK_TIME_TARGET as u128);

pub static BLOCK_GAS_LIMIT: u64 = 1_000_000;

pub static GLOBAL_MEMORY_PER_BYTE_COST: u64 = 2;
pub static GLOBAL_MEMORY_PER_BYTE_WRITE_COST: u64 = 5;
pub static MIN_TRANSACTION_GAS_UNITS: u64 = 600;
pub static LARGE_TRANSACTION_CUTOFF: u64 = 600;
pub static INSTRINSIC_GAS_PER_BYTE: u64 = 8;
pub static MAXIMUM_NUMBER_OF_GAS_UNITS: u64 = 4_000_000;
pub static MIN_PRICE_PER_GAS_UNIT: u64 = 1;
pub static MAX_PRICE_PER_GAS_UNIT: u64 = 10_000;
pub static MAX_TRANSACTION_SIZE_IN_BYTES: u64 = 4096 * 10;
pub static GAS_UNIT_SCALING_FACTOR: u64 = 1000;
pub static DEFAULT_ACCOUNT_SIZE: u64 = 800;

pub static EMPTY_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(|| vec![]);

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
        pre_mine_amount: PRE_MINT_AMOUNT,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
            block_gas_limit: BLOCK_GAS_LIMIT,
        },
        uncle_rate_target: UNCLE_RATE_TARGET,
        epoch_block_count: BLOCK_DIFF_WINDOW * 2,
        init_block_time_target: INIT_BLOCK_TIME_TARGET,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        init_reward_per_block: INIT_REWARD_PER_BLOCK,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_block_time_target: MIN_BLOCK_TIME_TARGET,
        max_block_time_target: MAX_BLOCK_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (Some(association_private_key), association_public_key),
        genesis_key_pair: Some((genesis_private_key, genesis_public_key)),
        consensus_strategy: ConsensusStrategy::Dummy,
        global_memory_per_byte_cost: GLOBAL_MEMORY_PER_BYTE_COST,
        global_memory_per_byte_write_cost: GLOBAL_MEMORY_PER_BYTE_WRITE_COST,
        min_transaction_gas_units: MIN_TRANSACTION_GAS_UNITS,
        large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
        instrinsic_gas_per_byte: INSTRINSIC_GAS_PER_BYTE,
        maximum_number_of_gas_units: MAXIMUM_NUMBER_OF_GAS_UNITS,
        min_price_per_gas_unit: 0, // set to 0
        max_price_per_gas_unit: MAX_PRICE_PER_GAS_UNIT,
        max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES, // to pass stdlib_upgrade
        gas_unit_scaling_factor: GAS_UNIT_SCALING_FACTOR,
        default_account_size: DEFAULT_ACCOUNT_SIZE,
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
        pre_mine_amount: PRE_MINT_AMOUNT,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            // ToDo: remove gas_schedule
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
            block_gas_limit: BLOCK_GAS_LIMIT,
        },
        uncle_rate_target: UNCLE_RATE_TARGET,
        epoch_block_count: BLOCK_DIFF_WINDOW * 2,
        init_block_time_target: INIT_BLOCK_TIME_TARGET,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        init_reward_per_block: INIT_REWARD_PER_BLOCK,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_block_time_target: MIN_BLOCK_TIME_TARGET,
        max_block_time_target: MAX_BLOCK_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (Some(association_private_key), association_public_key),
        genesis_key_pair: Some((genesis_private_key, genesis_public_key)),
        consensus_strategy: ConsensusStrategy::Dev,
        global_memory_per_byte_cost: GLOBAL_MEMORY_PER_BYTE_COST,
        global_memory_per_byte_write_cost: GLOBAL_MEMORY_PER_BYTE_WRITE_COST,
        min_transaction_gas_units: MIN_TRANSACTION_GAS_UNITS,
        large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
        instrinsic_gas_per_byte: INSTRINSIC_GAS_PER_BYTE,
        maximum_number_of_gas_units: MAXIMUM_NUMBER_OF_GAS_UNITS,
        min_price_per_gas_unit: MIN_PRICE_PER_GAS_UNIT,
        max_price_per_gas_unit: MAX_PRICE_PER_GAS_UNIT,
        max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES,
        gas_unit_scaling_factor: GAS_UNIT_SCALING_FACTOR,
        default_account_size: DEFAULT_ACCOUNT_SIZE,
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
        difficulty: 10.into(),
        nonce: 0,
        pre_mine_amount: PRE_MINT_AMOUNT,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
            block_gas_limit: BLOCK_GAS_LIMIT,
        },
        uncle_rate_target: UNCLE_RATE_TARGET,
        epoch_block_count: BLOCK_DIFF_WINDOW * 10,
        init_block_time_target: INIT_BLOCK_TIME_TARGET,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        init_reward_per_block: INIT_REWARD_PER_BLOCK,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_block_time_target: MIN_BLOCK_TIME_TARGET,
        max_block_time_target: MAX_BLOCK_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (
            None,
            Ed25519PublicKey::from_encoded_string(
                "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
            )
            .expect("decode public key must success."),
        ),
        genesis_key_pair: None,
        consensus_strategy: ConsensusStrategy::Argon,
        global_memory_per_byte_cost: GLOBAL_MEMORY_PER_BYTE_COST,
        global_memory_per_byte_write_cost: GLOBAL_MEMORY_PER_BYTE_WRITE_COST,
        min_transaction_gas_units: MIN_TRANSACTION_GAS_UNITS,
        large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
        instrinsic_gas_per_byte: INSTRINSIC_GAS_PER_BYTE,
        maximum_number_of_gas_units: MAXIMUM_NUMBER_OF_GAS_UNITS,
        min_price_per_gas_unit: MIN_PRICE_PER_GAS_UNIT,
        max_price_per_gas_unit: MAX_PRICE_PER_GAS_UNIT,
        max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES,
        gas_unit_scaling_factor: GAS_UNIT_SCALING_FACTOR,
        default_account_size: DEFAULT_ACCOUNT_SIZE,
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
    pre_mine_amount: PRE_MINT_AMOUNT,
    vm_config: VMConfig {
        publishing_option: VMPublishingOption::Open,
        gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        block_gas_limit: BLOCK_GAS_LIMIT,
    },
    uncle_rate_target: UNCLE_RATE_TARGET,
    epoch_block_count: BLOCK_DIFF_WINDOW * 10,
    init_block_time_target: INIT_BLOCK_TIME_TARGET,
    block_difficulty_window: BLOCK_DIFF_WINDOW,
    init_reward_per_block: INIT_REWARD_PER_BLOCK,
    reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
    min_block_time_target: MIN_BLOCK_TIME_TARGET,
    max_block_time_target: MAX_BLOCK_TIME_TARGET,
    max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
    association_key_pair: (
        None,
        Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
        .expect("decode public key must success."),
    ),
    genesis_key_pair: None,
    consensus_strategy: ConsensusStrategy::Argon,
    global_memory_per_byte_cost: GLOBAL_MEMORY_PER_BYTE_COST,
    global_memory_per_byte_write_cost: GLOBAL_MEMORY_PER_BYTE_WRITE_COST,
    min_transaction_gas_units: MIN_TRANSACTION_GAS_UNITS,
    large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
    instrinsic_gas_per_byte: INSTRINSIC_GAS_PER_BYTE,
    maximum_number_of_gas_units: MAXIMUM_NUMBER_OF_GAS_UNITS,
    min_price_per_gas_unit: MIN_PRICE_PER_GAS_UNIT,
    max_price_per_gas_unit: MAX_PRICE_PER_GAS_UNIT,
    max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES,
    gas_unit_scaling_factor: GAS_UNIT_SCALING_FACTOR,
    default_account_size: DEFAULT_ACCOUNT_SIZE,
});

pub static MAIN_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(|| vec![]);

pub static MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| GenesisConfig {
    version: Version { major: 1 },
    //TODO set parent_hash and timestamp
    parent_hash: HashValue::zero(),
    timestamp: 0,
    reward_delay: 7,
    difficulty: 10.into(),
    nonce: 0,
    pre_mine_amount: 0,
    vm_config: VMConfig {
        publishing_option: VMPublishingOption::Open,
        gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        block_gas_limit: BLOCK_GAS_LIMIT,
    },
    uncle_rate_target: UNCLE_RATE_TARGET,
    epoch_block_count: BLOCK_DIFF_WINDOW * 1000,
    init_block_time_target: INIT_BLOCK_TIME_TARGET,
    block_difficulty_window: BLOCK_DIFF_WINDOW,
    init_reward_per_block: INIT_REWARD_PER_BLOCK,
    reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
    min_block_time_target: MIN_BLOCK_TIME_TARGET,
    max_block_time_target: MAX_BLOCK_TIME_TARGET,
    max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
    association_key_pair: (
        None,
        Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
        .expect("decode public key must success."),
    ),
    genesis_key_pair: None,
    consensus_strategy: ConsensusStrategy::Argon,
    global_memory_per_byte_cost: GLOBAL_MEMORY_PER_BYTE_COST,
    global_memory_per_byte_write_cost: GLOBAL_MEMORY_PER_BYTE_WRITE_COST,
    min_transaction_gas_units: MIN_TRANSACTION_GAS_UNITS,
    large_transaction_cutoff: LARGE_TRANSACTION_CUTOFF,
    instrinsic_gas_per_byte: INSTRINSIC_GAS_PER_BYTE,
    maximum_number_of_gas_units: MAXIMUM_NUMBER_OF_GAS_UNITS,
    min_price_per_gas_unit: MIN_PRICE_PER_GAS_UNIT,
    max_price_per_gas_unit: MAX_PRICE_PER_GAS_UNIT,
    max_transaction_size_in_bytes: MAX_TRANSACTION_SIZE_IN_BYTES,
    gas_unit_scaling_factor: GAS_UNIT_SCALING_FACTOR,
    default_account_size: DEFAULT_ACCOUNT_SIZE,
});
