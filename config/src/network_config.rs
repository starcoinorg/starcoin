// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    decode_key, get_available_port_from, get_random_available_port, load_key, parse_key_val,
    ApiQuotaConfig, BaseConfig, ConfigModule, QuotaDuration, StarcoinOpt,
};
use anyhow::{bail, Result};
use network_p2p_types::{
    is_memory_addr, memory_addr,
    multiaddr::{Multiaddr, Protocol},
    MultiaddrWithPeerId,
};
use serde::{Deserialize, Serialize};
use starcoin_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::PeerId;
use std::net::Ipv4Addr;
use std::num::NonZeroU32;
use std::sync::Arc;
use structopt::StructOpt;

pub static DEFAULT_NETWORK_PORT: u16 = 9840;
static NETWORK_KEY_FILE: &str = "network_key";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct NetworkRpcQuotaConfiguration {
    #[structopt(
        name = "default-global-p2prpc-quota",
        long,
        help = "default global p2p rpc quota, eg: 1000/s",
        default_value = "1000/s"
    )]
    pub default_global_api_quota: ApiQuotaConfig,

    // number_of_values = 1 forces the user to repeat the -D option for each key-value pair:
    // my_program -D a=1 -D b=2
    #[structopt(
        name = "custom-global-p2prpc-quota",
        long,
        help = "customize global p2p rpc quota, eg: get_block=100/s",
        number_of_values = 1,
        parse(try_from_str = parse_key_val)
    )]
    pub custom_global_api_quota: Vec<(String, ApiQuotaConfig)>,

    #[structopt(
        name = "default-user-p2prpc-quota",
        long,
        help = "default p2p rpc quota of a peer, eg: 1000/s",
        default_value = "1000/s"
    )]
    pub default_user_api_quota: ApiQuotaConfig,

    #[structopt(
        name = "custom-user-p2prpc-quota",
        long,
        help = "customize p2p rpc quota of a peer, eg: get_block=10/s",
        parse(try_from_str = parse_key_val),
        number_of_values = 1
    )]
    pub custom_user_api_quota: Vec<(String, ApiQuotaConfig)>,
}

impl Default for NetworkRpcQuotaConfiguration {
    fn default() -> Self {
        Self {
            default_global_api_quota: ApiQuotaConfig {
                max_burst: NonZeroU32::new(1000).unwrap(),
                duration: QuotaDuration::Second,
            },
            custom_global_api_quota: vec![],
            default_user_api_quota: ApiQuotaConfig {
                max_burst: NonZeroU32::new(50).unwrap(),
                duration: QuotaDuration::Second,
            },
            custom_user_api_quota: vec![],
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    // The address that this node is listening on for new connections.
    pub listen: Multiaddr,
    pub seeds: Vec<MultiaddrWithPeerId>,
    pub enable_mdns: bool,
    //TODO skip this field, do not persistence this flag to config. this change will break network config.
    pub disable_seed: bool,
    #[serde(skip)]
    network_keypair: Option<Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>>,
    #[serde(skip)]
    self_peer_id: Option<PeerId>,
    #[serde(skip)]
    self_address: Option<MultiaddrWithPeerId>,
    #[serde(default)]
    pub network_rpc_quotas: NetworkRpcQuotaConfiguration,
}

impl NetworkConfig {
    pub fn network_keypair(&self) -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        self.network_keypair.clone().expect("Config should init.")
    }

    pub fn self_address(&self) -> MultiaddrWithPeerId {
        self.self_address.clone().expect("Config should init.")
    }

    pub fn self_peer_id(&self) -> PeerId {
        self.self_peer_id.clone().expect("Config should init.")
    }

    fn prepare_peer_id(&mut self) {
        let peer_id = PeerId::from_ed25519_public_key(self.network_keypair().public_key.clone());
        let host = if is_memory_addr(&self.listen) {
            self.listen.clone()
        } else {
            self.listen
                .clone()
                .replace(0, |_p| Some(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1))))
                .expect("Replace multi address fail.")
        };
        self.self_address = Some(MultiaddrWithPeerId::new(host, peer_id.clone().into()));
        self.self_peer_id = Some(peer_id);
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

        let port = if base.net.is_test() {
            get_random_available_port()
        } else if base.net.is_dev() {
            get_available_port_from(DEFAULT_NETWORK_PORT)
        } else {
            DEFAULT_NETWORK_PORT
        };
        //test env use in memory transport.
        let listen = if base.net.is_test() {
            memory_addr(port as u64)
        } else {
            format!("/ip4/0.0.0.0/tcp/{}", port)
                .parse()
                .expect("Parse multi address fail.")
        };
        Ok(Self {
            listen,
            seeds,
            enable_mdns: opt.enable_mdns,
            disable_seed: opt.disable_seed,
            network_keypair: Some(Arc::new(Self::load_or_generate_keypair(opt, base)?)),
            self_peer_id: None,
            self_address: None,
            network_rpc_quotas: opt.network_rpc_quotas.clone(),
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

        self.network_keypair = Some(Arc::new(Self::load_or_generate_keypair(opt, base)?));
        self.disable_seed = opt.disable_seed;
        self.prepare_peer_id();

        Ok(())
    }
}
