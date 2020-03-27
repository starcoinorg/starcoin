// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_derive::rpc;

pub use self::gen_client::Client as AccountClient;
use crate::FutureResult;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;

#[rpc]
pub trait AccountApi {
    #[rpc(name = "account.create")]
    fn create(&self, password: String) -> FutureResult<WalletAccount>;
    #[rpc(name = "account.list")]
    fn list(&self) -> FutureResult<Vec<WalletAccount>>;
    #[rpc(name = "account.sign_txn")]
    fn sign_txn(&self, raw_txn: RawUserTransaction) -> FutureResult<SignedUserTransaction>;
}
