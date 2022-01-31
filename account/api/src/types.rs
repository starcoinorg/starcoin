// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::keygen::KeyGen;
use starcoin_types::account::Account;
use starcoin_types::account_address::AccountAddress;
pub use starcoin_types::transaction::authenticator::{AccountPrivateKey, AccountPublicKey};
use starcoin_types::{
    account_address::{self},
    transaction::authenticator::AuthenticationKey,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccountInfo {
    pub address: AccountAddress,
    /// This account is default at current wallet.
    /// Every wallet must has one default account.
    pub is_default: bool,
    pub is_readonly: bool,
    pub public_key: AccountPublicKey,
    pub receipt_identifier: String,
}

impl AccountInfo {
    pub fn new(
        address: AccountAddress,
        public_key: AccountPublicKey,
        is_default: bool,
        is_readonly: bool,
    ) -> Self {
        Self {
            address,
            public_key,
            is_default,
            is_readonly,
            receipt_identifier: address.to_bech32(),
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
        AccountInfo {
            address,
            is_default: false,
            is_readonly: false,
            public_key: account_public_key,
            receipt_identifier: address.to_bech32(),
        }
    }
}

impl From<&Account> for AccountInfo {
    fn from(account: &Account) -> Self {
        AccountInfo::new(account.addr, account.public_key(), false, false)
    }
}

#[derive(Clone, Debug)]
pub struct DefaultAccountChangeEvent {
    pub new_account: AccountInfo,
}
