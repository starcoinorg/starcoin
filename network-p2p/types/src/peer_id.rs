// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libp2p::core::{
    identity::PublicKey, multihash::Error, multihash::Multihash, PeerId as Libp2pPeerId,
};
use schemars::{self, JsonSchema};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Eq, PartialEq, Hash, Clone, Debug, JsonSchema, Copy)]
pub struct PeerId(#[schemars(with = "String")] Libp2pPeerId);

impl PeerId {
    pub fn new(peer_id: Libp2pPeerId) -> Self {
        Self(peer_id)
    }

    /// Builds a `PeerId` from a public key.
    pub fn from_public_key(key: PublicKey) -> PeerId {
        Self::new(Libp2pPeerId::from_public_key(&key))
    }

    pub fn from_ed25519_public_key(key: Ed25519PublicKey) -> PeerId {
        let pub_key = libp2p::core::identity::ed25519::PublicKey::decode(key.to_bytes().as_ref())
            .expect("Decode pubkey must success.");
        Self::from_public_key(PublicKey::Ed25519(pub_key))
    }

    /// Checks whether `data` is a valid `PeerId`. If so, returns the `PeerId`. If not, returns
    /// back the data as an error.
    pub fn from_bytes(data: Vec<u8>) -> Result<PeerId, Error> {
        Ok(Self::new(Libp2pPeerId::from_bytes(&data)?))
    }

    /// Turns a `Multihash` into a `PeerId`. If the multihash doesn't use the correct algorithm,
    /// returns back the data as an error.
    pub fn from_multihash(data: Multihash) -> Result<PeerId, Multihash> {
        Ok(Self::new(Libp2pPeerId::from_multihash(data)?))
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.to_bytes()
    }

    /// Returns a base-58 encoded string of this `PeerId`.
    pub fn to_base58(&self) -> String {
        self.0.to_base58()
    }

    pub fn origin(&self) -> &Libp2pPeerId {
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
        Self(Libp2pPeerId::random())
    }
}

#[allow(clippy::from_over_into)]
impl Into<Libp2pPeerId> for PeerId {
    fn into(self) -> Libp2pPeerId {
        self.0
    }
}

impl From<Libp2pPeerId> for PeerId {
    fn from(peer_id: Libp2pPeerId) -> Self {
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
        Ok(Self(Libp2pPeerId::from_str(s)?))
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
            let peer_id = Libp2pPeerId::from_str(s.as_str()).map_err(D::Error::custom)?;
            Ok(PeerId(peer_id))
        } else {
            let b = <Vec<u8>>::deserialize(deserializer)?;
            let peer_id = Libp2pPeerId::from_bytes(&b)
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
        write!(f, "{}", self.0)
    }
}
