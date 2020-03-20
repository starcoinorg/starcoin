// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::mock::KeyPairWallet;
use crate::{Wallet, WalletAccount, WalletAsyncService};
use anyhow::Result;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::sync::Arc;

pub struct MockWalletService {
    wallet: Arc<dyn Wallet>,
}

#[allow(dead_code)]
impl MockWalletService {
    pub fn new() -> Result<Self> {
        Ok(Self::new_with_wallet(Arc::new(KeyPairWallet::new()?)))
    }

    pub fn new_with_wallet(wallet: Arc<dyn Wallet>) -> Self {
        Self { wallet }
    }
}

#[async_trait::async_trait(? Send)]
impl WalletAsyncService for MockWalletService {
    async fn create_account(self, password: &str) -> Result<WalletAccount> {
        self.wallet.create_account(password)
    }

    async fn get_default_account(self) -> Result<Option<WalletAccount>> {
        self.wallet.get_default_account()
    }

    async fn get_accounts(self) -> Result<Vec<WalletAccount>> {
        self.wallet.get_accounts()
    }

    async fn sign_txn(self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        self.wallet.sign_txn(raw_txn)
    }
}
