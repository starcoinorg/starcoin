// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_available_port;
use anyhow::{ensure, Result};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::test_utils::KeyPair;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use types::account_address::{AccountAddress, AuthenticationKey};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub dev_mode: bool,
    pub stratum_server: SocketAddr,
    //TODO refactor miner address config.
    #[serde(skip)]
    miner_account: (AccountAddress, AuthenticationKey),
    pub pacemaker_strategy: PacemakerStrategy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum PacemakerStrategy {
    HeadBlock,
    Ondemand,
    Schedule,
}

impl Default for MinerConfig {
    fn default() -> Self {
        let auth_key = AuthenticationKey::random();
        Self {
            dev_mode: false,
            stratum_server: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
            miner_account: (auth_key.derived_address(), auth_key),
        }
    }
}

impl MinerConfig {
    pub fn random_for_test() -> Self {
        let auth_key = AuthenticationKey::random();
        Self {
            dev_mode: true,
            stratum_server: format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
            miner_account: (auth_key.derived_address(), auth_key),
        }
    }

    pub fn load(&mut self, _data_dir: &PathBuf) -> Result<()> {
        Ok(())
    }

    pub fn account_address(&self) -> AccountAddress {
        self.miner_account.0
    }

    pub fn auth_key(&self) -> Vec<u8> {
        self.miner_account.1.prefix().to_vec()
    }

    pub fn set_default_account(&mut self, account: (AccountAddress, AuthenticationKey)) {
        self.miner_account = account;
    }
}
