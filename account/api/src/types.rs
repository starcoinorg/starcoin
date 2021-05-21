// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::keygen::KeyGen;
use starcoin_types::{
    account_address::{self, AccountAddress},
    transaction::authenticator::AuthenticationKey,
};

use starcoin_types::receipt_identifier::ReceiptIdentifier;
pub use starcoin_types::transaction::authenticator::{
    AccountPrivateKey, AccountPublicKey, AccountSignature,
};

#[derive(Clone, Debug, Hash, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: AccountAddress,
    /// This account is default at current wallet.
    /// Every wallet must has one default account.
    pub is_default: bool,
    pub public_key: AccountPublicKey,
    pub receipt_identifier: ReceiptIdentifier,
}

impl AccountInfo {
    pub fn new(address: AccountAddress, public_key: AccountPublicKey, is_default: bool) -> Self {
        let auth_key = public_key.authentication_key();
        Self {
            address,
            public_key,
            is_default,
            receipt_identifier: ReceiptIdentifier::V1(address, Some(auth_key)),
        }
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        self.public_key.authentication_key()
    }

    pub fn address(&self) -> &AccountAddress {
        &self.address
    }

    pub fn random() -> Self {
        let mut key_gen = KeyGen::from_os_rng();
        let (_private_key, public_key) = key_gen.generate_keypair();
        let address = account_address::from_public_key(&public_key);
        let account_public_key = AccountPublicKey::Single(public_key);
        let auth_key = account_public_key.authentication_key();
        AccountInfo {
            address,
            is_default: false,
            public_key: account_public_key,
            receipt_identifier: ReceiptIdentifier::V1(address, Some(auth_key)),
        }
    }
}
