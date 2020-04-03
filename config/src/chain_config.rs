// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::ValidKeyStringExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
    pub fn chain_id(&self) -> u64 {
        (*self).into()
    }
    pub fn is_dev(&self) -> bool {
        match self {
            ChainNetwork::Dev => true,
            _ => false,
        }
    }
    pub fn get_config(&self) -> &ChainConfig {
        match self {
            ChainNetwork::Dev => &DEV_CHAIN_CONFIG,
            ChainNetwork::Halley => &HALLEY_CHAIN_CONFIG,
            ChainNetwork::Proxima => &PROXIMA_CHAIN_CONFIG,
            ChainNetwork::Main => &MAIN_CHAIN_CONFIG,
        }
    }
}

impl Default for ChainNetwork {
    fn default() -> Self {
        ChainNetwork::Dev
    }
}

#[derive(Clone, Debug)]
pub struct PreMineConfig {
    pub account: AccountAddress,
    pub public_key: Ed25519PublicKey,
    /// pre mine percent of total_supply, from 0~100.
    pub pre_mine_percent: u64,
}

/// ChainConfig is a static hard code config.
#[derive(Clone, Debug)]
pub struct ChainConfig {
    /// Starcoin total supply.
    pub total_supply: u64,
    /// Base reward for every Block miner.
    pub base_block_reward: u64,
    /// Halving reward for how many block mined.
    pub reward_halving_interval: u64,
    /// How many block to delay before rewarding miners.
    pub reward_delay: u64,
    /// Pre mine accounts
    pub pre_mine_accounts: Vec<PreMineConfig>,
}

static DEV_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    total_supply: 2_100_000_000 * 1000_000,
    base_block_reward: 5000 * 1000_000,
    reward_halving_interval: 100,
    reward_delay: 1,
    /// Dev faucet is set when node start.
    pre_mine_accounts: vec![],
});

static HALLEY_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    total_supply: 2_100_000_000 * 1000_000,
    base_block_reward: 5000 * 1000_000,
    reward_halving_interval: 1000,
    reward_delay: 3,
    pre_mine_accounts: vec![
        /// halley test net faucet account.
        PreMineConfig {
            account: account_config::association_address(),
            public_key: Ed25519PublicKey::from_encoded_string(
                "025fbcc063f74edb4909fd8fb5f2fa3ed92748141fefc5eda29e425d98a95505",
            )
            .unwrap(),
            pre_mine_percent: 20,
        },
    ],
});

static PROXIMA_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    total_supply: 2_100_000_000 * 1000_000,
    base_block_reward: 5000 * 1000_000,
    reward_halving_interval: 10000,
    reward_delay: 7,
    pre_mine_accounts: vec![],
});

static MAIN_CHAIN_CONFIG: Lazy<ChainConfig> = Lazy::new(|| ChainConfig {
    total_supply: 2_100_000_000 * 1000_000,
    base_block_reward: 5000 * 1000_000,
    reward_halving_interval: 52500,
    reward_delay: 7,
    pre_mine_accounts: vec![],
});
