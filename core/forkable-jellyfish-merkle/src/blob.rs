// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use hex;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::*;
use std::fmt;
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Blob {
    blob: Vec<u8>,
}

impl fmt::Debug for Blob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // let decoded = lcs::from_bytes(&self.blob)
        //     .map(|account_state: AccountState| format!("{:#?}", account_state))
        //     .unwrap_or_else(|_| String::from("[fail]"));

        write!(
            f,
            "Blob {{ \n \
             Raw: 0x{} \n \
             }}",
            hex::encode(&self.blob),
            // decoded,
        )
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.blob
    }
}

impl From<Blob> for Vec<u8> {
    fn from(account_state_blob: Blob) -> Vec<u8> {
        account_state_blob.blob
    }
}

impl From<Vec<u8>> for Blob {
    fn from(blob: Vec<u8>) -> Blob {
        Blob { blob }
    }
}

impl CryptoHash for Blob {
    fn crypto_hash(&self) -> HashValue {
        HashValue::from_sha3_256(&self.blob)
    }
}
