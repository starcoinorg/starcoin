// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transaction::authenticator::AuthenticationKey;
use bech32::ToBase32;
pub use move_core_types::account_address::AccountAddress;
use move_core_types::account_address::AccountAddressParseError;
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::hash::{CryptoHasher, HashValue};

pub fn from_public_key(public_key: &Ed25519PublicKey) -> AccountAddress {
    AuthenticationKey::ed25519(public_key).derived_address()
}

// Define the Hasher used for hashing AccountAddress types. In order to properly use the
// CryptoHasher derive macro we need to have this in its own module so that it doesn't conflict
// with the imported `AccountAddress` from move-core-types. It needs to have the same name since
// the hash salt is calculated using the name of the type.
mod hasher {
    use starcoin_crypto::hash::CryptoHasher;
    #[derive(serde::Deserialize, CryptoHasher)]
    struct AccountAddress;
}

pub trait HashAccountAddress {
    fn hash(&self) -> HashValue;
}

impl HashAccountAddress for AccountAddress {
    fn hash(&self) -> HashValue {
        let mut state = hasher::AccountAddressHasher::default();
        state.update(self.as_ref());
        state.finish()
    }
}

pub trait Bech32AccountAddress {
    fn to_bech32(&self) -> String;
    fn from_bech32(s: impl AsRef<str>) -> Result<Self, AccountAddressParseError>
    where
        Self: Sized;
}

fn parse_bench32(s: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let (hrp, data, variant) = bech32::decode(s.as_ref())?;

    anyhow::ensure!(variant == bech32::Variant::Bech32, "expect bech32 encoding");
    anyhow::ensure!(hrp.as_str() == "stc", "expect bech32 hrp to be stc");

    let version = data.first().map(|u| u.to_u8());
    anyhow::ensure!(version.filter(|v| *v == 1u8).is_some(), "expect version 1");

    let data: Vec<u8> = bech32::FromBase32::from_base32(&data[1..])?;

    if data.len() == AccountAddress::LENGTH {
        Ok(data)
    } else if data.len() == AccountAddress::LENGTH + 32 {
        // for address + auth key format, just ignore auth key
        Ok(data[0..AccountAddress::LENGTH].to_vec())
    } else {
        anyhow::bail!("Invalid address's length");
    }
}

impl Bech32AccountAddress for AccountAddress {
    fn to_bech32(&self) -> String {
        let mut data = self.to_vec().to_base32();
        data.insert(
            0,
            bech32::u5::try_from_u8(1).expect("1 to u8 should success"),
        );
        bech32::encode("stc", data, bech32::Variant::Bech32).expect("bech32 encode should success")
    }

    // TODO(simon): Handle parse error properly.
    fn from_bech32(s: impl AsRef<str>) -> Result<Self, AccountAddressParseError> {
        Self::from_bytes(
            parse_bench32(s)
                .map_err(|e| AccountAddressParseError::InvalidHexChars(e.to_string()))?,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;

    #[test]
    fn address_hash() {
        let address: AccountAddress = "0xca843279e3427144cead5e4d5999a3d0".parse().unwrap();

        let hash_vec =
            Vec::from_hex("7d39654178dd4758d0bc33b26e3e06051f04a215fd7ad270d4fb5e4988c8e5d2")
                .expect("You must provide a valid Hex format");

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_vec[..32]);

        assert_eq!(address.hash(), HashValue::new(hash));
    }
}
