// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_derive::rpc;

pub use self::gen_client::Client as AccountClient;
use crate::FutureResult;
use starcoin_account_api::AccountInfo;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::token::token_code::TokenCode;

#[rpc]
pub trait AccountApi {
    /// Get default account
    #[rpc(name = "account.default")]
    fn default(&self) -> FutureResult<Option<AccountInfo>>;

    #[rpc(name = "account.set_default_account")]
    fn set_default_account(&self, addr: AccountAddress) -> FutureResult<Option<AccountInfo>>;

    #[rpc(name = "account.create")]
    fn create(&self, password: String) -> FutureResult<AccountInfo>;
    #[rpc(name = "account.list")]
    fn list(&self) -> FutureResult<Vec<AccountInfo>>;
    #[rpc(name = "account.get")]
    fn get(&self, address: AccountAddress) -> FutureResult<Option<AccountInfo>>;
    #[rpc(name = "account.sign_txn")]
    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction>;
    #[rpc(name = "account.unlock")]
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> FutureResult<()>;
    #[rpc(name = "account.lock")]
    fn lock(&self, address: AccountAddress) -> FutureResult<()>;

    /// Import private key with address.
    #[rpc(name = "account.import")]
    fn import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> FutureResult<AccountInfo>;

    /// Return the private key as bytes for `address`
    #[rpc(name = "account.export")]
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>>;

    #[rpc(name = "account.accepted_tokens")]
    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>>;
}
