// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::ensure;
use anyhow::Result;
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::{test_utils::KeyPair, Uniform};
use libp2p::multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    // The address that this node is listening on for new connections.
    pub listen: String,
    pub seeds: Vec<String>,
    network_key_file: PathBuf,
    #[serde(skip)]
    network_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen: "/ip4/0.0.0.0/tcp/9840".to_string(),
            seeds: vec![],
            network_key_file: PathBuf::from("network_key"),
            network_keypair: None,
        }
    }
}

impl NetworkConfig {
    pub fn random_for_test() -> Self {
        let mut default_config = Self::default();
        let keypair = crate::gen_keypair();
        default_config.network_keypair = Some(keypair);
        default_config
    }

    pub fn load(&mut self, data_dir: &PathBuf) -> Result<()> {
        ensure!(
            self.network_key_file.is_relative(),
            "network key file should be relative path"
        );
        ensure!(
            !self.network_key_file.as_os_str().is_empty(),
            "network key file should not be empty path"
        );
        let path = data_dir.join(&self.network_key_file);
        let keypair = if path.exists() {
            // load from file directly

            let network_keypair = crate::load_key(&path)?;
            Arc::new(network_keypair)
        } else {
            // generate key and save it
            let keypair = crate::gen_keypair();
            crate::save_key(&keypair.private_key.to_bytes(), &path)?;
            keypair
        };

        self.network_keypair = Some(keypair);

        Ok(())
    }

    pub fn network_keypair(&self) -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        self.network_keypair.clone().unwrap()
    }
}
