// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use jsonrpc_derive::rpc;
use starcoin_types::transaction::SignedUserTransaction;

pub use self::gen_client::Client as TxPoolClient;
use crate::types::{SignedUserTransactionView, StrView};
use starcoin_crypto::HashValue;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::account_address::AccountAddress;

#[rpc(client, server, schema)]
pub trait TxPoolApi {
    #[rpc(name = "txpool.submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> FutureResult<HashValue>;

    #[rpc(name = "txpool.submit_hex_transaction")]
    fn submit_hex_transaction(&self, tx: String) -> FutureResult<HashValue>;

    /// return current gas price
    #[rpc(name = "txpool.gas_price")]
    fn gas_price(&self) -> FutureResult<StrView<u64>>;
    /// get all pending txns in txpool of given sender.
    /// no matter the state of txn is ready or in future.
    #[rpc(name = "txpool.pending_txns_of_sender")]
    fn pending_txns(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<SignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    #[rpc(name = "txpool.pending_txn")]
    fn pending_txn(&self, txn_hash: HashValue) -> FutureResult<Option<SignedUserTransactionView>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.next_sequence_number")]
    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>>;

    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.state")]
    fn state(&self) -> FutureResult<TxPoolStatus>;
}
#[test]
fn test() {
    let schema = rpc_impl_TxPoolApi::gen_client::Client::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
