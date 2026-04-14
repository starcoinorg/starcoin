// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::multi_types::MultiSignedUserTransactionView;
use crate::types::{SignedUserTransactionView, StrView};
use jsonrpsee::{
    core::{RegisterMethodError, RpcResult},
    proc_macros::rpc,
    Methods,
};
use starcoin_crypto::HashValue;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_vm2_types::{
    account_address::AccountAddress as AccountAddress2,
    transaction::SignedUserTransaction as SignedUserTransaction2,
};

use starcoin_rpc_schema_derive::rpc_schema;

#[rpc_schema]
#[rpc(client, server, namespace = "txpool", namespace_separator = ".")]
pub trait TxPoolApi {
    #[method(name = "submit_transaction")]
    async fn submit_transaction(&self, tx: SignedUserTransaction) -> RpcResult<HashValue>;

    #[method(name = "submit_transactions")]
    async fn submit_transactions(
        &self,
        txs: Vec<SignedUserTransaction>,
    ) -> RpcResult<Vec<HashValue>>;

    #[method(name = "submit_transaction2")]
    async fn submit_transaction2(&self, tx: SignedUserTransaction2) -> RpcResult<HashValue>;

    #[method(name = "submit_hex_transaction")]
    async fn submit_hex_transaction(&self, tx: String) -> RpcResult<HashValue>;

    #[method(name = "submit_hex_transaction2")]
    async fn submit_hex_transaction2(&self, tx: String) -> RpcResult<HashValue>;

    /// return current gas price
    #[method(name = "gas_price")]
    async fn gas_price(&self) -> RpcResult<StrView<u64>>;

    /// get all pending txns in txpool of given sender.
    /// no matter the state of txn is ready or in future.
    #[method(name = "pending_txns_of_sender")]
    async fn pending_txns(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> RpcResult<Vec<SignedUserTransactionView>>;

    #[method(name = "pending_txns_of_sender_multi")]
    async fn pending_txns_multi(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> RpcResult<Vec<MultiSignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    #[method(name = "pending_txn")]
    async fn pending_txn(
        &self,
        txn_hash: HashValue,
    ) -> RpcResult<Option<SignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    #[method(name = "pending_txn_multi")]
    async fn pending_txn_multi(
        &self,
        txn_hash: HashValue,
    ) -> RpcResult<Option<MultiSignedUserTransactionView>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[method(name = "next_sequence_number")]
    async fn next_sequence_number(&self, address: AccountAddress) -> RpcResult<Option<u64>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[method(name = "next_sequence_number_in_batch")]
    async fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> RpcResult<Option<Vec<(AccountAddress, Option<u64>)>>>;

    /// or `None` if there are no pending transactions from that sender in txpool.
    #[method(name = "state")]
    async fn state(&self) -> RpcResult<TxPoolStatus>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    #[method(name = "next_sequence_number2")]
    async fn next_sequence_number2(&self, address: AccountAddress2) -> RpcResult<Option<u64>>;
}

pub use TxPoolApiClient as TxPoolApiRpcClient;
pub use TxPoolApiServer as TxPoolApiRpcServer;

/// Build jsonrpsee methods from legacy `TxPoolApi`.
pub fn txpool_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: TxPoolApiServer + Send + Sync + 'static,
{
    Ok(TxPoolApiServer::into_rpc(api).into())
}
