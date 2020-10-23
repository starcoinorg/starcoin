// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::ed25519::{
    Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature, ED25519_PRIVATE_KEY_LENGTH,
    ED25519_PUBLIC_KEY_LENGTH,
};
use starcoin_crypto::hash::CryptoHash;
use starcoin_crypto::keygen::KeyGen;
use starcoin_crypto::multi_ed25519::multi_shard::{
    MultiEd25519KeyShard, MultiEd25519SignatureShard,
};
use starcoin_crypto::multi_ed25519::MultiEd25519PublicKey;
use starcoin_crypto::{PrivateKey, SigningKey, ValidCryptoMaterialStringExt};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_types::{
    account_address::{self, AccountAddress},
    transaction::authenticator::AuthenticationKey,
};
use std::convert::TryFrom;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AccountPublicKey {
    Single(Ed25519PublicKey),
    Multi(MultiEd25519PublicKey),
}

#[derive(Eq, PartialEq, Debug)]
pub enum AccountPrivateKey {
    Single(Ed25519PrivateKey),
    Multi(MultiEd25519KeyShard),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AccountSignature {
    Single(Ed25519PublicKey, Ed25519Signature),
    Multi(MultiEd25519PublicKey, MultiEd25519SignatureShard),
}

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct AccountInfo {
    //TODO should contains a unique local name?
    //name: String,
    pub address: AccountAddress,
    /// This account is default at current wallet.
    /// Every wallet must has one default account.
    pub is_default: bool,
    pub public_key: AccountPublicKey,
}

impl AccountPublicKey {
    pub fn derived_address(&self) -> AccountAddress {
        self.auth_key().derived_address()
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        match self {
            Self::Single(key) => AuthenticationKey::ed25519(key),
            Self::Multi(key) => AuthenticationKey::multi_ed25519(key),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Single(key) => key.to_bytes().to_vec(),
            Self::Multi(key) => key.to_bytes(),
        }
    }

    pub fn to_encoded_string(&self) -> Result<String> {
        match self {
            Self::Single(key) => key.to_encoded_string(),
            Self::Multi(key) => key.to_encoded_string(),
        }
    }

    fn from_encoded_string(encoded_str: &str) -> Result<Self> {
        let bytes = ::hex::decode(encoded_str)?;
        Self::try_from(bytes.as_slice())
    }

    pub fn as_single(&self) -> Option<Ed25519PublicKey> {
        match self {
            Self::Single(key) => Some(key.clone()),
            _ => None,
        }
    }

    pub fn as_multi(&self) -> Option<MultiEd25519PublicKey> {
        match self {
            Self::Multi(key) => Some(key.clone()),
            _ => None,
        }
    }
}

impl Serialize for AccountPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_encoded_string()
                .map_err(<S::Error as ::serde::ser::Error>::custom)
                .and_then(|str| serializer.serialize_str(&str[..]))
        } else {
            serializer.serialize_bytes(self.to_bytes().as_slice())
        }
    }
}

impl<'de> Deserialize<'de> for AccountPublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            Self::from_encoded_string(encoded_key.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "AccountPublicKey")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            AccountPublicKey::try_from(value.0).map_err(|s| {
                <D::Error as ::serde::de::Error>::custom(format!(
                    "{} with {}",
                    s, "AccountPublicKey"
                ))
            })
        }
    }
}

impl TryFrom<&[u8]> for AccountPublicKey {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(if value.len() == ED25519_PUBLIC_KEY_LENGTH {
            Self::Single(Ed25519PublicKey::try_from(value)?)
        } else {
            Self::Multi(MultiEd25519PublicKey::try_from(value)?)
        })
    }
}

impl Into<AccountPublicKey> for Ed25519PublicKey {
    fn into(self) -> AccountPublicKey {
        AccountPublicKey::Single(self)
    }
}

impl Into<AccountPublicKey> for MultiEd25519PublicKey {
    fn into(self) -> AccountPublicKey {
        AccountPublicKey::Multi(self)
    }
}

impl AccountPrivateKey {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Single(key) => key.to_bytes().to_vec(),
            Self::Multi(key) => key.to_bytes(),
        }
    }

    pub fn from_encoded_string(encoded_str: &str) -> Result<Self> {
        let bytes = ::hex::decode(encoded_str)?;
        Self::try_from(bytes.as_slice())
    }

    pub fn public_key(&self) -> AccountPublicKey {
        match self {
            Self::Single(key) => AccountPublicKey::Single(key.public_key()),
            Self::Multi(key) => AccountPublicKey::Multi(key.public_key()),
        }
    }

    pub fn sign<T: CryptoHash + Serialize>(&self, message: &T) -> AccountSignature {
        match self {
            Self::Single(key) => AccountSignature::Single(key.public_key(), key.sign(message)),
            Self::Multi(key) => AccountSignature::Multi(key.public_key(), key.sign(message)),
        }
    }
}

impl Into<AccountPrivateKey> for Ed25519PrivateKey {
    fn into(self) -> AccountPrivateKey {
        AccountPrivateKey::Single(self)
    }
}

impl Into<AccountPrivateKey> for MultiEd25519KeyShard {
    fn into(self) -> AccountPrivateKey {
        AccountPrivateKey::Multi(self)
    }
}

impl TryFrom<&[u8]> for AccountPrivateKey {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Ok(if value.len() == ED25519_PRIVATE_KEY_LENGTH {
            Self::Single(Ed25519PrivateKey::try_from(value)?)
        } else {
            Self::Multi(MultiEd25519KeyShard::try_from(value)?)
        })
    }
}

impl AccountSignature {
    pub fn build_transaction(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        Ok(match self {
            Self::Single(public_key, signature) => {
                SignedUserTransaction::ed25519(raw_txn, public_key, signature)
            }
            Self::Multi(public_key, signature) => {
                if signature.is_enough() {
                    SignedUserTransaction::multi_ed25519(raw_txn, public_key, signature.into())
                } else {
                    bail!(
                        "MultiEd25519SignatureShard do not have enough signatures, current: {}, threshold: {}",
                        signature.signatures().len(),
                        signature.threshold()
                    )
                }
            }
        })
    }
}

impl AccountInfo {
    pub fn new(address: AccountAddress, public_key: AccountPublicKey, is_default: bool) -> Self {
        Self {
            address,
            public_key,
            is_default,
        }
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        self.public_key.auth_key()
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }

    pub fn random() -> Self {
        let mut key_gen = KeyGen::from_os_rng();
        let (_private_key, public_key) = key_gen.generate_keypair();
        let address = account_address::from_public_key(&public_key);
        AccountInfo {
            address,
            is_default: false,
            public_key: AccountPublicKey::Single(public_key),
        }
    }
}
