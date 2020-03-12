// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libp2p::identity::PublicKey;
use libp2p::multihash;
use serde::{de::Error as _, de::Unexpected, Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::{hash::CryptoHash, HashValue};
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
    pub id: PeerId,
}

impl PeerInfo {
    pub fn new(id: PeerId) -> Self {
        PeerInfo { id }
    }

    pub fn random() -> Self {
        PeerInfo {
            id: PeerId::random(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_info() {
        let peer_info = PeerInfo::random();
    }
}
