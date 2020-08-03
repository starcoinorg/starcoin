// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_derive::rpc;

pub use self::gen_client::Client as WalletClient;
use crate::FutureResult;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_wallet_api::WalletAccount;

#[rpc]
pub trait WalletApi {
    /// Get default account
    #[rpc(name = "wallet.default")]
    fn default(&self) -> FutureResult<Option<WalletAccount>>;
    #[rpc(name = "wallet.create")]
    fn create(&self, password: String) -> FutureResult<WalletAccount>;
    #[rpc(name = "wallet.list")]
    fn list(&self) -> FutureResult<Vec<WalletAccount>>;
    #[rpc(name = "wallet.get")]
    fn get(&self, address: AccountAddress) -> FutureResult<Option<WalletAccount>>;
    #[rpc(name = "wallet.sign_txn")]
    fn sign_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> FutureResult<SignedUserTransaction>;
    #[rpc(name = "wallet.unlock")]
    fn unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> FutureResult<()>;

    /// Import private key with address.
    #[rpc(name = "wallet.import")]
    fn import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> FutureResult<WalletAccount>;

    /// Return the private key as bytes for `address`
    #[rpc(name = "wallet.export")]
    fn export(&self, address: AccountAddress, password: String) -> FutureResult<Vec<u8>>;

    #[rpc(name = "wallet.accepted_tokens")]
    fn accepted_tokens(&self, address: AccountAddress) -> FutureResult<Vec<TokenCode>>;
}
