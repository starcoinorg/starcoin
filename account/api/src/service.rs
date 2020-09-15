// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{AccountRequest, AccountResponse};
use crate::AccountInfo;
use anyhow::Result;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

#[async_trait::async_trait]
pub trait AccountAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    async fn create_account(&self, password: String) -> Result<AccountInfo>;

    async fn get_default_account(&self) -> Result<Option<AccountInfo>>;
    async fn set_default_account(&self, address: AccountAddress) -> Result<Option<AccountInfo>>;
    async fn get_accounts(&self) -> Result<Vec<AccountInfo>>;

    async fn get_account(&self, address: AccountAddress) -> Result<Option<AccountInfo>>;

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> Result<SignedUserTransaction>;
    async fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> Result<()>;
    async fn lock_account(&self, address: AccountAddress) -> Result<()>;
    async fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> Result<AccountInfo>;

    /// Return the private key as bytes for `address`
    async fn export_account(&self, address: AccountAddress, password: String) -> Result<Vec<u8>>;

    async fn accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>>;
}

#[async_trait::async_trait]
impl<S> AccountAsyncService for ServiceRef<S>
where
    S: ActorService,
    S: ServiceHandler<S, AccountRequest>,
{
    async fn create_account(&self, password: String) -> Result<AccountInfo> {
        let response = self.send(AccountRequest::CreateAccount(password)).await??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_default_account(&self) -> Result<Option<AccountInfo>> {
        let response = self.send(AccountRequest::GetDefaultAccount()).await??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }
    async fn set_default_account(&self, address: AccountAddress) -> Result<Option<AccountInfo>> {
        let response = self
            .send(AccountRequest::SetDefaultAccount(address))
            .await??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_accounts(&self) -> Result<Vec<AccountInfo>> {
        let response = self.send(AccountRequest::GetAccounts()).await??;
        if let AccountResponse::AccountList(accounts) = response {
            Ok(accounts)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account(&self, address: AccountAddress) -> Result<Option<AccountInfo>> {
        let response = self.send(AccountRequest::GetAccount(address)).await??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> Result<SignedUserTransaction> {
        let response = self
            .send(AccountRequest::SignTxn {
                txn: Box::new(raw_txn),
                signer: signer_address,
            })
            .await??;
        if let AccountResponse::SignedTxn(txn) = response {
            Ok(*txn)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> Result<()> {
        let response = self
            .send(AccountRequest::UnlockAccount(address, password, duration))
            .await??;
        if let AccountResponse::UnlockAccountResponse = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn lock_account(&self, address: AccountAddress) -> Result<()> {
        let response = self.send(AccountRequest::LockAccount(address)).await??;
        if let AccountResponse::None = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> Result<AccountInfo> {
        let response = self
            .send(AccountRequest::ImportAccount {
                address,
                password,
                private_key,
            })
            .await??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn export_account(&self, address: AccountAddress, password: String) -> Result<Vec<u8>> {
        let response = self
            .send(AccountRequest::ExportAccount { address, password })
            .await??;
        if let AccountResponse::ExportAccountResponse(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>> {
        let response = self
            .send(AccountRequest::AccountAcceptedTokens { address })
            .await??;
        if let AccountResponse::AcceptedTokens(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }
}
