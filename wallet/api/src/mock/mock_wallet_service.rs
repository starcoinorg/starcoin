// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::mock::{KeyPairWallet, MemWalletStore};
use crate::{Wallet, WalletAccount, WalletAsyncService};
use anyhow::{Error, Result};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct MockWalletService {
    wallet: Arc<KeyPairWallet<MemWalletStore>>,
}

impl MockWalletService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            wallet: Arc::new(KeyPairWallet::new()?),
        })
    }
}

#[async_trait::async_trait]
impl WalletAsyncService for MockWalletService {
    async fn create_account(self, password: String) -> Result<WalletAccount> {
        self.wallet.create_account(password.as_str())
    }

    async fn get_default_account(self) -> Result<Option<WalletAccount>> {
        self.wallet.get_default_account()
    }

    async fn get_accounts(self) -> Result<Vec<WalletAccount>> {
        self.wallet.get_accounts()
    }

    async fn get_account(self, address: AccountAddress) -> Result<Option<WalletAccount>> {
        self.wallet.get_account(&address)
    }

    async fn sign_txn(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        self.wallet.sign_txn(raw_txn)
    }

    async fn unlock_account(
        self,
        address: AccountAddress,
        password: String,
        duration: Duration,
    ) -> Result<(), Error> {
        self.wallet
            .unlock_account(address, password.as_str(), duration)
    }

    async fn import_account(
        self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> Result<WalletAccount> {
        self.wallet
            .import_account(address, private_key, password.as_str())
    }

    /// Return the private key as bytes for `address`
    async fn export_account(self, address: AccountAddress, password: String) -> Result<Vec<u8>> {
        self.wallet.export_account(&address, password.as_str())
    }
}
