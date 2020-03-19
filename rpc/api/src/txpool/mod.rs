// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_types::transaction::SignedUserTransaction;

pub use self::gen_client::Client as TxPoolClient;

#[rpc]
pub trait TxPoolApi {
    #[rpc(name = "submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> FutureResult<bool>;
}
