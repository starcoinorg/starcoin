// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use anyhow::Result;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::{Wallet, WalletAccount, WalletService};
use std::sync::Arc;

pub struct WalletServiceImpl {
    //TODO support multi wallet.
    wallet: Arc<dyn Wallet>,
}

impl WalletServiceImpl {
    pub fn new(wallet: Arc<dyn Wallet>) -> Self {
        Self { wallet }
    }
}

impl WalletService for WalletServiceImpl {}

impl Wallet for WalletServiceImpl {
    fn create_account(&self, password: &str) -> Result<WalletAccount> {
        self.wallet.create_account(password)
    }

    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>> {
        self.wallet.get_account(address)
    }

    fn import_account(&self, private_key: Vec<u8>, password: &str) -> Result<WalletAccount> {
        self.wallet.import_account(private_key, password)
    }

    fn contains(&self, address: &AccountAddress) -> Result<bool> {
        self.wallet.contains(address)
    }

    fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> Result<()> {
        self.wallet.unlock_account(address, password, duration)
    }

    fn lock_account(&self, address: AccountAddress) -> Result<()> {
        self.wallet.lock_account(address)
    }

    fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        self.wallet.sign_txn(raw_txn)
    }

    fn get_default_account(&self) -> Result<Option<WalletAccount>> {
        self.wallet.get_default_account()
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>> {
        self.wallet.get_accounts()
    }

    fn set_default(&self, address: &AccountAddress) -> Result<()> {
        self.wallet.set_default(address)
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<()> {
        self.wallet.remove_account(address)
    }
}
