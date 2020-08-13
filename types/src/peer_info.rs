// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libp2p::identity::PublicKey;
use libp2p::multihash;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::ed25519::Ed25519PublicKey;

use crate::block::BlockHeader;
use crate::{block::BlockNumber, U256};
use starcoin_crypto::HashValue;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PeerId(libp2p::PeerId);

impl PeerId {
    pub fn new(peer_id: libp2p::PeerId) -> Self {
        Self(peer_id)
    }

    /// Builds a `PeerId` from a public key.
    pub fn from_public_key(key: PublicKey) -> PeerId {
        Self::new(libp2p::PeerId::from_public_key(key))
    }

    pub fn from_ed25519_public_key(key: Ed25519PublicKey) -> PeerId {
        let pub_key = libp2p::identity::ed25519::PublicKey::decode(key.to_bytes().as_ref())
            .expect("Decode pubkey must success.");
        Self::from_public_key(PublicKey::Ed25519(pub_key))
    }

    /// Checks whether `data` is a valid `PeerId`. If so, returns the `PeerId`. If not, returns
    /// back the data as an error.
    pub fn from_bytes(data: Vec<u8>) -> Result<PeerId, Vec<u8>> {
        Ok(Self::new(libp2p::PeerId::from_bytes(data)?))
    }

    /// Turns a `Multihash` into a `PeerId`. If the multihash doesn't use the correct algorithm,
    /// returns back the data as an error.
    pub fn from_multihash(data: multihash::Multihash) -> Result<PeerId, multihash::Multihash> {
        Ok(Self::new(libp2p::PeerId::from_multihash(data)?))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns a base-58 encoded string of this `PeerId`.
    pub fn to_base58(&self) -> String {
        self.0.to_base58()
    }

    /// Checks whether the public key passed as parameter matches the public key of this `PeerId`.
    ///
    /// Returns `None` if this `PeerId`s hash algorithm is not supported when encoding the
    /// given public key, otherwise `Some` boolean as the result of an equality check.
    pub fn is_public_key(&self, public_key: &PublicKey) -> Option<bool> {
        self.0.is_public_key(public_key)
    }

    pub fn random() -> Self {
        Self(libp2p::PeerId::random())
    }
}

impl Into<libp2p::PeerId> for PeerId {
    fn into(self) -> libp2p::PeerId {
        self.0
    }
}

impl From<libp2p::PeerId> for PeerId {
    fn from(peer_id: libp2p::PeerId) -> Self {
        Self(peer_id)
    }
}

impl std::convert::AsRef<[u8]> for PeerId {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl TryFrom<multihash::Multihash> for PeerId {
    type Error = multihash::Multihash;

    fn try_from(value: multihash::Multihash) -> Result<Self, Self::Error> {
        PeerId::from_multihash(value)
    }
}

impl From<PeerId> for multihash::Multihash {
    fn from(peer_id: PeerId) -> Self {
        peer_id.0.into()
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
            let peer_id = libp2p::PeerId::from_str(s.as_str()).map_err(D::Error::custom)?;
            Ok(PeerId(peer_id))
        } else {
            let b = <Vec<u8>>::deserialize(deserializer)?;
            let peer_id = libp2p::PeerId::from_bytes(b)
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
            self.0.as_bytes().serialize(serializer)
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
    pub latest_header: BlockHeader,
    pub total_difficulty: U256,
    pub rpc_protocols: Vec<(Cow<'static, [u8]>, RpcInfo)>,
}

impl PeerInfo {
    pub fn new_for_test(
        peer_id: PeerId,
        rpc_protocols: Vec<(Cow<'static, [u8]>, RpcInfo)>,
    ) -> Self {
        PeerInfo {
            peer_id,
            latest_header: BlockHeader::random(),
            total_difficulty: U256::zero(),
            rpc_protocols,
        }
    }

    pub fn new_with_proto(
        peer_id: PeerId,
        total_difficulty: U256,
        latest_header: BlockHeader,
        rpc_protocols: Vec<(Cow<'static, [u8]>, RpcInfo)>,
    ) -> Self {
        PeerInfo {
            peer_id,
            latest_header,
            total_difficulty,
            rpc_protocols,
        }
    }

    pub fn new_only_proto(rpc_protocols: Vec<(Cow<'static, [u8]>, RpcInfo)>) -> Self {
        let mut only_proto = Self::random();
        only_proto.rpc_protocols = rpc_protocols;
        only_proto
    }

    pub fn new_with_peer_info(
        peer_id: PeerId,
        total_difficulty: U256,
        latest_header: BlockHeader,
        old_peer_info: &PeerInfo,
    ) -> Self {
        PeerInfo {
            peer_id,
            latest_header,
            total_difficulty,
            rpc_protocols: old_peer_info.rpc_protocols.clone(),
        }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block_number(&self) -> BlockNumber {
        self.latest_header.number()
    }

    pub fn get_hash_number(&self) -> (HashValue, BlockNumber) {
        (self.latest_header.id(), self.latest_header.number())
    }

    pub fn get_block_id(&self) -> HashValue {
        self.latest_header.id()
    }

    pub fn get_total_difficulty(&self) -> U256 {
        self.total_difficulty
    }

    pub fn exist_rpc_proto(&self, rpc_proto_name: &[u8]) -> bool {
        let mut exist = false;
        for (name, _) in &self.rpc_protocols {
            if name == &Cow::Borrowed(rpc_proto_name) {
                exist = true;
                break;
            }
        }

        exist
    }

    pub fn register_rpc_proto(&mut self, rpc_proto_name: Cow<'static, [u8]>, rpc_info: RpcInfo) {
        assert!(!rpc_info.is_empty());
        // self.rpc_protocols
        //     .retain(|(name, _)| name != &rpc_proto_name);
        if !self.exist_rpc_proto(&rpc_proto_name) {
            self.rpc_protocols.push((rpc_proto_name, rpc_info));
        }
    }

    pub fn random() -> Self {
        Self {
            peer_id: PeerId::random(),
            total_difficulty: U256::from(0),
            latest_header: BlockHeader::random(),
            rpc_protocols: Vec::new(),
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

    pub fn new(paths: Vec<String>) -> Self {
        let mut inner = Vec::new();
        paths.iter().for_each(|path| {
            inner.retain(|p| p != path);
            inner.push(path.clone());
        });
        Self { paths: inner }
    }
}
