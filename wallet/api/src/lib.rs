// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

#[cfg(any(test, feature = "mock"))]
pub mod mock;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct WalletAccount {
    //TODO should contains a unique local name?
    //name: String,
    pub address: AccountAddress,
    /// This account is default at current wallet.
    /// Every wallet must has one default account.
    pub is_default: bool,
}

impl WalletAccount {
    pub fn new(address: AccountAddress, is_default: bool) -> Self {
        Self {
            address,
            is_default,
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

#[async_trait::async_trait(? Send)]
pub trait WalletAsyncService {
    async fn create_account(self, password: &str) -> Result<WalletAccount>;

    async fn get_default_account(self) -> Result<Option<WalletAccount>>;

    async fn get_accounts(self) -> Result<Vec<WalletAccount>>;

    async fn sign_txn(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction>;
}
