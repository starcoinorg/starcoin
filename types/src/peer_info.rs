// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{hash::CryptoHash, HashValue};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct PeerInfo {
    pub id: AccountAddress,
}

impl PeerInfo {
    pub fn new(address: AccountAddress) -> Self {
        PeerInfo { id: address }
    }

    pub fn random() -> Self {
        PeerInfo {
            id: AccountAddress::random(),
        }
    }
}
