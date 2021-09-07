// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_derive::rpc;

pub use self::gen_client::Client as AccountClient;
use crate::types::{SignedMessageView, StrView, TransactionRequest};
use crate::FutureResult;
use starcoin_account_api::AccountInfo;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::token::token_code::TokenCode;
#[rpc(client, server, schema)]
pub trait AccountApi {
    /// Get default account
    #[rpc(name = "account.default")]
    fn default(&self) -> FutureResult<Option<AccountInfo>>;
    #[rpc(name = "account.set_default_account")]
    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<AccountInfo>;
    #[rpc(name = "account.create")]
    fn create(&self, password: String) -> FutureResult<AccountInfo>;
    #[rpc(name = "account.list")]
    fn list(&self) -> FutureResult<Vec<AccountInfo>>;
    #[rpc(name = "account.get")]
    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>>;
    #[rpc(name = "account.sign")]
    fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> FutureResult<SignedMessageView>;

    /// sign a txn request, return hex encoded bcs_ext bytes of signed user txn.
    #[rpc(name = "account.sign_txn_request")]
    fn sign_txn_request(&self, txn_request: TransactionRequest) -> FutureResult<String>;

    #[rpc(name = "account.sign_txn")]
    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction>;

    /// unlock account for duration in seconds, default to u32::max.
    #[rpc(name = "account.unlock")]
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<AccountInfo>;

    #[rpc(name = "account.lock")]
    fn lock(&self, address: AccountAddress) -> FutureResult<AccountInfo>;

    /// Import private key with address.
    #[rpc(name = "account.import")]
    fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> FutureResult<AccountInfo>;

    /// Import a readonly account with public key.
    #[rpc(name = "account.import_readonly")]
    fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> FutureResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    #[rpc(name = "account.export")]
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>>;

    #[rpc(name = "account.change_password")]
    /// change account password, user need to unlock account first.
    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> FutureResult<AccountInfo>;

    //TODO remove this api
    #[rpc(name = "account.accepted_tokens")]
    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>>;

    /// remove account from local wallet.
    #[rpc(name = "account.remove")]
    fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> FutureResult<AccountInfo>;
}

#[test]
fn test() {
    let schema = rpc_impl_AccountApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
