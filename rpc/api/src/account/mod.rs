// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type AccountClient = jsonrpsee::async_client::Client;
use crate::types::{SignedMessageView, StrView, TransactionRequest};
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use starcoin_account_api::AccountInfo;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::token::token_code::TokenCode;
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
    fn sign_txn_in_batch(
        &self,
        raw_txn: Vec<RawUserTransaction>,
    ) -> FutureResult<Vec<SignedUserTransaction>>;

    /// unlock account for duration in seconds, default to u32::max.
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<AccountInfo>;

    /// unlock accounts for duration in seconds, default to u32::max.
    fn unlock_in_batch(
        &self,
        batch: Vec<(AccountAddress, String)>,
        duration: Option<u32>,
    ) -> FutureResult<Vec<AccountInfo>>;
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

    module.register_async_method("account.default", |_, api, _| async move {
        api.default().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.set_default_account", |params, api, _| async move {
        let addr: AccountAddress = params.one()?;
        api.set_default_account(addr)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.create", |params, api, _| async move {
        let password: String = params.one()?;
        api.create(password).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.list", |_, api, _| async move {
        api.list().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.get", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.get(address).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.sign", |params, api, _| async move {
        let (address, data): (AccountAddress, SigningMessage) = params.parse()?;
        api.sign(address, data).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.sign_txn_request", |params, api, _| async move {
        let txn_request: TransactionRequest = params.one()?;
        api.sign_txn_request(txn_request)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.sign_txn", |params, api, _| async move {
        let (raw_txn, signer): (RawUserTransaction, AccountAddress) = params.parse()?;
        api.sign_txn(raw_txn, signer)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.sign_txn_in_batch", |params, api, _| async move {
        let raw_txn: Vec<RawUserTransaction> = params.one()?;
        api.sign_txn_in_batch(raw_txn)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.unlock", |params, api, _| async move {
        let (address, password, duration): (AccountAddress, String, Option<u32>) = params.parse()?;
        api.unlock(address, password, duration)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.unlock_in_batch", |params, api, _| async move {
        let (batch, duration): (Vec<(AccountAddress, String)>, Option<u32>) = params.parse()?;
        api.unlock_in_batch(batch, duration)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.lock", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.lock(address).await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.import", |params, api, _| async move {
        let (address, private_key, password): (AccountAddress, StrView<Vec<u8>>, String) =
            params.parse()?;
        api.import(address, private_key, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.import_readonly", |params, api, _| async move {
        let (address, public_key): (AccountAddress, StrView<Vec<u8>>) = params.parse()?;
        api.import_readonly(address, public_key)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.export", |params, api, _| async move {
        let (address, password): (AccountAddress, String) = params.parse()?;
        api.export(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.change_password", |params, api, _| async move {
        let (address, new_password): (AccountAddress, String) = params.parse()?;
        api.change_account_password(address, new_password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.accepted_tokens", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.accepted_tokens(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("account.remove", |params, api, _| async move {
        let (address, password): (AccountAddress, Option<String>) = params.parse()?;
        api.remove(address, password)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}

