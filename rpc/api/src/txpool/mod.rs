// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::FutureResult;
use openrpc_derive::openrpc;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;

pub use self::gen_client::Client as TxPoolClient;
use crate::multi_types::MultiSignedUserTransactionView;
use crate::types::{SignedUserTransactionView, StrView};
use starcoin_crypto::HashValue;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_vm2_types::{
    account_address::AccountAddress as AccountAddress2,
    transaction::SignedUserTransaction as SignedUserTransaction2,
};

#[openrpc]
pub trait TxPoolApi {
    #[rpc(name = "txpool.submit_transaction")]
    fn submit_transaction(&self, tx: SignedUserTransaction) -> FutureResult<HashValue>;

    #[rpc(name = "txpool.submit_transaction2")]
    fn submit_transaction2(&self, tx: SignedUserTransaction2) -> FutureResult<HashValue>;

    #[rpc(name = "txpool.submit_transaction_multi")]
    fn submit_transaction_multi(&self, tx: MultiSignedUserTransaction) -> FutureResult<HashValue>;

    #[rpc(name = "txpool.submit_hex_transaction")]
    fn submit_hex_transaction(&self, tx: String) -> FutureResult<HashValue>;

    #[rpc(name = "txpool.submit_hex_transaction2")]
    fn submit_hex_transaction2(&self, tx: String) -> FutureResult<HashValue>;

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

    #[rpc(name = "txpool.pending_txns_of_sender_multi")]
    fn pending_txns_multi(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<MultiSignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    #[rpc(name = "txpool.pending_txn")]
    fn pending_txn(&self, txn_hash: HashValue) -> FutureResult<Option<SignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    #[rpc(name = "txpool.pending_txn_multi")]
    fn pending_txn_multi(
        &self,
        txn_hash: HashValue,
    ) -> FutureResult<Option<MultiSignedUserTransactionView>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.next_sequence_number")]
    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>>;

    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.state")]
    fn state(&self) -> FutureResult<TxPoolStatus>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[rpc(name = "txpool.next_sequence_number2")]
    fn next_sequence_number2(&self, address: AccountAddress2) -> FutureResult<Option<u64>>;
}
#[test]
fn test() {
    let schema = self::gen_schema();
    let j = serde_json::to_string_pretty(&schema).unwrap();
    println!("{}", j);
}
