// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockHeader;
use crate::block::BlockNumber;
use crate::startup_info::{ChainInfo, ChainStatus};
use crate::U256;
use network_p2p_types::identity::PublicKey;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::HashValue;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

pub use network_p2p_types::multiaddr::Multiaddr;
use network_p2p_types::multihash::Error;
pub use network_p2p_types::multihash::Multihash;
use std::borrow::Cow;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PeerId(network_p2p_types::PeerId);

impl PeerId {
    pub fn new(peer_id: network_p2p_types::PeerId) -> Self {
        Self(peer_id)
    }

    /// Builds a `PeerId` from a public key.
    pub fn from_public_key(key: PublicKey) -> PeerId {
        Self::new(network_p2p_types::PeerId::from_public_key(key))
    }

    pub fn from_ed25519_public_key(key: Ed25519PublicKey) -> PeerId {
        let pub_key =
            network_p2p_types::identity::ed25519::PublicKey::decode(key.to_bytes().as_ref())
                .expect("Decode pubkey must success.");
        Self::from_public_key(PublicKey::Ed25519(pub_key))
    }

    /// Checks whether `data` is a valid `PeerId`. If so, returns the `PeerId`. If not, returns
    /// back the data as an error.
    pub fn from_bytes(data: Vec<u8>) -> Result<PeerId, Error> {
        Ok(Self::new(network_p2p_types::PeerId::from_bytes(&data)?))
    }

    /// Turns a `Multihash` into a `PeerId`. If the multihash doesn't use the correct algorithm,
    /// returns back the data as an error.
    pub fn from_multihash(data: Multihash) -> Result<PeerId, Multihash> {
        Ok(Self::new(network_p2p_types::PeerId::from_multihash(data)?))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.to_bytes()
    }

    /// Returns a base-58 encoded string of this `PeerId`.
    pub fn to_base58(&self) -> String {
        self.0.to_base58()
    }

    pub fn origin(&self) -> &network_p2p_types::PeerId {
        &self.0
    }

    /// Checks whether the public key passed as parameter matches the public key of this `PeerId`.
    ///
    /// Returns `None` if this `PeerId`s hash algorithm is not supported when encoding the
    /// given public key, otherwise `Some` boolean as the result of an equality check.
    pub fn is_public_key(&self, public_key: &PublicKey) -> Option<bool> {
        self.0.is_public_key(public_key)
    }

    pub fn random() -> Self {
        Self(network_p2p_types::PeerId::random())
    }
}

#[allow(clippy::from_over_into)]
impl Into<network_p2p_types::PeerId> for PeerId {
    fn into(self) -> network_p2p_types::PeerId {
        self.0
    }
}

impl From<network_p2p_types::PeerId> for PeerId {
    fn from(peer_id: network_p2p_types::PeerId) -> Self {
        Self(peer_id)
    }
}

impl TryFrom<Multihash> for PeerId {
    type Error = Multihash;

    fn try_from(value: Multihash) -> Result<Self, Self::Error> {
        PeerId::from_multihash(value)
    }
}

impl From<PeerId> for Multihash {
    fn from(peer_id: PeerId) -> Self {
        peer_id.0.into()
    }
}

impl FromStr for PeerId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(network_p2p_types::PeerId::from_str(s)?))
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            //note: if use &str at here, json rpc raise a error: invalid type: string "xx", expected a borrowed string
            let s = <String>::deserialize(deserializer)?;
            let peer_id =
                network_p2p_types::PeerId::from_str(s.as_str()).map_err(D::Error::custom)?;
            Ok(PeerId(peer_id))
        } else {
            let b = <Vec<u8>>::deserialize(deserializer)?;
            let peer_id = network_p2p_types::PeerId::from_bytes(&b)
                .map_err(|e| D::Error::custom(format_args!("parse PeerId fail:{:?}", e)))?;
            Ok(PeerId(peer_id))
        }
    }
}

impl Serialize for PeerId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_base58().serialize(serializer)
        } else {
            self.0.to_bytes().serialize(serializer)
        }
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub chain_info: ChainInfo,
    pub notif_protocols: Vec<Cow<'static, str>>,
    pub rpc_protocols: Vec<Cow<'static, str>>,
}

impl PeerInfo {
    pub fn new(
        peer_id: PeerId,
        chain_info: ChainInfo,
        notif_protocols: Vec<Cow<'static, str>>,
        rpc_protocols: Vec<Cow<'static, str>>,
    ) -> Self {
        Self {
            peer_id,
            chain_info,
            notif_protocols,
            rpc_protocols,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn chain_info(&self) -> &ChainInfo {
        &self.chain_info
    }

    pub fn block_number(&self) -> BlockNumber {
        self.chain_info.head().number()
    }

    pub fn latest_header(&self) -> &BlockHeader {
        self.chain_info.head()
    }

    pub fn block_id(&self) -> HashValue {
        self.chain_info.head().id()
    }

    pub fn total_difficulty(&self) -> U256 {
        self.chain_info.total_difficulty()
    }

    pub fn update_chain_status(&mut self, chain_status: ChainStatus) {
        self.chain_info.update_status(chain_status)
    }

    /// This peer is support notification
    pub fn is_support_notification(&self) -> bool {
        !self.notif_protocols.is_empty()
    }

    pub fn is_support_notif_protocol(&self, protocol: Cow<'static, str>) -> bool {
        self.notif_protocols.contains(&protocol)
    }

    pub fn is_support_rpc(&self) -> bool {
        !self.rpc_protocols.is_empty()
    }

    pub fn is_support_rpc_protocol(&self, protocol: Cow<'static, str>) -> bool {
        self.rpc_protocols.contains(&protocol)
    }

    pub fn random() -> Self {
        Self {
            peer_id: PeerId::random(),
            chain_info: ChainInfo::random(),
            notif_protocols: vec![],
            rpc_protocols: vec![],
        }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct RpcInfo {
    paths: Vec<String>,
}

impl RpcInfo {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    pub fn empty() -> Self {
        Self { paths: vec![] }
    }
    pub fn new(mut paths: Vec<String>) -> Self {
        paths.sort();
        paths.dedup();
        Self { paths }
    }
}

impl IntoIterator for RpcInfo {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.into_iter()
    }
}
