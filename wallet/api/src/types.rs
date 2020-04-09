// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::{test_utils::KeyPair, Uniform};
use starcoin_types::account_address::{AccountAddress, AuthenticationKey};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct WalletAccount {
    //TODO should contains a unique local name?
    //name: String,
    pub address: AccountAddress,
    /// This account is default at current wallet.
    /// Every wallet must has one default account.
    pub is_default: bool,
    pub public_key: Ed25519PublicKey,
}

impl WalletAccount {
    pub fn new(address: AccountAddress, public_key: Ed25519PublicKey, is_default: bool) -> Self {
        Self {
            address,
            public_key,
            is_default,
        }
    }

    pub fn get_auth_key(&self) -> AuthenticationKey {
        AuthenticationKey::from_public_key(&self.public_key)
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }

    pub fn random() -> Self {
        let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
        let seed_buf: [u8; 32] = seed_rng.gen();
        let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
        let key_pair = KeyPair::generate_for_testing(&mut rng);
        let address = AccountAddress::from_public_key(&key_pair.public_key);
        WalletAccount {
            address,
            is_default: false,
            public_key: key_pair.public_key,
        }
    }
}
