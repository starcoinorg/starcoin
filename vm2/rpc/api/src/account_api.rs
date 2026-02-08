// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type AccountClient = jsonrpsee::async_client::Client;
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    sign_message::SigningMessage,
    transaction::{RawUserTransaction, SignedUserTransaction},
    view::{SignedMessageView, StrView, TransactionRequest},
};
use starcoin_vm2_vm_types::token::token_code::TokenCode;
use std::sync::Arc;

pub trait AccountApi {
    /// Get default account
    fn default(&self) -> FutureResult<Option<AccountInfo>>;
    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<AccountInfo>;
    fn create(&self, password: String) -> FutureResult<AccountInfo>;
    fn list(&self) -> FutureResult<Vec<AccountInfo>>;
    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>>;
    fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> FutureResult<SignedMessageView>;

    /// sign a txn request, return hex encoded bcs_ext bytes of signed user txn.
    fn sign_txn_request(&self, txn_request: TransactionRequest) -> FutureResult<String>;
    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction>;

    /// unlock account for duration in seconds, default to u32::max.
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<AccountInfo>;
    fn lock(&self, address: AccountAddress) -> FutureResult<AccountInfo>;

    /// Import private key with address.
    fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> FutureResult<AccountInfo>;

    /// Import a readonly account with public key.
    fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> FutureResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>>;
    /// change account password, user need to unlock account first.
    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> FutureResult<AccountInfo>;

    //TODO remove this api
    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>>;

    /// remove account from local wallet.
    fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> FutureResult<AccountInfo>;
}

/// Build jsonrpsee methods from legacy `AccountApi`.
pub fn account_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: AccountApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("account2.default", |_, api, _| async move {
        api.default().await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.set_default_account", |params, api, _| async move {
        let addr: AccountAddress = params.one()?;
        api.set_default_account(addr)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.create", |params, api, _| async move {
        let password: String = params.one()?;
        api.create(password).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.list", |_, api, _| async move {
        api.list().await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.get", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.get(address).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.sign", |params, api, _| async move {
        let (address, data): (AccountAddress, SigningMessage) = params.parse()?;
        api.sign(address, data).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.sign_txn_request", |params, api, _| async move {
        let txn_request: TransactionRequest = params.one()?;
        api.sign_txn_request(txn_request)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.sign_txn", |params, api, _| async move {
        let (raw_txn, signer): (RawUserTransaction, AccountAddress) = params.parse()?;
        api.sign_txn(raw_txn, signer)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.unlock", |params, api, _| async move {
        let (address, password, duration): (AccountAddress, String, Option<u32>) = params.parse()?;
        api.unlock(address, password, duration)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.lock", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.lock(address).await.map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.import", |params, api, _| async move {
        let (address, private_key, password): (AccountAddress, StrView<Vec<u8>>, String) =
            params.parse()?;
        api.import(address, private_key, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.import_readonly", |params, api, _| async move {
        let (address, public_key): (AccountAddress, StrView<Vec<u8>>) = params.parse()?;
        api.import_readonly(address, public_key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.export", |params, api, _| async move {
        let (address, password): (AccountAddress, String) = params.parse()?;
        api.export(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.change_password", |params, api, _| async move {
        let (address, new_password): (AccountAddress, String) = params.parse()?;
        api.change_account_password(address, new_password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.accepted_tokens", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.accepted_tokens(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;
    module.register_async_method("account2.remove", |params, api, _| async move {
        let (address, password): (AccountAddress, Option<String>) = params.parse()?;
        api.remove(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
