// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::AccountServiceError;
use crate::message::{AccountRequest, AccountResponse};
use crate::AccountInfo;
use actix::dev::ToEnvelope;
use actix::{Actor, Addr, Handler};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub type ServiceResult<T> = std::result::Result<T, AccountServiceError>;

#[async_trait::async_trait]
pub trait AccountAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    async fn create_account(&self, password: String) -> ServiceResult<AccountInfo>;

    async fn get_default_account(&self) -> ServiceResult<Option<AccountInfo>>;
    async fn set_default_account(
        &self,
        address: AccountAddress,
    ) -> ServiceResult<Option<AccountInfo>>;
    async fn get_accounts(&self) -> ServiceResult<Vec<AccountInfo>>;

    async fn get_account(&self, address: AccountAddress) -> ServiceResult<Option<AccountInfo>>;

    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> ServiceResult<SignedUserTransaction>;
    async fn unlock_account(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> ServiceResult<()>;
    async fn lock_account(&self, address: AccountAddress) -> ServiceResult<()>;
    async fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> ServiceResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    async fn export_account(
        &self,
        address: AccountAddress,
        password: String,
    ) -> ServiceResult<Vec<u8>>;

    async fn accepted_tokens(&self, address: AccountAddress) -> ServiceResult<Vec<TokenCode>>;
}

#[async_trait::async_trait]
impl<A> AccountAsyncService for Addr<A>
where
    A: Actor,
    A: Handler<AccountRequest>,
    A::Context: ToEnvelope<A, AccountRequest>,
    A: std::marker::Send,
{
    async fn create_account(&self, password: String) -> ServiceResult<AccountInfo> {
        let response = self
            .send(AccountRequest::CreateAccount(password))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_default_account(&self) -> ServiceResult<Option<AccountInfo>> {
        let response = self
            .send(AccountRequest::GetDefaultAccount())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }
    async fn set_default_account(
        &self,
        address: AccountAddress,
    ) -> ServiceResult<Option<AccountInfo>> {
        let response = self
            .send(AccountRequest::SetDefaultAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfoOption(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_accounts(&self) -> ServiceResult<Vec<AccountInfo>> {
        let response = self
            .send(AccountRequest::GetAccounts())
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountList(accounts) = response {
            Ok(accounts)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account(&self, address: AccountAddress) -> ServiceResult<Option<AccountInfo>> {
        let response = self
            .send(AccountRequest::GetAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
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
    ) -> ServiceResult<SignedUserTransaction> {
        let response = self
            .send(AccountRequest::SignTxn {
                txn: Box::new(raw_txn),
                signer: signer_address,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
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
    ) -> ServiceResult<()> {
        let response = self
            .send(AccountRequest::UnlockAccount(address, password, duration))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::UnlockAccountResponse = response {
            Ok(())
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn lock_account(&self, address: AccountAddress) -> ServiceResult<()> {
        let response = self
            .send(AccountRequest::LockAccount(address))
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
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
    ) -> ServiceResult<AccountInfo> {
        let response = self
            .send(AccountRequest::ImportAccount {
                address,
                password,
                private_key,
            })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AccountInfo(account) = response {
            Ok(*account)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn export_account(
        &self,
        address: AccountAddress,
        password: String,
    ) -> ServiceResult<Vec<u8>> {
        let response = self
            .send(AccountRequest::ExportAccount { address, password })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::ExportAccountResponse(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn accepted_tokens(&self, address: AccountAddress) -> ServiceResult<Vec<TokenCode>> {
        let response = self
            .send(AccountRequest::AccountAcceptedTokens { address })
            .await
            .map_err(|e| AccountServiceError::OtherError(Box::new(e)))??;
        if let AccountResponse::AcceptedTokens(data) = response {
            Ok(data)
        } else {
            panic!("Unexpect response type.")
        }
    }
}
