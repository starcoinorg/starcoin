// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::*;
use std::fmt;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct Blob {
    blob: Vec<u8>,
}

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Blob {{ \n \
             Raw: 0x{} \n \
             }}",
            hex::encode(&self.blob),
        )
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.blob
    }
}

impl From<Blob> for Vec<u8> {
    fn from(blob: Blob) -> Vec<u8> {
        blob.blob
    }
}

impl From<Vec<u8>> for Blob {
    fn from(blob: Vec<u8>) -> Blob {
        Blob { blob }
    }
}
