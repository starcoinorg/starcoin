// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub use self::gen_client::Client as AccountClient;
use crate::FutureResult;
use openrpc_derive::openrpc;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::sign_message::SigningMessage;
use starcoin_vm2_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm2_types::view::{SignedMessageView, StrView, TransactionRequest};
use starcoin_vm2_vm_types::token::token_code::TokenCode;

#[openrpc]
pub trait AccountApi {
    /// Get default account
    #[rpc(name = "account2.default")]
    fn default(&self) -> FutureResult<Option<AccountInfo>>;
    #[rpc(name = "account2.set_default_account")]
    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<AccountInfo>;
    #[rpc(name = "account2.create")]
    fn create(&self, password: String) -> FutureResult<AccountInfo>;
    #[rpc(name = "account2.list")]
    fn list(&self) -> FutureResult<Vec<AccountInfo>>;
    #[rpc(name = "account2.get")]
    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>>;
    #[rpc(name = "account2.sign")]
    fn sign(
        &self,
        address: AccountAddress,
        data: SigningMessage,
    ) -> FutureResult<SignedMessageView>;

    /// sign a txn request, return hex encoded bcs_ext bytes of signed user txn.
    #[rpc(name = "account2.sign_txn_request")]
    fn sign_txn_request(&self, txn_request: TransactionRequest) -> FutureResult<String>;

    #[rpc(name = "account2.sign_txn")]
    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction>;

    /// unlock account for duration in seconds, default to u32::max.
    #[rpc(name = "account2.unlock")]
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> FutureResult<AccountInfo>;

    #[rpc(name = "account2.lock")]
    fn lock(&self, address: AccountAddress) -> FutureResult<AccountInfo>;

    /// Import private key with address.
    #[rpc(name = "account2.import")]
    fn import(
        &self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> FutureResult<AccountInfo>;

    /// Import a readonly account with public key.
    #[rpc(name = "account2.import_readonly")]
    fn import_readonly(
        &self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> FutureResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    #[rpc(name = "account2.export")]
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>>;

    #[rpc(name = "account2.change_password")]
    /// change account password, user need to unlock account first.
    fn change_account_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> FutureResult<AccountInfo>;

    //TODO remove this api
    #[rpc(name = "account2.accepted_tokens")]
    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>>;

    /// remove account from local wallet.
    #[rpc(name = "account2.remove")]
    fn remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> FutureResult<AccountInfo>;
}

#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
