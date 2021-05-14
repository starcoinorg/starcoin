// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ed25519::{
    Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
    ED25519_PUBLIC_KEY_LENGTH,
};
use crate::hash::{CryptoHash, CryptoHasher};
use crate::{CryptoMaterialError, Length, PrivateKey, Signature, ValidCryptoMaterial};
use crate::{SigningKey, Uniform};
use anyhow::{anyhow, bail, ensure, Result};
use diem_crypto::multi_ed25519::{MultiEd25519PublicKey, MultiEd25519Signature};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

const MAX_NUM_OF_KEYS: usize = 32;
const BITMAP_NUM_OF_BYTES: usize = 4;

/// Part of private keys in the multi-key Ed25519 structure along with the threshold.
/// note: the private keys must be a sequential part of the MultiEd25519PrivateKey
#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub struct MultiEd25519KeyShard {
    /// Public keys must contains all public key of the MultiEd25519PrivateKey
    public_keys: Vec<Ed25519PublicKey>,
    threshold: u8,
    // pos => private_key
    private_keys: BTreeMap<usize, Ed25519PrivateKey>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MultiEd25519SignatureShard {
    signature: MultiEd25519Signature,
    threshold: u8,
}

impl MultiEd25519KeyShard {
    pub fn new(
        public_keys: Vec<Ed25519PublicKey>,
        threshold: u8,
        private_key: Ed25519PrivateKey,
    ) -> Result<Self, CryptoMaterialError> {
        Self::new_multi(public_keys, threshold, vec![private_key])
    }

    pub fn new_multi(
        public_keys: Vec<Ed25519PublicKey>,
        threshold: u8,
        private_keys: Vec<Ed25519PrivateKey>,
    ) -> Result<Self, CryptoMaterialError> {
        let num_of_public_keys = public_keys.len();
        let num_of_private_keys = private_keys.len();
        if threshold == 0 || num_of_private_keys == 0 || num_of_public_keys < threshold as usize {
            Err(CryptoMaterialError::ValidationError)
        } else if num_of_private_keys > MAX_NUM_OF_KEYS || num_of_public_keys > MAX_NUM_OF_KEYS {
            Err(CryptoMaterialError::WrongLengthError)
        } else {
            let mut pos_verified_private_keys = BTreeMap::default();
            for private_key in private_keys {
                let private_key_public_key = private_key.public_key();
                match public_keys
                    .iter()
                    .position(|k| k == &private_key_public_key)
                {
                    Some(pos) => pos_verified_private_keys.insert(pos, private_key),
                    None => return Err(CryptoMaterialError::ValidationError),
                };
            }
            Ok(Self {
                public_keys,
                threshold,
                private_keys: pos_verified_private_keys,
            })
        }
    }

    /// Generate `shards` MultiEd25519SignatureShard for test
    pub fn generate<R>(rng: &mut R, shards: usize, threshold: u8) -> Result<Vec<Self>>
    where
        R: ::rand::RngCore + ::rand::CryptoRng,
    {
        ensure!(
            threshold as usize <= shards,
            "threshold should less than shards"
        );
        let private_keys = (0..shards)
            .map(|_i| Ed25519PrivateKey::generate(rng))
            .collect::<Vec<_>>();
        let public_keys = private_keys
            .iter()
            .map(|private_key| private_key.public_key())
            .collect::<Vec<_>>();
        private_keys
            .into_iter()
            .map(|private_key| {
                Self::new(public_keys.clone(), threshold, private_key).map_err(anyhow::Error::new)
            })
            .collect()
    }

    pub fn public_key(&self) -> MultiEd25519PublicKey {
        MultiEd25519PublicKey::new(self.public_keys.clone(), self.threshold)
            .expect("New MultiEd25519PublicKey should success.")
    }
    pub fn private_keys(&self) -> Vec<Ed25519PrivateKey> {
        self.private_keys.values().cloned().collect()
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn len(&self) -> usize {
        self.private_keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.private_keys.is_empty()
    }

    pub fn sign<T: CryptoHash + Serialize>(&self, message: &T) -> MultiEd25519SignatureShard {
        let signatures: Vec<(Ed25519Signature, u8)> = self
            .private_keys
            .iter()
            .map(|(i, item)| (item.sign(message), *i as u8))
            .collect();

        MultiEd25519SignatureShard::new(
            MultiEd25519Signature::new(signatures).expect("Init MultiEd25519Signature should ok"),
            self.threshold,
        )
    }
}

impl ValidCryptoMaterial for MultiEd25519KeyShard {
    /// Serialize a MultiEd25519PrivateKeyShard.
    #[allow(clippy::vec_init_then_push)]
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.push(self.public_keys.len() as u8);
        bytes.push(self.threshold);
        bytes.push(self.private_keys.len() as u8);

        bytes.extend(
            self.public_keys
                .iter()
                .flat_map(ValidCryptoMaterial::to_bytes)
                .collect::<Vec<u8>>(),
        );
        bytes.extend(
            self.private_keys
                .values()
                .flat_map(ValidCryptoMaterial::to_bytes)
                .collect::<Vec<u8>>(),
        );

        bytes
    }
}

impl std::fmt::Debug for MultiEd25519KeyShard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MultiEd25519PrivateKeyShard(public_keys={}, private_keys={}, threshold={})",
            self.public_keys.len(),
            self.private_keys.len(),
            self.threshold
        )
    }
}

impl TryFrom<&[u8]> for MultiEd25519KeyShard {
    type Error = CryptoMaterialError;

    /// Deserialize an Ed25519PrivateKey. This method will also check for key and threshold validity.
    fn try_from(bytes: &[u8]) -> Result<MultiEd25519KeyShard, Self::Error> {
        const HEADER_LEN: usize = 3;
        let bytes_len = bytes.len();
        if bytes_len < HEADER_LEN {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let public_key_len = bytes[0];
        let threshold = bytes[1];
        let private_key_len = bytes[2];

        let public_key_bytes_len = public_key_len as usize * ED25519_PUBLIC_KEY_LENGTH;
        let private_key_bytes_len = private_key_len as usize * ED25519_PRIVATE_KEY_LENGTH;
        if bytes_len < HEADER_LEN + public_key_bytes_len + private_key_bytes_len {
            return Err(CryptoMaterialError::WrongLengthError);
        }
        let public_key_bytes = &bytes[HEADER_LEN..HEADER_LEN + public_key_bytes_len];

        let public_keys: Result<Vec<Ed25519PublicKey>, _> = public_key_bytes
            .chunks_exact(ED25519_PUBLIC_KEY_LENGTH)
            .map(Ed25519PublicKey::try_from)
            .collect();

        let private_key_bytes = &bytes[HEADER_LEN + public_key_bytes_len..];
        let private_keys: Result<Vec<Ed25519PrivateKey>, _> = private_key_bytes
            .chunks_exact(ED25519_PRIVATE_KEY_LENGTH)
            .map(Ed25519PrivateKey::try_from)
            .collect();

        MultiEd25519KeyShard::new_multi(public_keys?, threshold, private_keys?)
    }
}

impl MultiEd25519SignatureShard {
    /// This method will also sort signatures based on index.
    pub fn new(signature: MultiEd25519Signature, threshold: u8) -> Self {
        Self {
            signature,
            threshold,
        }
    }

    pub fn merge(shards: Vec<Self>) -> Result<Self> {
        if shards.is_empty() {
            bail!("MultiEd25519SignatureShard shards is empty");
        }

        let threshold = shards[0].threshold;
        let mut signatures = vec![];
        for shard in shards {
            if shard.threshold != threshold {
                bail!("MultiEd25519SignatureShard shards threshold not same.")
            }
            signatures.extend(shard.signatures());
        }

        Ok(Self::new(
            MultiEd25519Signature::new(signatures)?,
            threshold,
        ))
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn is_enough(&self) -> bool {
        self.signature.signatures().len() >= self.threshold as usize
    }

    /// Getter signatures and index.
    pub fn signatures(&self) -> Vec<(Ed25519Signature, u8)> {
        self.into()
    }

    /// Getter bitmap.
    pub fn bitmap(&self) -> &[u8; BITMAP_NUM_OF_BYTES] {
        self.signature.bitmap()
    }

    /// Serialize a MultiEd25519SignatureShard
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.signature.to_bytes();
        bytes.push(self.threshold);
        bytes
    }

    pub fn verify<T: CryptoHash + Serialize>(
        &self,
        message: &T,
        public_key: &MultiEd25519PublicKey,
    ) -> Result<()> {
        let mut bytes = <T as CryptoHash>::Hasher::seed().to_vec();
        bcs_ext::serialize_into(&mut bytes, &message)
            .map_err(|_| CryptoMaterialError::SerializationError)?;
        self.verify_arbitrary_msg(&bytes, public_key)
    }

    /// Checks that `self` is valid for an arbitrary &[u8] `message` using `public_key`.
    /// Outside of this crate, this particular function should only be used for native signature
    /// verification in Move.
    fn verify_arbitrary_msg(
        &self,
        message: &[u8],
        public_key: &MultiEd25519PublicKey,
    ) -> Result<()> {
        ensure!(
            self.threshold == *public_key.threshold(),
            "public_key and signature threshold mismatch."
        );
        let bitmap = *self.bitmap();

        match bitmap_last_set_bit(bitmap) {
            Some(last_bit) if last_bit as usize <= public_key.length() => (),
            _ => {
                return Err(anyhow!(
                    "{}",
                    CryptoMaterialError::BitVecError("Signature index is out of range".to_string())
                ))
            }
        };

        let mut bitmap_index = 0;
        let signatures = self.signature.signatures();
        // TODO use deterministic batch verification when gets available.
        for sig in signatures {
            while !bitmap_get_bit(bitmap, bitmap_index) {
                bitmap_index += 1;
            }
            sig.verify_arbitrary_msg(message, &public_key.public_keys()[bitmap_index])?;
            bitmap_index += 1;
        }
        Ok(())
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<(Ed25519Signature, u8)>> for &MultiEd25519SignatureShard {
    fn into(self) -> Vec<(Ed25519Signature, u8)> {
        let signatures = self.signature.signatures();
        let bitmap = *self.signature.bitmap();
        let mut result = vec![];

        let mut bitmap_index = 0;
        for sig in signatures {
            while !bitmap_get_bit(bitmap, bitmap_index) {
                bitmap_index += 1;
            }
            result.push((sig.clone(), (bitmap_index as u8)));
            bitmap_index += 1;
        }

        result
    }
}

#[allow(clippy::from_over_into)]
impl Into<MultiEd25519Signature> for MultiEd25519SignatureShard {
    fn into(self) -> MultiEd25519Signature {
        self.signature
    }
}

impl TryFrom<Vec<MultiEd25519SignatureShard>> for MultiEd25519SignatureShard {
    type Error = anyhow::Error;

    fn try_from(value: Vec<MultiEd25519SignatureShard>) -> Result<Self, Self::Error> {
        MultiEd25519SignatureShard::merge(value)
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl std::hash::Hash for MultiEd25519SignatureShard {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let encoded_signature = self.to_bytes();
        state.write(&encoded_signature);
    }
}

impl fmt::Display for MultiEd25519SignatureShard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.to_bytes()[..]))
    }
}

impl fmt::Debug for MultiEd25519SignatureShard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MultiEd25519SignatureShard({})", self)
    }
}

// Helper method to get the input's bit at index.
fn bitmap_get_bit(input: [u8; BITMAP_NUM_OF_BYTES], index: usize) -> bool {
    let bucket = index / 8;
    // It's always invoked with index < 32, thus there is no need to check range.
    let bucket_pos = index - (bucket * 8);
    (input[bucket] & (128 >> bucket_pos as u8)) != 0
}

// Find the last set bit.
fn bitmap_last_set_bit(input: [u8; BITMAP_NUM_OF_BYTES]) -> Option<u8> {
    input
        .iter()
        .rev()
        .enumerate()
        .find(|(_, byte)| byte != &&0u8)
        .map(|(i, byte)| (8 * (BITMAP_NUM_OF_BYTES - i) - byte.trailing_zeros() as usize - 1) as u8)
}
