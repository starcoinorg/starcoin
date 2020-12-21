// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path::AccessPath;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::CryptoHasher;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct AccessResourceBlob {
    access_path: AccessPath,
    value: Vec<u8>,
}

impl AccessResourceBlob {
    pub fn new(access_path: AccessPath, value: Vec<u8>) -> AccessResourceBlob {
        AccessResourceBlob { access_path, value }
    }

    pub fn value(element_bytes: Option<Vec<u8>>) -> Result<Option<Vec<u8>>> {
        match element_bytes {
            Some(bytes) => {
                let blob: AccessResourceBlob = scs::from_bytes(bytes.as_slice())?;
                Ok(Some(blob.value))
            }
            None => Ok(None),
        }
    }

    pub fn into_bytes(&self) -> Result<Vec<u8>> {
        scs::to_bytes(self)
    }
}
