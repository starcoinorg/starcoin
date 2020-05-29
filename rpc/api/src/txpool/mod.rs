// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_types::transaction::SignedUserTransaction;

pub use self::gen_client::Client as TxPoolClient;
use starcoin_types::account_address::AccountAddress;

#[rpc]
pub trait TxPoolApi {
    #[rpc(name = "txpool.submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> FutureResult<Result<(), String>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.next_sequence_number")]
    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>>;
}
