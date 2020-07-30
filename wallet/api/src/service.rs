// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::AccountServiceError;
use crate::{Wallet, WalletAccount};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub type ServiceResult<T> = std::result::Result<T, AccountServiceError>;

pub trait WalletService: Wallet {}

#[async_trait::async_trait]
pub trait WalletAsyncService: Clone + std::marker::Unpin + Send + Sync {
    async fn create_account(self, password: String) -> ServiceResult<WalletAccount>;

    async fn get_default_account(self) -> ServiceResult<Option<WalletAccount>>;

    async fn get_accounts(self) -> ServiceResult<Vec<WalletAccount>>;

    async fn get_account(self, address: AccountAddress) -> ServiceResult<Option<WalletAccount>>;

    async fn sign_txn(
        self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> ServiceResult<SignedUserTransaction>;
    async fn unlock_account(
        self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> ServiceResult<()>;
    async fn lock_account(self, address: AccountAddress) -> ServiceResult<()>;
    async fn import_account(
        self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> ServiceResult<WalletAccount>;

    /// Return the private key as bytes for `address`
    async fn export_account(
        self,
        address: AccountAddress,
        password: String,
    ) -> ServiceResult<Vec<u8>>;
}
