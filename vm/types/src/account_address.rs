// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use move_core_types::account_address::AccountAddress;

use crate::transaction::authenticator::AuthenticationKey;
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

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;

    #[test]
    fn address_hash() {
        let address: AccountAddress = "ca843279e3427144cead5e4d5999a3d0".parse().unwrap();

        let hash_vec =
            Vec::from_hex("7d39654178dd4758d0bc33b26e3e06051f04a215fd7ad270d4fb5e4988c8e5d2")
                .expect("You must provide a valid Hex format");

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_vec[..32]);

        assert_eq!(address.hash(), HashValue::new(hash));
    }
}
