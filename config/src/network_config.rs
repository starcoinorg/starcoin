// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::{decode_key, gen_keypair, generate_node_name, load_key, save_key};
use crate::{
    get_available_port_from, get_random_available_port, parse_key_val, ApiQuotaConfig, BaseConfig,
    ConfigModule, QuotaDuration, StarcoinOpt,
};
use anyhow::Result;
use network_p2p_types::{
    is_memory_addr, memory_addr,
    multiaddr::{Multiaddr, Protocol},
    MultiaddrWithPeerId,
};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_logger::prelude::*;
use starcoin_types::peer_info::PeerId;
use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

pub static DEFAULT_NETWORK_PORT: u16 = 9840;
static NETWORK_KEY_FILE: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("network_key"));

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, StructOpt)]
pub struct NetworkRpcQuotaConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "p2prpc-default-global-api-quota",
        long,
        help = "default global p2p rpc quota, eg: 1000/s"
    )]
    pub default_global_api_quota: Option<ApiQuotaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "p2prpc-custom-global-api-quota",
        long,
        number_of_values = 1,
        parse(try_from_str = parse_key_val)
    )]
    /// customize global p2p rpc quota, eg: get_block=100/s
    /// number_of_values = 1 forces the user to repeat the -D option for each key-value pair:
    /// my_program -D a=1 -D b=2
    pub custom_global_api_quota: Option<Vec<(String, ApiQuotaConfig)>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "p2prpc-default-user-api-quota",
        long,
        help = "default p2p rpc quota of a peer, eg: 1000/s"
    )]
    pub default_user_api_quota: Option<ApiQuotaConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(
        name = "p2prpc-custom-user-api-quota",
        long,
        help = "customize p2p rpc quota of a peer, eg: get_block=10/s",
        parse(try_from_str = parse_key_val),
        number_of_values = 1
    )]
    pub custom_user_api_quota: Option<Vec<(String, ApiQuotaConfig)>>,
}

impl NetworkRpcQuotaConfiguration {
    pub fn default_global_api_quota(&self) -> ApiQuotaConfig {
        self.default_global_api_quota
            .clone()
            .unwrap_or(ApiQuotaConfig {
                max_burst: NonZeroU32::new(1000).unwrap(),
                duration: QuotaDuration::Second,
            })
    }

    pub fn custom_global_api_quota(&self) -> Vec<(String, ApiQuotaConfig)> {
        self.custom_global_api_quota.clone().unwrap_or_default()
    }

    pub fn default_user_api_quota(&self) -> ApiQuotaConfig {
        self.default_user_api_quota
            .clone()
            .unwrap_or(ApiQuotaConfig {
                max_burst: NonZeroU32::new(50).unwrap(),
                duration: QuotaDuration::Second,
            })
    }

    pub fn custom_user_api_quota(&self) -> Vec<(String, ApiQuotaConfig)> {
        self.custom_user_api_quota.clone().unwrap_or_default()
    }

    pub fn merge(&mut self, o: &Self) -> Result<()> {
        if o.default_global_api_quota.is_some() {
            self.default_global_api_quota = o.default_global_api_quota.clone();
        }
        //TODO should merge two vec?
        if o.custom_global_api_quota.is_some() {
            self.custom_global_api_quota = o.custom_global_api_quota.clone();
        }
        if o.default_user_api_quota.is_some() {
            self.default_user_api_quota = o.default_user_api_quota.clone();
        }
        if o.custom_user_api_quota.is_some() {
            self.custom_user_api_quota = o.custom_user_api_quota.clone();
        }
        Ok(())
    }
}
//for avoid conflict between seed vec and subcommand, so define a custom type to parse seeds.
//https://github.com/TeXitoi/structopt/issues/367
#[derive(Default, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Seeds(pub Vec<MultiaddrWithPeerId>);
impl Seeds {
    pub fn into_vec(self) -> Vec<MultiaddrWithPeerId> {
        self.into()
    }
    pub fn merge(&mut self, other: &Seeds) {
        let mut seeds = HashSet::new();
        seeds.extend(self.0.clone().into_iter());
        seeds.extend(other.0.clone().into_iter());
        let mut seeds: Vec<MultiaddrWithPeerId> = seeds.into_iter().collect();
        //keep order in config
        seeds.sort();
        self.0 = seeds;
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl FromStr for Seeds {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seeds = s
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| MultiaddrWithPeerId::from_str(s))
            .collect::<Result<Vec<MultiaddrWithPeerId>, network_p2p_types::ParseErr>>()?;
        Ok(Seeds(seeds))
    }
}
impl Into<Vec<MultiaddrWithPeerId>> for Seeds {
    fn into(self) -> Vec<MultiaddrWithPeerId> {
        self.0
    }
}
impl From<Vec<MultiaddrWithPeerId>> for Seeds {
    fn from(seeds: Vec<MultiaddrWithPeerId>) -> Self {
        Seeds(seeds)
    }
}
#[derive(Default, Clone, Debug, Deserialize, PartialEq, Serialize, StructOpt)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "node-name")]
    /// Node network name, just for display, if absent will generate a random name.
    pub node_name: Option<String>,

    #[serde(skip)]
    #[structopt(long = "node-key")]
    /// Node network private key string
    /// This option is skip for config file, only support cli option, after init will write the key to node_key_file
    pub node_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long = "node-key-file", parse(from_os_str), conflicts_with("node-key"))]
    /// Node network private key file, default is network_key under the data dir.
    pub node_key_file: Option<PathBuf>,

    #[serde(skip_serializing_if = "Seeds::is_empty")]
    #[serde(default)]
    #[structopt(long = "seed", default_value = "")]
    /// P2P network seed, multi seed should use ',' as delimiter.
    pub seeds: Seeds,

    #[serde(skip)]
    #[structopt(long = "disable-mdns")]
    /// Disable p2p mdns discovery, for automatically discover the peer from the local network.
    /// disable_mdns is true in default.
    pub disable_mdns: Option<bool>,

    #[serde(skip)]
    #[structopt(long = "disable-seed")]
    /// Do not connect to seed node, include builtin and config seed.
    /// This option is skip for config file, only support cli option.
    pub disable_seed: bool,

    #[structopt(flatten)]
    pub network_rpc_quotas: NetworkRpcQuotaConfiguration,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    /// min peers to propagate new block and new transactions. Default to 8.
    min_peers_to_propagate: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    ///max peers to propagate new block and new transactions. Default to 128.
    max_peers_to_propagate: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    /// p2p network listen address, Default is /ip4/0.0.0.0/tcp/9840
    listen: Option<Multiaddr>,

    #[serde(skip)]
    #[structopt(skip)]
    base: Option<Arc<BaseConfig>>,

    #[serde(skip)]
    #[structopt(skip)]
    network_keypair: Option<(Ed25519PrivateKey, Ed25519PublicKey)>,

    #[serde(skip)]
    #[structopt(skip)]
    generate_listen: Option<Multiaddr>,
}

impl NetworkConfig {
    fn base(&self) -> &BaseConfig {
        self.base.as_ref().expect("Config should init.")
    }

    pub fn listen(&self) -> Multiaddr {
        self.generate_listen.clone().expect("Config should init.")
    }

    pub fn seeds(&self) -> Vec<MultiaddrWithPeerId> {
        if self.disable_seed {
            return vec![];
        }
        let mut seeds: HashSet<MultiaddrWithPeerId> =
            self.seeds.clone().into_vec().into_iter().collect();
        seeds.extend(self.base().net().boot_nodes().iter().cloned());

        let self_peer_id = self.self_peer_id();
        seeds.retain(|node| {
            if &node.peer_id == self_peer_id.origin() {
                info!(
                    "Self peer_id({}) contains in boot nodes, removed.",
                    self_peer_id
                );
                false
            } else {
                true
            }
        });
        let mut seeds: Vec<MultiaddrWithPeerId> = seeds.into_iter().collect();
        // shuffle seeds, connect seeds with random orders.
        seeds.shuffle(&mut thread_rng());
        seeds
    }

    pub fn network_keypair(&self) -> &(Ed25519PrivateKey, Ed25519PublicKey) {
        self.network_keypair.as_ref().expect("Config should init.")
    }

    pub fn self_address(&self) -> MultiaddrWithPeerId {
        let addr = self.listen();
        let host = if is_memory_addr(&addr) {
            addr
        } else {
            addr.replace(0, |_p| Some(Protocol::Ip4(Ipv4Addr::new(127, 0, 0, 1))))
                .expect("Replace multi address fail.")
        };
        MultiaddrWithPeerId::new(host, self.self_peer_id().into())
    }

    pub fn disable_mdns(&self) -> bool {
        // mdns is disable by default.
        self.disable_mdns.unwrap_or(true)
    }
    pub fn disable_seed(&self) -> bool {
        self.disable_seed
    }

    pub fn self_peer_id(&self) -> PeerId {
        PeerId::from_ed25519_public_key(self.network_keypair().1.clone())
    }

    pub fn max_peers_to_propagate(&self) -> u32 {
        self.max_peers_to_propagate.clone().unwrap_or(128)
    }

    pub fn min_peers_to_propagate(&self) -> u32 {
        self.min_peers_to_propagate.clone().unwrap_or(8)
    }

    pub fn node_name(&self) -> String {
        self.node_name.clone().unwrap_or_else(generate_node_name)
    }

    fn node_key_file(&self) -> PathBuf {
        let path = self.node_key_file.as_ref().unwrap_or(&NETWORK_KEY_FILE);
        if path.is_absolute() {
            path.clone()
        } else {
            self.base().data_dir().join(path.as_path())
        }
    }

    /// node key loader step:
    /// 1. if node_key is Some, directly decode the key.
    /// 2. try load node key from node_key_file
    /// 3. if node_key_file is not exists, generate and save key to the node_key_file.
    fn load_or_generate_keypair(&mut self) -> Result<()> {
        let keypair = match self.node_key.as_ref() {
            Some(node_key) => decode_key(node_key)?,
            None => {
                let path = self.node_key_file();
                if path.exists() {
                    load_key(&path)?
                } else {
                    let keypair = gen_keypair();
                    save_key(&keypair.0.to_bytes(), &path)?;
                    keypair
                }
            }
        };
        self.network_keypair = Some(keypair);
        Ok(())
    }

    fn generate_listen_address(&mut self) -> Result<()> {
        if self.listen.is_some() {
            self.generate_listen = self.listen.clone();
        } else {
            let base = self.base();
            let port = if base.net().is_test() {
                get_random_available_port()
            } else if base.net().is_dev() {
                get_available_port_from(DEFAULT_NETWORK_PORT)
            } else {
                DEFAULT_NETWORK_PORT
            };

            //test env use in memory transport.
            let listen = if base.net().is_test() {
                memory_addr(port as u64)
            } else {
                format!("/ip4/0.0.0.0/tcp/{}", port)
                    .parse()
                    .expect("Parse multi address fail.")
            };
            self.generate_listen = Some(listen);
        }
        Ok(())
    }
}

impl ConfigModule for NetworkConfig {
    fn merge_with_opt(&mut self, opt: &StarcoinOpt, base: Arc<BaseConfig>) -> Result<()> {
        self.base = Some(base);

        self.seeds.merge(&opt.network.seeds);

        if opt.network.disable_seed {
            self.disable_seed = opt.network.disable_seed;
        }

        info!(
            "Final bootstrap seeds: {:?}, disable_seed: {}",
            self.seeds, self.disable_seed
        );

        self.network_rpc_quotas
            .merge(&opt.network.network_rpc_quotas)?;

        if opt.network.node_name.is_some() {
            self.node_name = opt.network.node_name.clone();
        }

        if self.node_name.is_none() {
            self.node_name = Some(generate_node_name())
        }

        if opt.network.node_key.is_some() {
            self.node_key = opt.network.node_key.clone();
        }

        if opt.network.listen.is_some() {
            self.listen = opt.network.listen.clone();
        }
        if let Some(m) = opt.network.max_peers_to_propagate {
            self.max_peers_to_propagate = Some(m);
        }
        if let Some(m) = opt.network.min_peers_to_propagate {
            self.min_peers_to_propagate = Some(m);
        }
        if opt.network.disable_mdns.is_some() {
            self.disable_mdns = opt.network.disable_mdns;
        }

        self.load_or_generate_keypair()?;
        self.generate_listen_address()?;
        Ok(())
    }
}
