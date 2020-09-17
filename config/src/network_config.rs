// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    decode_key, get_available_port_from, get_random_available_port, load_key, BaseConfig,
    ConfigModule, StarcoinOpt,
};
use anyhow::{bail, format_err, Result};
use libp2p::multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::PeerId;
use std::net::Ipv4Addr;
use std::sync::Arc;

pub static DEFAULT_NETWORK_PORT: u16 = 9840;
static NETWORK_KEY_FILE: &str = "network_key";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    // The address that this node is listening on for new connections.
    pub listen: Multiaddr,
    pub seeds: Vec<Multiaddr>,
    pub disable_seed: bool,
    #[serde(skip)]
    pub network_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
    #[serde(skip)]
    pub self_peer_id: Option<PeerId>,
    #[serde(skip)]
    pub self_address: Option<Multiaddr>,
}

impl NetworkConfig {
    pub fn network_keypair(&self) -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        self.network_keypair.clone().unwrap()
    }

    pub fn self_address(&self) -> Result<Multiaddr> {
        self.self_address
            .as_ref()
            .cloned()
            .ok_or_else(|| format_err!("Config not init."))
    }

    pub fn self_peer_id(&self) -> Result<PeerId> {
        self.self_peer_id
            .clone()
            .ok_or_else(|| format_err!("Self peer_id has not init."))
    }

    fn prepare_peer_id(&mut self) {
        let peer_id = PeerId::from_ed25519_public_key(self.network_keypair().public_key.clone());
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

    fn load_or_generate_keypair(
        opt: &StarcoinOpt,
        base: &BaseConfig,
    ) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        let data_dir = base.data_dir();
        let path = data_dir.join(NETWORK_KEY_FILE);
        if path.exists() {
            load_key(&path)
        } else {
            let keypair = match (&opt.node_key, &opt.node_key_file) {
                (Some(_), Some(_)) => bail!("Only one of node-key and node-key-file can be set."),
                (Some(node_key), None) => decode_key(node_key)?,
                (None, Some(node_key_file)) => load_key(node_key_file)?,
                (None, None) => crate::gen_keypair(),
            };
            crate::save_key(&keypair.private_key.to_bytes(), &path)?;
            Ok(keypair)
        }
    }
}

impl ConfigModule for NetworkConfig {
    fn default_with_opt(opt: &StarcoinOpt, base: &BaseConfig) -> Result<Self> {
        let seeds = opt
            .seed
            .as_ref()
            .map(|seed| vec![seed.clone()])
            .unwrap_or_default();
        for seed in &seeds {
            Self::check_seed(seed)?;
        }
        let port = if base.net.is_test() {
            get_random_available_port()
        } else if base.net.is_dev() {
            get_available_port_from(DEFAULT_NETWORK_PORT)
        } else {
            DEFAULT_NETWORK_PORT
        };
        Ok(Self {
            listen: format!("/ip4/0.0.0.0/tcp/{}", port)
                .parse()
                .expect("Parse multi address fail."),
            seeds,
            network_keypair: Some(Arc::new(Self::load_or_generate_keypair(opt, base)?)),
            self_peer_id: None,
            self_address: None,
            disable_seed: opt.disable_seed,
        })
    }

    fn after_load(&mut self, opt: &StarcoinOpt, base: &BaseConfig) -> Result<()> {
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
        self.network_keypair = Some(Arc::new(Self::load_or_generate_keypair(opt, base)?));
        self.disable_seed = opt.disable_seed;
        self.prepare_peer_id();

        Ok(())
    }
}
