// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::types::{SignedMessageView, StrView, TransactionRequest};
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_account_api::AccountInfo;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::token::token_code::TokenCode;

use starcoin_rpc_schema_derive::rpc_schema;

#[rpc_schema]
#[rpc(client, server, namespace = "account", namespace_separator = ".")]
pub trait AccountApi {
    /// Get default account
    #[method(name = "default")]
    async fn default(&self) -> RpcResult<Option<AccountInfo>>;

    #[method(name = "set_default_account")]
    async fn set_default_account(&self, addr: AccountAddress) -> RpcResult<AccountInfo>;

    #[method(name = "create")]
    async fn create(&self, password: String) -> RpcResult<AccountInfo>;

    #[method(name = "list")]
    async fn list(&self) -> RpcResult<Vec<AccountInfo>>;

    #[method(name = "get")]
    async fn get(&self, address: AccountAddress) -> RpcResult<Option<AccountInfo>>;

    #[method(name = "sign")]
    async fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> RpcResult<SignedMessageView>;

    /// sign a txn request, return hex encoded bcs_ext bytes of signed user txn.
    #[method(name = "sign_txn_request")]
    async fn sign_txn_request(&self, txn_request: TransactionRequest) -> RpcResult<String>;

    #[method(name = "sign_txn")]
    async fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> RpcResult<SignedUserTransaction>;

    #[method(name = "sign_txn_in_batch")]
    async fn sign_txn_in_batch(
        &self,
        raw_txn: Vec<RawUserTransaction>,
    ) -> RpcResult<Vec<SignedUserTransaction>>;

    /// unlock account for duration in seconds, default to u32::max.
    #[method(name = "unlock")]
    async fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> RpcResult<AccountInfo>;

    /// unlock accounts for duration in seconds, default to u32::max.
    #[method(name = "unlock_in_batch")]
    async fn unlock_in_batch(
        &self,
        batch: Vec<(AccountAddress, String)>,
        duration: Option<u32>,
    ) -> RpcResult<Vec<AccountInfo>>;

    #[method(name = "lock")]
    async fn lock(&self, address: AccountAddress) -> RpcResult<AccountInfo>;

    /// Import private key with address.
    #[method(name = "import")]
    async fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> RpcResult<AccountInfo>;

    /// Import a readonly account with public key.
    #[method(name = "import_readonly")]
    async fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> RpcResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    #[method(name = "export")]
    async fn export(&self, address: AccountAddress, password: String) -> RpcResult<Vec<u8>>;

    /// change account password, user need to unlock account first.
    #[method(name = "change_password")]
    async fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> RpcResult<AccountInfo>;

    //TODO remove this api
    #[method(name = "accepted_tokens")]
    async fn accepted_tokens(&self, address: AccountAddress) -> RpcResult<Vec<TokenCode>>;

    /// remove account from local wallet.
    #[method(name = "remove")]
    async fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> RpcResult<AccountInfo>;
}

pub use AccountApiClient as AccountApiRpcClient;
pub use AccountApiServer as AccountApiRpcServer;

/// Build jsonrpsee methods from legacy `AccountApi`.
pub fn account_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: AccountApiServer + Send + Sync + 'static,
{
    Ok(AccountApiServer::into_rpc(api).into())
}
