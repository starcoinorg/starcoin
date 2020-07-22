// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use libp2p::Multiaddr;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{ed25519::*, Genesis, HashValue, PrivateKey, ValidCryptoMaterialStringExt};
use starcoin_types::{
    transaction::{RawUserTransaction, SignedUserTransaction},
    U256,
};
use starcoin_vm_types::on_chain_config::{
    VMConfig, VMPublishingOption, Version, INITIAL_GAS_SCHEDULE,
};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
#[repr(u64)]
#[serde(tag = "net")]
pub enum ChainNetwork {
    /// A ephemeral network just for developer test.
    Dev = 1024,
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
            "dev" => Ok(ChainNetwork::Dev),
            "halley" => Ok(ChainNetwork::Halley),
            "proxima" => Ok(ChainNetwork::Proxima),
            _ => Err(format_err!("")),
        }
    }
}

impl ChainNetwork {
    pub fn chain_id(self) -> u64 {
        self.into()
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
            ChainNetwork::Dev => &DEV_CHAIN_CONFIG,
            ChainNetwork::Halley => &HALLEY_CHAIN_CONFIG,
            ChainNetwork::Proxima => &PROXIMA_CHAIN_CONFIG,
            ChainNetwork::Main => &MAIN_CHAIN_CONFIG,
        }
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

pub static DEV_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    let (association_private_key, association_public_key) = genesis_key_pair();
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();

    ChainConfig {
        version: Version { major: 1 },
        parent_hash: HashValue::zero(),
        timestamp: 0,
        total_supply: STARCOIN_TOTAL_SUPPLY, //420_000_000 * 1_000_000 + 1_680_000_000 * 1_000_000 // 840_000_000 * 1_000_000 * 2 = (70_000_000 * 1_000_000 * 12) * 2 = (840_000_000 * 1_000_000 / 116) * 2
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
    }
});

pub static HALLEY_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    ChainConfig {
        version: Version { major: 1 },
        parent_hash: HashValue::zero(),
        timestamp: 0,
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
    }
});

pub static PROXIMA_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| {
    ChainConfig {
        version: Version { major: 1 },
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
    }
});

pub static MAIN_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    version: Version { major: 1 },
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
});
