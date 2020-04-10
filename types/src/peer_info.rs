// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libp2p::identity::PublicKey;
use libp2p::multihash;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::ed25519::Ed25519PublicKey;

use crate::{block::BlockNumber, U512};
use starcoin_crypto::HashValue;
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
            let s = <&str>::deserialize(deserializer)?;
            let peer_id = libp2p::PeerId::from_str(s).map_err(D::Error::custom)?;
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
    pub block_number: BlockNumber,
    pub total_difficult: U512,
    pub block_id: HashValue,
}

impl PeerInfo {
    pub fn new_for_test(peer_id: PeerId) -> Self {
        PeerInfo {
            peer_id,
            block_number: 0,
            total_difficult: U512::zero(),
            block_id: HashValue::random(),
        }
    }

    pub fn new(
        peer_id: PeerId,
        block_number: BlockNumber,
        total_difficult: U512,
        block_id: HashValue,
    ) -> Self {
        PeerInfo {
            peer_id,
            block_number,
            total_difficult,
            block_id,
        }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block_number(&self) -> BlockNumber {
        self.block_number
    }

    pub fn default() -> Self {
        Self {
            peer_id: PeerId::random(),
            block_number: 0,
            total_difficult: U512::from(0),
            block_id: HashValue::default(),
        }
    }
}
