// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    decode_key, get_available_port, load_key, BaseConfig, ChainNetwork, ConfigModule, StarcoinOpt,
};
use anyhow::{bail, ensure, Result};
use libp2p::multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::PeerId;
use starcoin_types::{BLOCK_PROTOCOL_NAME, CHAIN_PROTOCOL_NAME, TXN_PROTOCOL_NAME};
use std::borrow::Cow;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;

pub static DEFAULT_NETWORK_PORT: u16 = 9840;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    // The address that this node is listening on for new connections.
    pub listen: Multiaddr,
    pub seeds: Vec<Multiaddr>,
    network_key_file: PathBuf,
    #[serde(skip)]
    pub network_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
    #[serde(skip)]
    pub self_peer_id: Option<PeerId>,
    #[serde(skip)]
    pub self_address: Option<Multiaddr>,
    pub disable_seed: bool,
    #[serde(skip)]
    pub protocols: Vec<Cow<'static, [u8]>>,
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

    fn set_peer_id(&mut self) {
        let peer_id = PeerId::from_ed25519_public_key(self.network_keypair().public_key.clone());
        //TODO use a more robust method to get local best advertise ip
        let host = self
            .listen
            .clone()
            .replace(0, |_p| Some(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1))))
            .expect("Replace multi address fail.");
        let mut p2p_address = host;
        p2p_address.push(Protocol::P2p(peer_id.clone().into()));
        self.self_address = Some(p2p_address);
        self.self_peer_id = Some(peer_id);
    }

    fn check_seed(seed: &Multiaddr) -> Result<()> {
        if let Some(Protocol::P2p(_peer_id)) = seed.clone().pop() {
            return Ok(());
        }
        bail!(
            "Invalid seed {:?}, seed addr last part must is p2p/peer_id ",
            seed
        )
    }
}

impl ConfigModule for NetworkConfig {
    fn default_with_net(net: ChainNetwork) -> Self {
        let port = match net {
            ChainNetwork::Dev => get_available_port(),
            _ => DEFAULT_NETWORK_PORT,
        };
        Self {
            listen: format!("/ip4/0.0.0.0/tcp/{}", port)
                .parse()
                .expect("Parse multi address fail."),
            seeds: vec![],
            network_key_file: PathBuf::from("network_key"),
            network_keypair: None,
            self_peer_id: None,
            self_address: None,
            disable_seed: false,
            protocols: vec![
                CHAIN_PROTOCOL_NAME.into(),
                TXN_PROTOCOL_NAME.into(),
                BLOCK_PROTOCOL_NAME.into(),
            ],
        }
    }

    fn random(&mut self, _base: &BaseConfig) {
        let keypair = crate::gen_keypair();
        self.network_keypair = Some(Arc::new(keypair));
        self.set_peer_id();
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
        if let Some(opt_seed) = &opt.seed {
            if self.seeds.contains(opt_seed) {
                warn!(
                    "Command line option seed {:?} has contains in config file.",
                    opt_seed
                );
            } else {
                self.seeds.push(opt_seed.clone());
                debug!("Add command line option seed {:?} to config", self.seeds);
            }
            info!("Final bootstrap seeds: {:?}", self.seeds);
        }
        for seed in &self.seeds {
            Self::check_seed(seed)?;
        }
        let data_dir = base.data_dir();
        let path = data_dir.join(&self.network_key_file);
        let keypair = if path.exists() {
            load_key(&path)?
        } else {
            let keypair = match (&opt.node_key, &opt.node_key_file) {
                (Some(_), Some(_)) => bail!("Only one of node-key and node-key-file can be set."),
                (Some(node_key), None) => decode_key(node_key)?,
                (None, Some(node_key_file)) => load_key(node_key_file)?,
                (None, None) => crate::gen_keypair(),
            };
            crate::save_key(&keypair.private_key.to_bytes(), &path)?;
            keypair
        };

        self.network_keypair = Some(Arc::new(keypair));
        self.set_peer_id();

        self.disable_seed = opt.disable_seed;

        Ok(())
    }
}
