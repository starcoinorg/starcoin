// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::{VMConfig, VMPublishingOption, Version, INITIAL_GAS_SCHEDULE};
use crate::transaction::{RawUserTransaction, SignedUserTransaction};
use anyhow::{bail, format_err, Result};
use ethereum_types::U256;
use libp2p::Multiaddr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{ed25519::*, Genesis, HashValue, PrivateKey, ValidCryptoMaterialStringExt};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[repr(u8)]
#[serde(tag = "type")]
pub enum ConsensusStrategy {
    Dummy = 0,
    Dev = 1,
    Argon = 2,
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
#[serde(tag = "net")]
pub enum ChainNetwork {
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

impl Display for ChainNetwork {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainNetwork::Test => write!(f, "test"),
            ChainNetwork::Dev => write!(f, "dev"),
            ChainNetwork::Halley => write!(f, "halley"),
            ChainNetwork::Proxima => write!(f, "proxima"),
            ChainNetwork::Main => write!(f, "main"),
        }
    }
}

impl FromStr for ChainNetwork {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test" => Ok(ChainNetwork::Test),
            "dev" => Ok(ChainNetwork::Dev),
            "halley" => Ok(ChainNetwork::Halley),
            "proxima" => Ok(ChainNetwork::Proxima),
            s => Err(format_err!("Unknown network: {}", s)),
        }
    }
}

impl ChainNetwork {
    pub fn chain_id(self) -> ChainId {
        ChainId(self.into())
    }

    pub fn is_test(self) -> bool {
        match self {
            ChainNetwork::Test => true,
            _ => false,
        }
    }

    pub fn is_dev(self) -> bool {
        match self {
            ChainNetwork::Dev => true,
            _ => false,
        }
    }

    pub fn is_main(self) -> bool {
        match self {
            ChainNetwork::Main => true,
            _ => false,
        }
    }

    pub fn is_halley(self) -> bool {
        match self {
            ChainNetwork::Halley => true,
            _ => false,
        }
    }

    pub fn get_config(self) -> &'static ChainConfig {
        match self {
            ChainNetwork::Test => &TEST_CHAIN_CONFIG,
            ChainNetwork::Dev => &DEV_CHAIN_CONFIG,
            ChainNetwork::Halley => &HALLEY_CHAIN_CONFIG,
            ChainNetwork::Proxima => &PROXIMA_CHAIN_CONFIG,
            ChainNetwork::Main => &MAIN_CHAIN_CONFIG,
        }
    }

    pub fn consensus(self) -> ConsensusStrategy {
        self.get_config().consensus_strategy
    }

    pub fn sign_with_association(self, txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        if let (Some(private_key), public_key) = &self.get_config().association_key_pair {
            Ok(txn.sign(private_key, public_key.clone())?.into_inner())
        } else {
            bail!(
                "association private_key not config at current network: {}.",
                self
            )
        }
    }

    pub fn sign_with_genesis(self, txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        if let Some((private_key, public_key)) = &self.get_config().genesis_key_pair {
            Ok(txn.sign(private_key, public_key.clone())?.into_inner())
        } else {
            bail!(
                "genesis private_key not config at current network: {}.",
                self
            )
        }
    }

    pub fn networks() -> Vec<ChainNetwork> {
        vec![
            ChainNetwork::Test,
            ChainNetwork::Dev,
            ChainNetwork::Halley,
            ChainNetwork::Proxima,
            ChainNetwork::Main,
        ]
    }
}

impl Default for ChainNetwork {
    fn default() -> Self {
        ChainNetwork::Dev
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ChainId(u8);

impl ChainId {
    pub fn new(id: u8) -> Self {
        ChainId(id)
    }

    pub fn id(self) -> u8 {
        self.0
    }

    pub fn test() -> Self {
        ChainNetwork::Test.chain_id()
    }

    pub fn dev() -> Self {
        ChainNetwork::Dev.chain_id()
    }
}

/// ChainConfig is a static hard code config.
#[derive(Debug)]
pub struct ChainConfig {
    /// Starcoin system major version for genesis.
    pub version: Version,
    /// Genesis block parent hash
    pub parent_hash: HashValue,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Starcoin total supply.
    pub total_supply: u128,
    /// How many block to delay before rewarding miners.
    pub reward_delay: u64,
    /// Genesis difficulty, should match consensus in different ChainNetwork.
    pub difficulty: U256,
    /// Genesis consensus nonce.
    pub nonce: u64,
    /// pre mine to Association account, percent of total_supply, from 0~100.
    pub pre_mine_percent: u64,
    /// VM config for publishing_option and gas_schedule
    pub vm_config: VMConfig,
    /// List of initial node addresses
    pub boot_nodes: Vec<Multiaddr>,
    /// uncle rate target
    pub uncle_rate_target: u64,
    /// epoch time target
    pub epoch_time_target: u64,
    /// reward half epoch
    pub reward_half_epoch: u64,
    /// init first epoch
    pub init_block_time_target: u64,
    /// block window
    pub block_difficulty_window: u64,
    /// reward per uncle percent
    pub reward_per_uncle_percent: u64,
    /// min time target
    pub min_time_target: u64,
    /// max uncle block count per block
    pub max_uncles_per_block: u64,
    /// association account's key pair
    pub association_key_pair: (Option<Ed25519PrivateKey>, Ed25519PublicKey),
    /// genesis account's key pair
    pub genesis_key_pair: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,
    /// consensus strategy for chain
    pub consensus_strategy: ConsensusStrategy,
}

pub static STARCOIN_TOTAL_SUPPLY: u128 = 2_100_000_000 * 1_000_000;
pub static EPOCH_TIME_TARGET: u64 = 1_209_600;
pub static REWARD_HALF_TIME_TARGET: u64 = 126_144_000;
pub static INIT_BLOCK_TIME_TARGET: u64 = 20;
pub static BLOCK_DIFF_WINDOW: u64 = 24;
pub static REWARD_PER_UNCLE_PERCENT: u64 = 10;
pub static MIN_TIME_TARGET: u64 = 10;
pub static MAX_UNCLES_PER_BLOCK: u64 = 2;

pub static DEV_EPOCH_TIME_TARGET: u64 = 60;
pub static DEV_MIN_TIME_TARGET: u64 = 1;

pub static TEST_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    ChainConfig {
        version: Version { major: 1 },
        parent_hash: HashValue::random(),
        //Test timestamp set to 0 for mock time.
        timestamp: 0,
        total_supply: STARCOIN_TOTAL_SUPPLY,
        reward_delay: 1,
        difficulty: 1.into(),
        nonce: 0,
        pre_mine_percent: 20,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        boot_nodes: vec![],
        uncle_rate_target: 80,
        epoch_time_target: DEV_EPOCH_TIME_TARGET,
        reward_half_epoch: DEV_EPOCH_TIME_TARGET * 2 / DEV_EPOCH_TIME_TARGET,
        init_block_time_target: 5,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_time_target: DEV_MIN_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (Some(association_private_key), association_public_key),
        genesis_key_pair: Some((genesis_private_key, genesis_public_key)),
        consensus_strategy: ConsensusStrategy::Dummy,
    }
});

pub static DEV_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    ChainConfig {
        version: Version { major: 1 },
        //use latest git commit version's hash
        parent_hash: HashValue::sha3_256_of(
            hex::decode("4df939777a8560668a7bb23bf7305e62bdb116f2")
                .expect("invalid hex")
                .as_slice(),
        ),
        timestamp: 1595924170,
        total_supply: STARCOIN_TOTAL_SUPPLY,
        reward_delay: 1,
        difficulty: 1.into(),
        nonce: 0,
        pre_mine_percent: 20,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        boot_nodes: vec![],
        uncle_rate_target: 80,
        epoch_time_target: DEV_EPOCH_TIME_TARGET,
        reward_half_epoch: DEV_EPOCH_TIME_TARGET * 2 / DEV_EPOCH_TIME_TARGET,
        init_block_time_target: 5,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_time_target: DEV_MIN_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (Some(association_private_key), association_public_key),
        genesis_key_pair: Some((genesis_private_key, genesis_public_key)),
        consensus_strategy: ConsensusStrategy::Dev,
    }
});

pub static HALLEY_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    ChainConfig {
        version: Version { major: 1 },
        //use latest git commit hash
        parent_hash: HashValue::sha3_256_of(hex::decode("4df939777a8560668a7bb23bf7305e62bdb116f2").expect("invalid hex").as_slice()),
        timestamp: 1595924170,
        total_supply: STARCOIN_TOTAL_SUPPLY,
        reward_delay: 3,
        difficulty: 10.into(),
        nonce: 0,
        pre_mine_percent: 20,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        boot_nodes: vec!["/dns4/halley1.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"),
                         "/dns4/halley2.seed.starcoin.org/tcp/9840/p2p/12D3KooWAua4KokJMiCodGPEF2n4yN42B2Q26KgwrQTntnrCDRHd".parse().expect("parse multi addr should be ok"),
                         "/dns4/halley3.seed.starcoin.org/tcp/9840/p2p/12D3KooW9vHQJk9o69tZPMM2viQ3eWpgp6veDBRz8tTvDFDBejwk".parse().expect("parse multi addr should be ok"), ],
        uncle_rate_target: 80,
        epoch_time_target: EPOCH_TIME_TARGET,
        reward_half_epoch: REWARD_HALF_TIME_TARGET / EPOCH_TIME_TARGET,
        init_block_time_target: INIT_BLOCK_TIME_TARGET,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_time_target: MIN_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (None, Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
            .expect("decode public key must success.")),
        genesis_key_pair: None,
        consensus_strategy: ConsensusStrategy::Argon,
    }
});

pub static PROXIMA_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    ChainConfig {
        version: Version { major: 1 },
        //TODO set parent_hash and timestamp
        parent_hash: HashValue::zero(),
        timestamp: 0,
        total_supply: STARCOIN_TOTAL_SUPPLY,
        reward_delay: 7,
        difficulty: 10.into(),
        nonce: 0,
        pre_mine_percent: 20,
        vm_config: VMConfig {
            publishing_option: VMPublishingOption::Open,
            gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
        },
        boot_nodes: vec!["/dns4/proxima1.seed.starcoin.org/tcp/9840/p2p/12D3KooW9vHQJk9o69tZPMM2viQ3eWpgp6veDBRz8tTvDFDBejwk".parse().expect("parse multi addr should be ok"),
                         "/dns4/proxima2.seed.starcoin.org/tcp/9840/p2p/12D3KooWAua4KokJMiCodGPEF2n4yN42B2Q26KgwrQTntnrCDRHd".parse().expect("parse multi addr should be ok"),
                         "/dns4/proxima3.seed.starcoin.org/tcp/9840/p2p/12D3KooWFvCKQ1n2JkSQpn8drqGwU27vTPkKx264zD4CFbgaKDJU".parse().expect("parse multi addr should be ok"), ],
        uncle_rate_target: 80,
        epoch_time_target: EPOCH_TIME_TARGET,
        reward_half_epoch: REWARD_HALF_TIME_TARGET / EPOCH_TIME_TARGET,
        init_block_time_target: INIT_BLOCK_TIME_TARGET,
        block_difficulty_window: BLOCK_DIFF_WINDOW,
        reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
        min_time_target: MIN_TIME_TARGET,
        max_uncles_per_block: MAX_UNCLES_PER_BLOCK,
        association_key_pair: (None, Ed25519PublicKey::from_encoded_string(
            "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
        )
            .expect("decode public key must success.")),
        genesis_key_pair: None,
        consensus_strategy: ConsensusStrategy::Argon,
    }
});

pub static MAIN_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    version: Version { major: 1 },
    //TODO set parent_hash and timestamp
    parent_hash: HashValue::zero(),
    timestamp: 0,
    total_supply: STARCOIN_TOTAL_SUPPLY,
    reward_delay: 7,
    difficulty: 10.into(),
    nonce: 0,
    pre_mine_percent: 0,
    vm_config: VMConfig {
        publishing_option: VMPublishingOption::Open,
        gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
    },
    boot_nodes: vec![],
    uncle_rate_target: 80,
    epoch_time_target: EPOCH_TIME_TARGET,
    reward_half_epoch: REWARD_HALF_TIME_TARGET / EPOCH_TIME_TARGET,
    init_block_time_target: INIT_BLOCK_TIME_TARGET,
    block_difficulty_window: BLOCK_DIFF_WINDOW,
    reward_per_uncle_percent: REWARD_PER_UNCLE_PERCENT,
    min_time_target: MIN_TIME_TARGET,
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
});
