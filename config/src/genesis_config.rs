// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, format_err, Result};
use network_p2p_types::MultiaddrWithPeerId;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use schemars::{self, JsonSchema};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::multi_ed25519::multi_shard::MultiEd25519KeyShard;
use starcoin_crypto::{
    ed25519::*,
    multi_ed25519::{genesis_multi_key_pair, MultiEd25519PublicKey},
    HashValue, ValidCryptoMaterialStringExt,
};
use starcoin_gas::StarcoinGasParameters;
use starcoin_gas_algebra_ext::{CostTable, FromOnChainGasSchedule};
use starcoin_time_service::{TimeService, TimeServiceType};
use starcoin_uint::U256;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::event::EventHandle;
use starcoin_vm_types::gas_schedule::{
    latest_cost_table, G_GAS_CONSTANTS_V1, G_GAS_CONSTANTS_V2, G_GAS_CONSTANTS_V3,
    G_LATEST_GAS_COST_TABLE, G_TEST_GAS_CONSTANTS,
};
use starcoin_vm_types::genesis_config::{ChainId, ConsensusStrategy, StdlibVersion};
use starcoin_vm_types::on_chain_config::{
    instruction_table_v1, instruction_table_v2, native_table_v1, native_table_v2, v4_native_table,
    ConsensusConfig, DaoConfig, GasSchedule, TransactionPublishOption, VMConfig, Version,
};
use starcoin_vm_types::on_chain_resource::Epoch;
use starcoin_vm_types::token::stc::STCUnit;
use starcoin_vm_types::token::token_value::TokenValue;
use starcoin_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

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
    JsonSchema,
)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Default)]
pub enum BuiltinNetworkID {
    /// A ephemeral network just for unit test.
    Test = 255,
    /// A ephemeral network just for developer test.
    Dev = 254,
    /// Starcoin test network,
    /// The data on the chain will be cleaned up periodically。
    /// Comet Halley, officially designated 1P/Halley, is a short-period comet visible from Earth every 75–76 years.
    Halley = 253,
    /// Starcoin long-running test network,
    /// Proxima Centauri is a small, low-mass star located 4.244 light-years (1.301 pc) away from the Sun in the southern constellation of Centaurus.
    /// Its Latin name means the "nearest [star] of Centaurus".
    Proxima = 252,
    /// Starcoin permanent test network,
    /// Barnard's Star is a red dwarf about six light-years away from Earth in the constellation of Ophiuchus.
    Barnard = 251,
    /// A ephemeral dag-related network just for developer test.
    DagTest = 250,
    /// Starcoin main net.
    #[default]
    Main = 1,
}

impl Display for BuiltinNetworkID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinNetworkID::Test => write!(f, "test"),
            BuiltinNetworkID::Dev => write!(f, "dev"),
            BuiltinNetworkID::Halley => write!(f, "halley"),
            BuiltinNetworkID::Proxima => write!(f, "proxima"),
            BuiltinNetworkID::Barnard => write!(f, "barnard"),
            BuiltinNetworkID::Main => write!(f, "main"),
            BuiltinNetworkID::DagTest => write!(f, "dagtest"),
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
            "barnard" => Ok(BuiltinNetworkID::Barnard),
            "main" => Ok(BuiltinNetworkID::Main),
            "dagtest" => Ok(BuiltinNetworkID::DagTest),
            s => Err(format_err!("Unknown network: {}", s)),
        }
    }
}
impl TryFrom<ChainId> for BuiltinNetworkID {
    type Error = anyhow::Error;
    fn try_from(id: ChainId) -> Result<Self, Self::Error> {
        Ok(match id.id() {
            255 => Self::Test,
            254 => Self::Dev,
            253 => Self::Halley,
            252 => Self::Proxima,
            251 => Self::Barnard,
            250 => Self::DagTest,
            1 => Self::Main,
            id => bail!("{} is not a builtin chain id", id),
        })
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
        matches!(self, BuiltinNetworkID::Test | BuiltinNetworkID::Dev)
    }

    pub fn is_test(self) -> bool {
        matches!(self, BuiltinNetworkID::Test)
    }

    pub fn is_dev(self) -> bool {
        matches!(self, BuiltinNetworkID::Dev)
    }

    pub fn is_main(self) -> bool {
        matches!(self, BuiltinNetworkID::Main)
    }

    pub fn is_halley(self) -> bool {
        matches!(self, BuiltinNetworkID::Halley)
    }

    pub fn networks() -> Vec<BuiltinNetworkID> {
        vec![
            BuiltinNetworkID::Test,
            BuiltinNetworkID::Dev,
            BuiltinNetworkID::Halley,
            BuiltinNetworkID::Proxima,
            BuiltinNetworkID::Barnard,
            BuiltinNetworkID::Main,
            BuiltinNetworkID::DagTest,
        ]
    }

    pub fn genesis_config(self) -> &'static GenesisConfig {
        match self {
            BuiltinNetworkID::Test => &G_TEST_CONFIG,
            BuiltinNetworkID::Dev => &G_DEV_CONFIG,
            BuiltinNetworkID::Halley => &G_HALLEY_CONFIG,
            BuiltinNetworkID::Proxima => &G_PROXIMA_CONFIG,
            BuiltinNetworkID::Barnard => &G_BARNARD_CONFIG,
            BuiltinNetworkID::Main => &G_MAIN_CONFIG,
            BuiltinNetworkID::DagTest => &G_DAG_TEST_CONFIG,
        }
    }

    pub fn boot_nodes(self) -> &'static [MultiaddrWithPeerId] {
        match self {
            BuiltinNetworkID::Test => G_EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Dev => G_EMPTY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Halley => G_HALLEY_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Proxima => G_PROXIMA_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Barnard => G_BARNARD_BOOT_NODES.as_slice(),
            BuiltinNetworkID::Main => G_MAIN_BOOT_NODES.as_slice(),
            BuiltinNetworkID::DagTest => G_EMPTY_BOOT_NODES.as_slice(),
        }
    }

    pub fn boot_nodes_domain(self) -> String {
        match self {
            BuiltinNetworkID::Test | BuiltinNetworkID::Dev | BuiltinNetworkID::DagTest => {
                "localhost".to_string()
            }
            _ => format!("{}.seed.starcoin.org", self),
        }
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
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

#[derive(Clone, Debug, PartialEq, Eq, JsonSchema)]
#[allow(clippy::upper_case_acronyms)]
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
    pub const BARNARD: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Barnard);
    pub const MAIN: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::Main);
    pub const DAGTEST: ChainNetworkID = ChainNetworkID::Builtin(BuiltinNetworkID::DagTest);

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
        self.is_test() || self.is_dev() || self.is_dag_test()
    }

    pub fn is_dag_test(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinNetworkID::DagTest))
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinNetworkID::Test))
    }

    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinNetworkID::Dev))
    }

    pub fn is_main(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinNetworkID::Main))
    }

    pub fn is_halley(&self) -> bool {
        matches!(self, Self::Builtin(BuiltinNetworkID::Halley))
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Default data dir name of this network
    pub fn dir_name(&self) -> String {
        match self {
            Self::Builtin(net) => net.to_string(),
            Self::Custom(net) => net.chain_name().to_string(),
        }
    }

    pub fn boot_nodes(&self) -> &[MultiaddrWithPeerId] {
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

    pub fn limit_peers(&self) -> u8 {
        match self {
            Self::Builtin(BuiltinNetworkID::Main) => 5,
            _ => 1,
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

    pub fn is_dag_test(&self) -> bool {
        self.id.is_custom()
    }

    pub fn is_main(&self) -> bool {
        self.id.is_main()
    }

    pub fn is_custom(&self) -> bool {
        self.id.is_custom()
    }

    pub fn boot_nodes(&self) -> &[MultiaddrWithPeerId] {
        self.id.boot_nodes()
    }

    /// Please ensure network is_ready() before genesis_block_parameter
    pub fn genesis_block_parameter(&self) -> &GenesisBlockParameter {
        self.genesis_config
            .genesis_block_parameter()
            .expect("Genesis block parameter is not ready")
    }

    /// This network is ready to launch
    pub fn is_ready(&self) -> bool {
        self.genesis_config.is_ready()
    }

    /// resolve the FutureBlockParameter to static GenesisBlockParameter.
    pub fn resolve(&mut self, resolver: &dyn FutureBlockParameterResolver) -> Result<()> {
        match &self.genesis_config.genesis_block_parameter {
            GenesisBlockParameterConfig::Static(_) => {}
            GenesisBlockParameterConfig::FutureBlock(future_block_parameter) => {
                let parameter = resolver.resolve(future_block_parameter)?;
                self.genesis_config.genesis_block_parameter =
                    GenesisBlockParameterConfig::Static(parameter);
            }
        }
        Ok(())
    }

    pub fn genesis_epoch(&self) -> Epoch {
        Epoch::new(
            0,
            self.genesis_block_parameter().timestamp,
            0,
            self.genesis_config.consensus_config.epoch_block_count,
            self.genesis_config.consensus_config.base_block_time_target,
            self.genesis_config.consensus_config.base_reward_per_block,
            self.genesis_config
                .consensus_config
                .base_reward_per_uncle_percent,
            self.genesis_config
                .consensus_config
                .base_block_difficulty_window,
            self.genesis_config
                .consensus_config
                .base_max_uncles_per_block,
            self.genesis_config.consensus_config.base_block_gas_limit,
            self.genesis_config.consensus_config.strategy,
            //TODO conform new Epoch events salt value.
            EventHandle::new_from_address(&genesis_address(), 0),
        )
    }

    pub fn min_peers(&self) -> u8 {
        self.id.limit_peers()
    }
}

pub trait FutureBlockParameterResolver {
    fn resolve(&self, parameter: &FutureBlockParameter) -> Result<GenesisBlockParameter>;
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GenesisBlockParameter {
    /// Genesis block parent hash
    pub parent_hash: HashValue,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Genesis difficulty
    pub difficulty: U256,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct FutureBlockParameter {
    pub network: BuiltinNetworkID,
    pub block_number: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum GenesisBlockParameterConfig {
    Static(GenesisBlockParameter),
    FutureBlock(FutureBlockParameter),
}

/// GenesisConfig is a config for initialize a chain genesis.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct GenesisConfig {
    /// Parameter for init genesis block
    pub genesis_block_parameter: GenesisBlockParameterConfig,
    /// Starcoin system major version for genesis.
    pub version: Version,
    /// How many block to delay before rewarding miners.
    pub reward_delay: u64,
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
    pub publishing_option: TransactionPublishOption,
    /// consensus config
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

    /// Flexidag effective height
    pub dag_effective_height: u64,
}

impl GenesisConfig {
    /// Get the genesis block parent_hash, timestamp and difficulty
    pub fn genesis_block_parameter(&self) -> Option<&GenesisBlockParameter> {
        if let GenesisBlockParameterConfig::Static(parameter) = &self.genesis_block_parameter {
            Some(parameter)
        } else {
            None
        }
    }

    pub fn is_ready(&self) -> bool {
        self.genesis_block_parameter().is_some()
    }

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
        let buf = serde_json::to_vec_pretty(self)?;
        file.write_all(buf.as_slice())?;
        Ok(())
    }

    pub fn consensus(&self) -> ConsensusStrategy {
        ConsensusStrategy::try_from(self.consensus_config.strategy)
            .expect("consensus strategy config error.")
    }
}

static G_UNCLE_RATE_TARGET: u64 = 240;
static G_DEFAULT_BASE_BLOCK_TIME_TARGET: u64 = 10000;
static G_DEFAULT_BASE_BLOCK_DIFF_WINDOW: u64 = 24;
static G_BASE_REWARD_PER_UNCLE_PERCENT: u64 = 10;
static G_MIN_BLOCK_TIME_TARGET: u64 = 5000;
static G_MAX_BLOCK_TIME_TARGET: u64 = 60000;
pub static G_BASE_MAX_UNCLES_PER_BLOCK: u64 = 2;

pub static G_TOTAL_STC_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(3185136000));

//for Private funding
static G_DEFAULT_PRE_MINT_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(159256800));

//for Starcoin Foundation + DevTeam time lock release.
static G_DEFAULT_TIME_LOCKED_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(85043130 * 3 + 74213670 * 3));

//three years.
static G_DEFAULT_TIME_LOCKED_PERIOD: u64 = 3600 * 24 * 365 * 3;

static G_DEFAULT_BASE_REWARD_PER_BLOCK: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(10));

pub static G_BASE_BLOCK_GAS_LIMIT: u64 = 50_000_000; //must big than maximum_number_of_gas_units

static G_EMPTY_BOOT_NODES: Lazy<Vec<MultiaddrWithPeerId>> = Lazy::new(Vec::new);
const ONE_DAY: u64 = 86400;

pub static G_DAG_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_multi_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter {
            parent_hash: HashValue::sha3_256_of(b"starcoin_dag_test"),
            //Test timestamp set to 0 for mock time.
            timestamp: 0,
            difficulty: 1.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 1,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: G_DEFAULT_TIME_LOCKED_PERIOD,
        vm_config: VMConfig {
            gas_schedule: latest_cost_table(G_TEST_GAS_CONSTANTS.clone()),
        },
        publishing_option: TransactionPublishOption::open(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 2,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT * 10,
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
        dag_effective_height: 0,
    }
});

pub static G_TEST_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_multi_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter {
            parent_hash: HashValue::sha3_256_of(b"starcoin_test"),
            //Test timestamp set to 0 for mock time.
            timestamp: 0,
            difficulty: 1.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 1,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: G_DEFAULT_TIME_LOCKED_PERIOD,
        vm_config: VMConfig {
            gas_schedule: latest_cost_table(G_TEST_GAS_CONSTANTS.clone()),
        },
        publishing_option: TransactionPublishOption::open(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 2,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT * 10,
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
        dag_effective_height: u64::MAX,
    }
});

pub static G_DEV_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_multi_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    let mut gas_constant = G_TEST_GAS_CONSTANTS.clone();
    gas_constant.min_price_per_gas_unit = 1;

    let stdlib_version = StdlibVersion::Latest;
    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter {
            parent_hash: HashValue::sha3_256_of(b"starcoin_dev"),
            timestamp: 0,
            difficulty: 1.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 1,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: 3600 * 24,
        vm_config: VMConfig {
            gas_schedule: latest_cost_table(gas_constant),
        },
        publishing_option: TransactionPublishOption::open(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 2,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT * 10,
            strategy: ConsensusStrategy::Dummy.value(),
        },
        association_key_pair: (
            Some(Arc::new(association_private_key)),
            association_public_key,
        ),
        genesis_key_pair: Some((Arc::new(genesis_private_key), genesis_public_key)),
        time_service_type: TimeServiceType::MockTimeService,
        stdlib_version,
        dao_config: DaoConfig {
            voting_delay: 60_000,          // 1min
            voting_period: 60 * 60 * 1000, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 1000, // 1h
        },
        transaction_timeout: ONE_DAY,
        dag_effective_height: u64::MAX,
    }
});

pub static G_HALLEY_BOOT_NODES: Lazy<Vec<MultiaddrWithPeerId>> = Lazy::new(|| {
    vec!["/dns4/halley1.seed.starcoin.org/tcp/9840/p2p/12D3KooW9yQoKZrByqrUjmmPHXtR23qCXRQvF5KowYgoqypuhuCn".parse().expect("parse multi addr should be ok"), ]
});

pub static G_HALLEY_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    // let stdlib_version = StdlibVersion::Latest;
    let stdlib_version = StdlibVersion::Version(12);
    let association_public_key = "068b8493d8c533fd08568429274e49639518a8517f6ab03a0f0cc37edcbdfdd0071855fd941dbcefeb9e4da9f417c7b0f39f73226c9310d39881ae13b45017fa67cc9cb01386e9f5e321b078d4d3a2925b520f955cf7dfd9f6891de366c186ce6ec4a3d5a1c6c795126e5ee1222e23f9a28266c07ecce3e2cd19c6e123b465c091bc45a1fa7f778c66c37af15f3e81ff511e69ff0481bcfaab7b4673f469a3d29760cacf5dd0105a541b5f50720b9577a4c3ff7475554afedbf6a884777f9db4c461fe9aca18df90ed31ee967fe49ed47756311eaa2a6042b7aff1422e48643dc7a0004e0ca3e6b8e548c80d76eeb88e84a82f6b863a1346eabadfe4d5d9be86f98fa72c63f1e1a3f193d4ff71e10dbf364200b221e1a7f71cfab55cc7f7ad2a05";

    let mut gas_constant = G_TEST_GAS_CONSTANTS.clone();
    gas_constant.min_price_per_gas_unit = 1;

    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter {
            parent_hash: HashValue::sha3_256_of(b"starcoin_halley"),
            timestamp: 1713105562000,
            difficulty: 100.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 3,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: 3600 * 24 * 31,
        vm_config: VMConfig {
            gas_schedule: latest_cost_table(gas_constant),
        },
        publishing_option: TransactionPublishOption::open(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT * 10,
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (
            None,
            MultiEd25519PublicKey::from_encoded_string(association_public_key)
                .expect("create multi public key must success."),
        ),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version,
        dao_config: DaoConfig {
            voting_delay: 60_000,          // 1min
            voting_period: 60 * 60 * 1000, // 1h
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 1000, // 1h
        },
        transaction_timeout: ONE_DAY,
        // todo: rollback it to zero and initialize BlockDag properly
        dag_effective_height: 1u64,
    }
});

pub static G_PROXIMA_BOOT_NODES: Lazy<Vec<MultiaddrWithPeerId>> = Lazy::new(|| {
    vec!["/dns4/proxima1.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"),
      ]
});

pub static G_PROXIMA_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let stdlib_version = StdlibVersion::Version(12);
    let association_public_key = "068b8493d8c533fd08568429274e49639518a8517f6ab03a0f0cc37edcbdfdd0071855fd941dbcefeb9e4da9f417c7b0f39f73226c9310d39881ae13b45017fa67cc9cb01386e9f5e321b078d4d3a2925b520f955cf7dfd9f6891de366c186ce6ec4a3d5a1c6c795126e5ee1222e23f9a28266c07ecce3e2cd19c6e123b465c091bc45a1fa7f778c66c37af15f3e81ff511e69ff0481bcfaab7b4673f469a3d29760cacf5dd0105a541b5f50720b9577a4c3ff7475554afedbf6a884777f9db4c461fe9aca18df90ed31ee967fe49ed47756311eaa2a6042b7aff1422e48643dc7a0004e0ca3e6b8e548c80d76eeb88e84a82f6b863a1346eabadfe4d5d9be86f98fa72c63f1e1a3f193d4ff71e10dbf364200b221e1a7f71cfab55cc7f7ad2a05";
    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter {
            parent_hash: HashValue::sha3_256_of(b"starcoin_proxima"),
            timestamp: 1661944080000,
            difficulty: 100.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 7,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: G_DEFAULT_TIME_LOCKED_PERIOD / 12,
        vm_config: VMConfig {
            gas_schedule: CostTable {
                instruction_table: instruction_table_v2(),
                native_table: v4_native_table(),
                gas_constants: G_GAS_CONSTANTS_V3.clone(),
            },
        },
        publishing_option: TransactionPublishOption::open(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (
            None,
            MultiEd25519PublicKey::from_encoded_string(association_public_key)
                .expect("create multi public key must success."),
        ),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version,
        dao_config: DaoConfig {
            voting_delay: 60 * 1000,       // 1 minute
            voting_period: 10 * 60 * 1000, // 30 minute
            voting_quorum_rate: 2,
            min_action_delay: 60 * 1000, // 1 minute
        },
        transaction_timeout: ONE_DAY,
        // todo: rollback it to zero and initialize BlockDag properly
        dag_effective_height: 1u64,
    }
});

pub static G_BARNARD_BOOT_NODES: Lazy<Vec<MultiaddrWithPeerId>> = Lazy::new(|| {
    vec![
        "/dns4/barnard4.seed.starcoin.org/tcp/9840/p2p/12D3KooWRUQ4CZ6tiy2kZo5vVjm27ksYJAqMwB2QPfqpB5WEfzy4".parse().expect("parse multi addr should be ok"),
        "/dns4/barnard5.seed.starcoin.org/tcp/9840/p2p/12D3KooWPwRSY555ycvo8BNiEqWqaJRvgtkv7BfJhq9JWHty6e2R".parse().expect("parse multi addr should be ok"),
        "/dns4/barnard6.seed.starcoin.org/tcp/9840/p2p/12D3KooWSMJRCgT4inuEZNxvjSCHY1d3DwVX3SQ6qrvqAZCLLMwJ".parse().expect("parse multi addr should be ok"),
    ]
});

pub static G_BARNARD_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    // This is a test config,
    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter{
            parent_hash: HashValue::from_hex_literal("0x3a06de3042a4b8fe156c4ae88d93e7a2e23d621965eddf46351d13d3e8ba3bb6").unwrap(),
            timestamp: 1616846974851,
            difficulty: 0x03bd.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 7,
        pre_mine_amount: STCUnit::STC.value_of(15925680).scaling(),
        time_mint_amount: STCUnit::STC.value_of(47777040).scaling(),
        time_mint_period: G_DEFAULT_TIME_LOCKED_PERIOD,
        vm_config: VMConfig {
            gas_schedule: CostTable {
                instruction_table: instruction_table_v1(),
                native_table: native_table_v1(),
                gas_constants: G_GAS_CONSTANTS_V1.clone(),
            },
        },
        publishing_option: TransactionPublishOption::locked(),
        consensus_config: ConsensusConfig {
            uncle_rate_target: 500,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: STCUnit::STC.value_of(1).scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (None,  MultiEd25519PublicKey::from_encoded_string("3e6c08fb7f265a35ffd121c809bfa233041d92165c2fdd13f8b85be0814243ba2d616c5105dc8baa39ff764bbcd072e44fcb8bfe5a2f773636285c40d1af15087b00e16ec03438e99858127374c3c148b57a5e10068ca956eff06240c8199f46e4746a6fac58d7d65cfd3ccad4331d071a9ff1a0a29c3bc3896b86c0a7f4ce79e75fbc8422501f5a6bb50ae39e7656949f76d24ce4b677ea224254d8661e509d839e3222ea576580b965d94920765aa1ec62047b7536b0ae57fbdffef968f09e3a5847fb627a9a7909961b21c50c868e26797e2a406879f5cf1d80f4035a448a32fa70d239907d561e116d03dfd9fcba8ab1095117b36b188bf277cc977fc4af87c071e8106a551f0bfe57e9aa2b03d037afd3aaab5c8f0eb56d725f598deada04")
            .expect("create multi public key must success.")),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version: StdlibVersion::Version(1),
        dao_config: DaoConfig {
            voting_delay: 60 * 60 * 1000,           // 1h
            voting_period: 60 * 60 * 24 * 1000, // 1d
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 24 * 1000, // 1d
        },
        transaction_timeout: ONE_DAY,
        dag_effective_height: u64::MAX,
    }
});

pub static G_MAIN_BOOT_NODES: Lazy<Vec<MultiaddrWithPeerId>> = Lazy::new(|| {
    vec![
    "/dns4/main1.seed.starcoin.org/tcp/9840/p2p/12D3KooWSW8t32L6VyjPZxxe3VSD7H6Ffgh69eCaDnDyab2M6tfK".parse().expect("parse multi addr should be ok"),
    "/dns4/main2.seed.starcoin.org/tcp/9840/p2p/12D3KooWBWsibdKdogDpFUfSVejPdK6t4s1eEvcqjuPVJ3gSpypx".parse().expect("parse multi addr should be ok"),
    "/dns4/main3.seed.starcoin.org/tcp/9840/p2p/12D3KooWB9vGtpgqyD2cG4PTEU1SHSuWV6PErMPJFbbi5vYpkj3H".parse().expect("parse multi addr should be ok"),
    "/dns4/main4.seed.starcoin.org/tcp/9840/p2p/12D3KooWKqZ2k2qQWY1khPH6WV2qFD5X2TZrpRMj75MHxCd8VH5r".parse().expect("parse multi addr should be ok"),
    "/dns4/main5.seed.starcoin.org/tcp/9840/p2p/12D3KooW9quK2EEjeyTs3csNRWPnfMw4M3afGE1SHm1dCZDRWSAj".parse().expect("parse multi addr should be ok"),
    "/dns4/main6.seed.starcoin.org/tcp/9840/p2p/12D3KooWH13WqMtEPQfEHHU8riaHt6L2oPLvN7GTin14AziTdukw".parse().expect("parse multi addr should be ok"),
    "/dns4/main7.seed.starcoin.org/tcp/9840/p2p/12D3KooWMuvSkk51syDSSesKs4QmApETBBfC2FWfA4b59vEpqtH9".parse().expect("parse multi addr should be ok"),
    "/dns4/main8.seed.starcoin.org/tcp/9840/p2p/12D3KooWQajuoiuY1Ba4Cz2Z7fGpNK38hKwzECGJQyCWnRb17JJ4".parse().expect("parse multi addr should be ok"),
    "/dns4/main9.seed.starcoin.org/tcp/9840/p2p/12D3KooWLKo5X7yntEaAhUTh62ksD8pwsSu7CyTgZ76bRcStHF7x".parse().expect("parse multi addr should be ok"),
]
});

pub static G_MAIN_CONFIG: Lazy<GenesisConfig> = Lazy::new(|| {
    let stdlib_version = StdlibVersion::Version(4);
    let publishing_option = TransactionPublishOption::locked();
    GenesisConfig {
        genesis_block_parameter: GenesisBlockParameterConfig::Static(GenesisBlockParameter{
            parent_hash: HashValue::from_hex_literal("0xb82a2c11f2df62bf87c2933d0281e5fe47ea94d5f0049eec1485b682df29529a").unwrap(),
            timestamp: 1621311100863,
            difficulty: 0xb1ec37.into(),
        }),
        version: Version { major: 1 },
        reward_delay: 7,
        pre_mine_amount: G_DEFAULT_PRE_MINT_AMOUNT.scaling(),
        time_mint_amount: G_DEFAULT_TIME_LOCKED_AMOUNT.scaling(),
        time_mint_period: G_DEFAULT_TIME_LOCKED_PERIOD,
        vm_config: VMConfig {
            gas_schedule: CostTable {
                instruction_table: instruction_table_v1(),
                native_table: native_table_v2(),
                gas_constants: G_GAS_CONSTANTS_V2.clone(),
            },
        },
        publishing_option,
        consensus_config: ConsensusConfig {
            uncle_rate_target: G_UNCLE_RATE_TARGET,
            base_block_time_target: G_DEFAULT_BASE_BLOCK_TIME_TARGET,
            base_reward_per_block: G_DEFAULT_BASE_REWARD_PER_BLOCK.scaling(),
            epoch_block_count: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW * 10,
            base_block_difficulty_window: G_DEFAULT_BASE_BLOCK_DIFF_WINDOW,
            base_reward_per_uncle_percent: G_BASE_REWARD_PER_UNCLE_PERCENT,
            min_block_time_target: G_MIN_BLOCK_TIME_TARGET,
            max_block_time_target: G_MAX_BLOCK_TIME_TARGET,
            base_max_uncles_per_block: G_BASE_MAX_UNCLES_PER_BLOCK,
            base_block_gas_limit: G_BASE_BLOCK_GAS_LIMIT,
            strategy: ConsensusStrategy::CryptoNight.value(),
        },
        association_key_pair: (None,  MultiEd25519PublicKey::from_encoded_string("810a82a896a4f8fd065bcab8b06588fe1afdbb3d3830693c65a73d31ee1e482d85a40286b624b8481b05d9ed748e7c051b63ed36ce952cbc48bb0de4bfc6ec5888feded087075af9585a83c777ba52da1ab3aef139764a0de5fbc2d8aa8d380b02")
            .expect("create multi public key must success.")),
        genesis_key_pair: None,
        time_service_type: TimeServiceType::RealTimeService,
        stdlib_version,
        dao_config: DaoConfig {
            voting_delay: 60 * 60 * 1000,           // 1h
            voting_period: 60 * 60 * 24 * 7 * 1000, // 7d
            voting_quorum_rate: 4,
            min_action_delay: 60 * 60 * 24 * 1000, // 1d
        },
        transaction_timeout: ONE_DAY,
        dag_effective_height: u64::MAX,
    }
});

pub static G_LATEST_GAS_PARAMS: Lazy<StarcoinGasParameters> = Lazy::new(|| {
    let gas_schedule = GasSchedule::from(&G_LATEST_GAS_COST_TABLE.clone());
    StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map()).unwrap()
});

#[cfg(test)]
mod tests {
    use starcoin_gas::StarcoinGasParameters;
    use starcoin_gas_algebra_ext::{CostTable, FromOnChainGasSchedule};
    use starcoin_vm_types::gas_schedule::{
        latest_cost_table, G_GAS_CONSTANTS_V1, G_GAS_CONSTANTS_V2, G_LATEST_GAS_COST_TABLE,
        G_TEST_GAS_CONSTANTS,
    };
    use starcoin_vm_types::on_chain_config::{
        instruction_gas_schedule_v1, instruction_gas_schedule_v2, instruction_table_v1,
        native_gas_schedule_v1, native_gas_schedule_v2, native_gas_schedule_v4, native_table_v1,
        native_table_v2, txn_gas_schedule_test, txn_gas_schedule_v1, txn_gas_schedule_v2,
        txn_gas_schedule_v3, GasSchedule,
    };

    fn config_entries(
        instrs: Vec<(String, u64)>,
        natives: Vec<(String, u64)>,
        constants: Vec<(String, u64)>,
    ) -> Vec<(String, u64)> {
        let mut entries = instrs;
        let mut natives = natives;
        let mut constants = constants;
        entries.push(("instr.ld_u16".to_string(), 3));
        entries.push(("instr.ld_u32".to_string(), 2));
        entries.push(("instr.ld_u256".to_string(), 3));
        entries.push(("instr.cast_u16".to_string(), 3));
        entries.push(("instr.cast_u32".to_string(), 2));
        entries.push(("instr.cast_u256".to_string(), 3));
        entries.append(&mut natives);
        // native_table don't have these
        entries.push(("nursery.debug.print.base_cost".to_string(), 1));
        entries.push(("nursery.debug.print_stack_trace.base_cost".to_string(), 1));

        entries.push((
            "move_stdlib.hash.sha2_256.legacy_min_input_len".to_string(),
            1,
        ));
        entries.push((
            "move_stdlib.hash.sha3_256.legacy_min_input_len".to_string(),
            1,
        ));
        entries.push(("move_stdlib.bcs.to_bytes.failure".to_string(), 182));
        entries.push((
            "move_stdlib.bcs.to_bytes.legacy_min_output_size".to_string(),
            1,
        ));
        entries.append(&mut constants);
        entries
    }

    #[test]
    fn test_dev_config() {
        let entries = config_entries(
            instruction_gas_schedule_v2(),
            native_gas_schedule_v4(),
            txn_gas_schedule_test(),
        );

        let gas_schedule = GasSchedule::from(&latest_cost_table(G_TEST_GAS_CONSTANTS.clone()));
        assert_eq!(entries, gas_schedule.entries);
        let gas_params =
            StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map());
        assert_eq!(
            gas_params.unwrap().natives.nursery.debug.print.base_cost,
            1.into()
        );
    }

    #[test]
    fn test_halley_config() {
        let entries = config_entries(
            instruction_gas_schedule_v2(),
            native_gas_schedule_v4(),
            txn_gas_schedule_v3(),
        );

        let gas_schedule = GasSchedule::from(&G_LATEST_GAS_COST_TABLE.clone());
        assert_eq!(entries, gas_schedule.entries);
        let gas_params =
            StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map());
        assert_eq!(
            gas_params.unwrap().natives.nursery.debug.print.base_cost,
            1.into()
        );
    }

    #[test]
    fn test_proxima_config() {
        let entries = config_entries(
            instruction_gas_schedule_v2(),
            native_gas_schedule_v4(),
            txn_gas_schedule_v3(),
        );
        let gas_schedule = GasSchedule::from(&G_LATEST_GAS_COST_TABLE.clone());
        assert_eq!(entries, gas_schedule.entries);
        let gas_params =
            StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map());
        assert_eq!(
            gas_params.unwrap().natives.nursery.debug.print.base_cost,
            1.into()
        );
    }

    #[test]
    fn test_barnard_config() {
        let entries = config_entries(
            instruction_gas_schedule_v1(),
            native_gas_schedule_v1(),
            txn_gas_schedule_v1(),
        );
        let gas_schedule = GasSchedule::from(&CostTable {
            instruction_table: instruction_table_v1(),
            native_table: native_table_v1(),
            gas_constants: G_GAS_CONSTANTS_V1.clone(),
        });
        assert_eq!(entries, gas_schedule.entries);
        let gas_params =
            StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map());
        assert_eq!(
            gas_params.unwrap().natives.nursery.debug.print.base_cost,
            1.into()
        );
    }

    #[test]
    fn test_main_config() {
        let entries = config_entries(
            instruction_gas_schedule_v1(),
            native_gas_schedule_v2(),
            txn_gas_schedule_v2(),
        );

        let gas_schedule = GasSchedule::from(&CostTable {
            instruction_table: instruction_table_v1(),
            native_table: native_table_v2(),
            gas_constants: G_GAS_CONSTANTS_V2.clone(),
        });
        assert_eq!(entries, gas_schedule.entries);
        let gas_params =
            StarcoinGasParameters::from_on_chain_gas_schedule(&gas_schedule.to_btree_map());
        assert_eq!(
            gas_params.unwrap().natives.nursery.debug.print.base_cost,
            1.into()
        );
    }
}
