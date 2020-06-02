// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! A library supplying various cryptographic primitives
// just wrap libra-crypto.

pub mod ed25519 {
    pub use libra_crypto::ed25519::*;
}

pub mod hash;
pub mod keygen;

pub mod test_utils {
    pub use libra_crypto::test_utils::*;
}

pub mod traits {
    pub use libra_crypto::traits::*;
}

pub use crate::hash::HashValue;
pub use crate::traits::*;

// Reexport once_cell for use in CryptoHasher Derive implementation
#[doc(hidden)]
pub use once_cell as _once_cell;
#[doc(hidden)]
pub use serde_name as _serde_name;
