// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::test_utils::KeyPair;

use logger::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt};
use starcoin_crypto::HashValue;
use starcoin_types::peer_info::PeerId;
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
    pub network_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
    pub genesis_hash: Option<HashValue>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::default_with_net(ChainNetwork::default())
    }
}

impl NetworkConfig {
    pub fn network_keypair(&self) -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        self.network_keypair.clone().unwrap()
    }
}

impl ConfigModule for NetworkConfig {
    fn default_with_net(_net: ChainNetwork) -> Self {
        Self {
            listen: "/ip4/0.0.0.0/tcp/9840".to_string(),
            seeds: vec![],
            network_key_file: PathBuf::from("network_key"),
            network_keypair: None,
            genesis_hash: None,
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        let keypair = crate::gen_keypair();
        self.network_keypair = Some(keypair);
    }

    fn load(&mut self, base: &BaseConfig, opt: &StarcoinOpt) -> Result<()> {
        ensure!(
            self.network_key_file.is_relative(),
            "network key file should be relative path"
        );
        ensure!(
            !self.network_key_file.as_os_str().is_empty(),
            "network key file should not be empty path"
        );
        if let Some(seeds) = &opt.seeds {
            self.seeds.extend_from_slice(seeds.as_slice());
            info!(
                "Update seeds config from command line opt, seeds: {:?}",
                self.seeds
            );
        }
        let data_dir = base.data_dir();
        let path = data_dir.join(&self.network_key_file);
        let keypair = if path.exists() {
            // load from file directly
            let network_keypair = crate::load_key(&path)?;
            Arc::new(network_keypair)
        } else {
            // generate key and save it
            let keypair = crate::gen_keypair();
            let peer_id = PeerId::from_ed25519_public_key(keypair.public_key.clone());
            info!("peer_id is : {:?}", peer_id.to_base58());
            crate::save_key(&keypair.private_key.to_bytes(), &path)?;
            keypair
        };

        self.network_keypair = Some(keypair);

        Ok(())
    }
}
