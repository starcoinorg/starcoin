// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Error, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use std::str::FromStr;

/// SigningMessage is a message to be signed and encapsulates the salt
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct SigningMessage {
    message: Vec<u8>,
}

impl FromStr for SigningMessage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(!s.is_empty(), "signing message should not be empty.",);
        Ok(Self {
            message: s.as_bytes().to_vec(),
        })
    }
}
