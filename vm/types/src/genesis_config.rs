// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::{
    AbstractMemorySize, GasAlgebra, GasCarrier, GasConstants, GasPrice, GasUnits,
};
use crate::on_chain_config::DaoConfig;
use crate::on_chain_config::{
    ConsensusConfig, VMConfig, VMPublishingOption, Version, INITIAL_GAS_SCHEDULE,
};
use crate::time::{TimeService, TimeServiceType};
use crate::token::stc::STCUnit;
use crate::token::token_value::TokenValue;
use crate::transaction::{RawUserTransaction, SignedUserTransaction};
use anyhow::{bail, ensure, format_err, Result};
use libp2p::multiaddr::Multiaddr;
use move_core_types::move_resource::MoveResource;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::de::Error;
use serde::export::fmt::Debug;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519KeyShard;
use starcoin_crypto::{
    ed25519::*,
    multi_ed25519::{genesis_multi_key_pair, MultiEd25519PublicKey},
    HashValue, ValidCryptoMaterialStringExt,
};
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

type VersionNumber = u64;

impl StdlibVersion {
    pub fn new(version: u64) -> Self {
        StdlibVersion::Version(version)
    }

    pub fn as_string(&self) -> String {
        match self {
            StdlibVersion::Latest => "latest".to_string(),
            StdlibVersion::Version(version) => format!("{}", version),
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            StdlibVersion::Latest => 0,
            StdlibVersion::Version(version) => *version,
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
    Argon = 1,
    Keccak = 2,
    CryptoNight = 3,
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
pub enum BuiltinNetworkID {
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

impl Display for BuiltinNetworkID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinNetworkID::Test => write!(f, "test"),
            BuiltinNetworkID::Dev => write!(f, "dev"),
            BuiltinNetworkID::Halley => write!(f, "halley"),
            BuiltinNetworkID::Proxima => write!(f, "proxima"),
            BuiltinNetworkID::Main => write!(f, "main"),
        }
    }
}

impl FromStr for BuiltinNetworkID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test" => Ok(BuiltinNetworkID::Test),
            "dev" => Ok(BuiltinNetworkID::Dev),
            "halley" => Ok(BuiltinNetworkID::Halley),
            "proxima" => Ok(BuiltinNetworkID::Proxima),
            "main" => Ok(BuiltinNetworkID::Main),
            s => Err(format_err!("Unknown network: {}", s)),
        }
    }
}

impl BuiltinNetworkID {
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
            BuiltinNetworkID::Test | BuiltinNetworkID::Dev => true,
            _ => false,
        }
    }

    pub fn is_test(self) -> bool {
        match self {
            BuiltinNetworkID::Test => true,
            _ => false,
        }
    }

    pub fn is_dev(self) -> bool {
        match self {
            BuiltinNetworkID::Dev => true,
            _ => false,
        }
    }

    pub fn is_main(self) -> bool {
        match self {
            BuiltinNetworkID::Main => true,
            _ => false,
        }
    }

    pub fn is_halley(self) -> bool {
        match self {
            BuiltinNetworkID::Halley => true,
            _ => false,
        }
    }

    pub fn networks() -> Vec<BuiltinNetworkID> {
        vec![
            BuiltinNetworkID::Test,
            BuiltinNetworkID::Dev,
            BuiltinNetworkID::Halley,
            BuiltinNetworkID::Proxima,
            BuiltinNetworkID::Main,
        ]
    }

    pub fn genesis_config(self) -> &'static GenesisConfig {
        match self {
            BuiltinNetworkID::Test => &TEST_CONFIG,
            BuiltinNetworkID::Dev => &DEV_CONFIG,
            BuiltinNetworkID::Halley => &HALLEY_CONFIG,
            BuiltinNetworkID::Proxima => &PROXIMA_CONFIG,
            BuiltinNetworkID::Main => &MAIN_CONFIG,
        }
    }

    pub fn boot_nodes(self) -> &'static [Multiaddr] {
        match self {
            BuiltinNetworkID::Test => EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Dev => &EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Halley => &HALLEY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Proxima => &PROXIMA_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Main => &MAIN_BOOT_NODES.as_slice(),
        }
    }
}

impl Default for BuiltinNetworkID {
    fn default() -> Self {
        BuiltinNetworkID::Dev
    }
}

impl From<BuiltinNetworkID> for ChainNetwork {
    fn from(network: BuiltinNetworkID) -> Self {
        ChainNetwork::new(
            ChainNetworkID::Builtin(network),
            network.genesis_config().clone(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CustomNetworkID {
    chain_name: String,
    chain_id: ChainId,
}

impl Display for CustomNetworkID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain_name, self.chain_id)
    }
}

impl CustomNetworkID {
    fn new(chain_name: String, chain_id: ChainId) -> Self {
        Self {
            chain_name,
            chain_id,
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn chain_name(&self) -> &str {
        self.chain_name.as_str()
    }
}

impl FromStr for CustomNetworkID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            bail!("Invalid Custom chain network {}, custom chain network format is: chain_name:chain_id", s);
        }
        let chain_name = parts[0].to_string();
        let chain_id = ChainId::from_str(parts[1])?;
        Ok(Self::new(chain_name, chain_id))
    }
}

// ChainNetwork is a global variable and does not create many instances, so allow large enum
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ChainNetworkID {
    Builtin(BuiltinNetworkID),
    Custom(CustomNetworkID),
}

impl Display for ChainNetworkID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Builtin(b) => b.to_string(),
            Self::Custom(c) => c.to_string(),
        };
        write!(f, "{}", name)
    }
}

impl From<BuiltinNetworkID> for ChainNetworkID {
    fn from(network: BuiltinNetworkID) -> Self {
        ChainNetworkID::Builtin(network)
    }
}

impl FromStr for ChainNetworkID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match BuiltinNetworkID::from_str(s) {
            Ok(net) => Ok(Self::Builtin(net)),
            Err(_e) => Ok(Self::Custom(CustomNetworkID::from_str(s)?)),
        }
    }
}

impl Serialize for ChainNetworkID {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for ChainNetworkID {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Self::from_str(s.as_str()).map_err(D::Error::custom)
    }
}

impl ChainNetworkID {
    pub const TEST: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Test);
    pub const DEV: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Dev);
    pub const HALLEY: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Halley);
    pub const PROXIMA: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Proxima);
    pub const MAIN: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Main);

    pub fn new_builtin(network: BuiltinNetworkID) -> Self {
        Self::Builtin(network)
    }
    pub fn new_custom(chain_name: String, chain_id: ChainId) -> Result<Self> {
        for net in BuiltinNetworkID::networks() {
            if net.chain_id() == chain_id {
                bail!("Chain id {} has used for builtin {}", chain_id, net);
            }
            if net.chain_name() == chain_name {
                bail!("Chain name {} has used for builtin {}", chain_name, net);
            }
        }
        Ok(Self::Custom(CustomNetworkID::new(chain_name, chain_id)))
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
            Self::Builtin(BuiltinNetworkID::Test) => true,
            _ => false,
        }
    }

    pub fn is_dev(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetworkID::Dev) => true,
            _ => false,
        }
    }

    pub fn is_main(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetworkID::Main) => true,
            _ => false,
        }
    }

    pub fn is_halley(&self) -> bool {
        match self {
            Self::Builtin(BuiltinNetworkID::Halley) => true,
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

    pub fn boot_nodes(&self) -> &[Multiaddr] {
        match self {
            Self::Builtin(b) => b.boot_nodes(),
            _ => &[],
        }
    }

    pub fn as_builtin(&self) -> Option<&BuiltinNetworkID> {
        match self {
            Self::Builtin(net) => Some(net),
            _ => None,
        }
    }

    pub fn as_custom(&self) -> Option<&CustomNetworkID> {
        match self {
            Self::Custom(net) => Some(net),
            _ => None,
        }
    }
}

impl Default for ChainNetworkID {
    fn default() -> Self {
        ChainNetworkID::Builtin(BuiltinNetworkID::default())
    }
}

#[derive(Clone, Debug)]
pub struct ChainNetwork {
    id: ChainNetworkID,
    genesis_config: GenesisConfig,
    time_service: Arc<dyn TimeService>,
}

impl Display for ChainNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl PartialEq for ChainNetwork {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl ChainNetwork {
    pub fn new(id: ChainNetworkID, genesis_config: GenesisConfig) -> Self {
        let time_service = genesis_config.time_service_type.new_time_service();
        Self {
            id,
            genesis_config,
            time_service,
        }
    }
    pub fn new_with_time_service(
        id: ChainNetworkID,
        genesis_config: GenesisConfig,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            id,
            genesis_config,
            time_service,
        }
    }

    pub fn new_builtin(builtin_id: BuiltinNetworkID) -> Self {
        Self::new(builtin_id.into(), builtin_id.genesis_config().clone())
    }

    pub fn new_custom(
        chain_name: String,
        chain_id: ChainId,
        genesis_config: GenesisConfig,
    ) -> Result<Self> {
        Ok(Self::new(
            ChainNetworkID::new_custom(chain_name, chain_id)?,
            genesis_config,
        ))
    }

    pub fn new_test() -> Self {
        Self::new_builtin(BuiltinNetworkID::Test)
    }

    pub fn id(&self) -> &ChainNetworkID {
        &self.id
    }

    pub fn genesis_config(&self) -> &GenesisConfig {
        &self.genesis_config
    }

    pub fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }

    pub fn stdlib_version(&self) -> StdlibVersion {
        self.genesis_config().stdlib_version
    }

    pub fn chain_id(&self) -> ChainId {
        self.id.chain_id()
    }

    pub fn is_test(&self) -> bool {
        self.id.is_test()
    }

    pub fn is_dev(&self) -> bool {
        self.id.is_dev()
    }

    pub fn is_halley(&self) -> bool {
        self.id.is_halley()
    }

    pub fn is_main(&self) -> bool {
        self.id.is_main()
    }

    pub fn is_custom(&self) -> bool {
        self.id.is_custom()
    }

    pub fn boot_nodes(&self) -> &[Multiaddr] {
        self.id.boot_nodes()
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
        BuiltinNetworkID::Test.chain_id()
    }

    pub fn dev() -> Self {
        BuiltinNetworkID::Dev.chain_id()
    }
}

impl From<ChainNetworkID> for ChainId {
    fn from(id: ChainNetworkID) -> Self {
        id.chain_id()
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
    pub nonce: u32,
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
    pub association_key_pair: (Option<Arc<MultiEd25519KeyShard>>, MultiEd25519PublicKey),
    /// genesis account's key pair, only set at Test and Dev network for test.
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
            let signature = private_key.sign(&txn);
            ensure!(
                signature.is_enough(),
                "association key shard threshold should be 1"
            );
            Ok(SignedUserTransaction::multi_ed25519(
                txn,
                public_key.clone(),
                signature.into(),
            ))
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

    pub fn consensus(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.consensus_config.strategy)
            .expect("consensus strategy config error.")
    }
}

static UNCLE_RATE_TARGET: u64 = 80;
static DEFAULT_BASE_BLOCK_TIME_TARGET: u64 = 10000;
static DEFAULT_BASE_BLOCK_DIFF_WINDOW: u64 = 24;
static BASE_REWARD_PER_UNCLE_PERCENT: u64 = 10;
static MIN_BLOCK_TIME_TARGET: u64 = 10000;
static MAX_BLOCK_TIME_TARGET: u64 = 60000;
static BASE_MAX_UNCLES_PER_BLOCK: u64 = 2;

//for Private funding
static DEFAULT_PRE_MINT_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(15925680));

//for Starcoin Foundation + DevTeam time lock release.
static DEFAULT_TIME_LOCKED_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(8504313 * 3 + 7421367 * 3));

//three years.
static DEFAULT_TIME_LOCKED_PERIOD: u64 = 3600 * 24 * 365 * 3;

static DEFAULT_BASE_REWARD_PER_BLOCK: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(1));

pub static BASE_BLOCK_GAS_LIMIT: u64 = 100_000;

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

static EMPTY_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(Vec::new);
const ONE_DAY: u64 = 86400;

pub static TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_multi_key_pair();
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
            voting_delay: 60_000,          // 1min
            voting_period: 60 * 60 * 1000, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 1000, // 1h
        },
        transaction_timeout: ONE_DAY,
    }
});

pub static DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_multi_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    GenesisConfig {
        version: Version { major: 1 },
        //use latest git commit version's hash
        parent_hash: HashValue::sha3_256_of(b"starcoin_dev"),
        timestamp: 0,
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
        time_service_type: TimeServiceType::MockTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60_000,          // 1min
            voting_period: 60 * 60 * 1000, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 1000, // 1h
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
        parent_hash: HashValue::sha3_256_of(b"starcoin_halley"),
        timestamp: 1603766401000,
        reward_delay: 3,
        difficulty: 10.into(),
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
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (
            None,
            MultiEd25519PublicKey::from_encoded_string("fde53c76807c8a5ec5855ed6200868be8653c34a0f18c6b01f60040ead5daa87b1157be91c2637b709c09ed5d420976c0d4df79537372d69a272fc4869c1364ce3700a1ed3f00ea87c015028cd4a03a4881f6fe203b02f7059db906b764cd23202")
            .expect("create multi public key must success."),
        ),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60_000,       // 1min
            voting_period: 60 * 60 * 1000, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 1000, // 1h
        },
        transaction_timeout: ONE_DAY,
    }
});

pub static PROXIMA_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(|| {
    vec!["/dns4/proxima1.seed.starcoin.org/tcp/9840/p2p/12D3KooW9vHQJk9o69tZPMM2viQ3eWpgp6veDBRz8tTvDFDBejwk".parse().expect("parse multi addr should be ok"),
         "/dns4/proxima2.seed.starcoin.org/tcp/9840/p2p/12D3KooWAua4KokJMiCodGPEF2n4yN42B2Q26KgwrQTntnrCDRHd".parse().expect("parse multi addr should be ok"),
         "/dns4/proxima3.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"), ]
});

pub static PROXIMA_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    GenesisConfig {
    version: Version { major: 1 },
    parent_hash: HashValue::sha3_256_of(b"starcoin_proxima"),
    timestamp: 1603766401000,
    reward_delay: 7,
    difficulty: 100.into(),
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
        strategy: ConsensusStrategy::CryptoNight.value(),
    },
    association_key_pair: (
        None,
        MultiEd25519PublicKey::from_encoded_string("3e6c08fb7f265a35ffd121c809bfa233041d92165c2fdd13f8b85be0814243ba2d616c5105dc8baa39ff764bbcd072e44fcb8bfe5a2f773636285c40d1af15087b00e16ec03438e99858127374c3c148b57a5e10068ca956eff06240c8199f46e4746a6fac58d7d65cfd3ccad4331d071a9ff1a0a29c3bc3896b86c0a7f4ce79e75fbc8422501f5a6bb50ae39e7656949f76d24ce4b677ea224254d8661e509d839e3222ea576580b965d94920765aa1ec62047b7536b0ae57fbdffef968f09e3a5847fb627a9a7909961b21c50c868e26797e2a406879f5cf1d80f4035a448a32fa70d239907d561e116d03dfd9fcba8ab1095117b36b188bf277cc977fc4af87c071e8106a551f0bfe57e9aa2b03d037afd3aaab5c8f0eb56d725f598deada04")
        .expect("create multi public key must success."),
    ),
    genesis_key_pair: None,
    time_service_type: TimeServiceType::RealTimeService,
    stdlib_version: StdlibVersion::Latest,
    dao_config: DaoConfig {
        voting_delay: 60 * 60 * 1000,           // 1h
        voting_period: 60 * 60 * 24 * 2 * 1000, // 2d
        voting_quorum_rate: 4,
        min_action_delay: 60 * 60 * 24 * 1000, // 1d
    },
    transaction_timeout: ONE_DAY,
}
});

pub static MAIN_BOOT_NODES: Lazy<Vec<Multiaddr>> = Lazy::new(Vec::new);

pub static MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    //TODO set public key
    let (_association_private_key, association_public_key) = genesis_multi_key_pair();
    GenesisConfig {
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
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (None, association_public_key),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version: StdlibVersion::Latest,
        dao_config: DaoConfig {
            voting_delay: 60 * 60 * 1000,           // 1h
            voting_period: 60 * 60 * 24 * 2 * 1000, // 2d
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 24 * 1000, // 1d
        },
        transaction_timeout: ONE_DAY,
    }
});
