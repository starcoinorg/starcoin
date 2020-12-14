// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! A library supplying various cryptographic primitives
// just wrap diem-crypto.

pub mod ed25519 {
    use crate::keygen::KeyGen;
    use crate::{Genesis, PrivateKey};
    pub use diem_crypto::ed25519::*;

    pub fn random_public_key() -> Ed25519PublicKey {
        KeyGen::from_os_rng().generate_keypair().1
    }

    /// A static key pair
    pub fn genesis_key_pair() -> (Ed25519PrivateKey, Ed25519PublicKey) {
        let private_key = Ed25519PrivateKey::genesis();
        let public_key = private_key.public_key();
        (private_key, public_key)
    }
}

pub mod hash;
pub mod keygen;
pub mod multi_ed25519;

pub mod test_utils {
    pub use diem_crypto::test_utils::*;
}

pub mod traits {
    pub use diem_crypto::traits::*;
}

pub use crate::hash::HashValue;
pub use crate::traits::*;

// Reexport once_cell for use in CryptoHasher Derive implementation
#[doc(hidden)]
pub use once_cell as _once_cell;
#[doc(hidden)]
pub use serde_name as _serde_name;

pub mod derive {
    pub use diem_crypto_derive::{DeserializeKey, SerializeKey, SilentDebug, SilentDisplay};
}
