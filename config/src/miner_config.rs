// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::get_available_port;
use anyhow::{ensure, Result};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::test_utils::KeyPair;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use types::account_address::{AccountAddress, AuthenticationKey};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinerConfig {
    pub dev_mode: bool,
    pub stratum_server: SocketAddr,
    pub pacemaker_strategy: PacemakerStrategy,
    mint_key_file: PathBuf,
    #[serde(skip)]
    pub mint_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
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
        Self {
            dev_mode: false,
            stratum_server: "127.0.0.1:9000".parse::<SocketAddr>().unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
            mint_key_file: PathBuf::from("mint_key"),
            mint_keypair: None,
        }
    }
}

impl MinerConfig {
    pub fn random_for_test() -> Self {
        Self {
            dev_mode: true,
            stratum_server: format!("127.0.0.1:{}", get_available_port())
                .parse::<SocketAddr>()
                .unwrap(),
            pacemaker_strategy: PacemakerStrategy::Schedule,
            mint_key_file: PathBuf::from("mint_key"),
            mint_keypair: Some(crate::gen_keypair()),
        }
    }

    pub fn load(&mut self, data_dir: &PathBuf) -> Result<()> {
        ensure!(
            self.mint_key_file.is_relative(),
            "mint key file should be relative path"
        );
        ensure!(
            !self.mint_key_file.as_os_str().is_empty(),
            "mint key file should not be empty path"
        );
        let path = data_dir.join(&self.mint_key_file);
        let keypair = if path.exists() {
            // load from file directly

            let mint_keypair = crate::load_key(&path)?;
            Arc::new(mint_keypair)
        } else {
            // generate key and save it
            let keypair = crate::gen_keypair();
            crate::save_key(&keypair.private_key.to_bytes(), &path)?;
            keypair
        };

        self.mint_keypair = Some(keypair);

        Ok(())
    }

    pub fn account_address(&self) -> AccountAddress {
        AccountAddress::from_public_key(&self.mint_keypair.clone().take().unwrap().public_key)
    }

    pub fn auth_key(&self) -> Vec<u8> {
        AuthenticationKey::from_public_key(&self.mint_keypair.clone().take().unwrap().public_key)
            .prefix()
            .to_vec()
    }
}
