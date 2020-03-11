// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
