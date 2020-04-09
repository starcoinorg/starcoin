// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::WalletError;
use crate::WalletAccount;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::Duration;

pub type WalletResult<T> = std::result::Result<T, WalletError>;

pub trait Wallet {
    fn create_account(&self, password: &str) -> WalletResult<WalletAccount>;

    fn get_account(&self, address: &AccountAddress) -> WalletResult<Option<WalletAccount>>;

    fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: &str,
    ) -> WalletResult<WalletAccount>;

    /// Return the private key as bytes for `address`
    fn export_account(&self, address: &AccountAddress, password: &str) -> WalletResult<Vec<u8>>;

    fn contains(&self, address: &AccountAddress) -> WalletResult<bool>;

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> WalletResult<()>;

    fn lock_account(&self, address: AccountAddress) -> WalletResult<()>;

    /// Sign transaction by txn sender's Account.
    /// If the wallet is protected by password, should unlock the sender's account first.
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> WalletResult<SignedUserTransaction>;

    /// Return the default account
    fn get_default_account(&self) -> WalletResult<Option<WalletAccount>>;

    fn get_accounts(&self) -> WalletResult<Vec<WalletAccount>>;

    /// Set the address's Account to default account, and unset the origin default account.
    fn set_default(&self, address: &AccountAddress) -> WalletResult<()>;

    /// Remove account by address.
    /// Wallet must ensure that the default account can not bean removed.
    fn remove_account(&self, address: &AccountAddress) -> WalletResult<()>;
}
