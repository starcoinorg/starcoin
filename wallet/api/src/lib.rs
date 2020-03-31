// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::Ed25519PublicKey;
use starcoin_crypto::{test_utils::KeyPair, Uniform};
use starcoin_types::account_address::{AccountAddress, AuthenticationKey};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

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

pub trait Wallet {
    fn create_account(&self, password: &str) -> Result<WalletAccount>;

    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>>;

    fn import_account(&self, private_key: Vec<u8>, password: &str) -> Result<WalletAccount>;

    fn contains(&self, address: &AccountAddress) -> Result<bool>;

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> Result<()>;

    fn lock_account(&self, address: AccountAddress) -> Result<()>;

    /// Sign transaction by txn sender's Account.
    /// If the wallet is protected by password, should unlock the sender's account first.
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction>;

    /// Return the default account
    fn get_default_account(&self) -> Result<Option<WalletAccount>>;

    fn get_accounts(&self) -> Result<Vec<WalletAccount>>;

    /// Set the address's Account to default account, and unset the origin default account.
    fn set_default(&self, address: &AccountAddress) -> Result<()>;

    /// Remove account by address.
    /// Wallet must ensure that the default account can not bean removed.
    fn remove_account(&self, address: &AccountAddress) -> Result<()>;
}

pub trait WalletStore {
    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>>;
    fn save_account(&self, account: WalletAccount) -> Result<()>;
    fn remove_account(&self, address: &AccountAddress) -> Result<()>;
    fn get_accounts(&self) -> Result<Vec<WalletAccount>>;
    fn save_to_account(&self, address: &AccountAddress, key: String, value: Vec<u8>) -> Result<()>;
    fn get_from_account(&self, address: &AccountAddress, key: &str) -> Result<Option<Vec<u8>>>;
}

pub trait WalletService: Wallet {}

#[async_trait::async_trait]
pub trait WalletAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn create_account(self, password: String) -> Result<WalletAccount>;

    async fn get_default_account(self) -> Result<Option<WalletAccount>>;

    async fn get_accounts(self) -> Result<Vec<WalletAccount>>;

    async fn get_account(self, address: AccountAddress) -> Result<Option<WalletAccount>>;

    async fn sign_txn(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction>;
}
