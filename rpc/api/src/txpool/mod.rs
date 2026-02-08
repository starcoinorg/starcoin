// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

pub type TxPoolClient = jsonrpsee::async_client::Client;
use crate::multi_types::MultiSignedUserTransactionView;
use crate::types::{SignedUserTransactionView, StrView};
use crate::FutureResult;
use jsonrpsee::{core::RegisterMethodError, Methods, RpcModule};
use starcoin_crypto::HashValue;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::{account_address::AccountAddress, transaction::SignedUserTransaction};
use starcoin_vm2_types::{
    account_address::AccountAddress as AccountAddress2,
    transaction::SignedUserTransaction as SignedUserTransaction2,
};
use std::sync::Arc;
pub trait TxPoolApi {
    fn submit_transaction(&self, tx: SignedUserTransaction) -> FutureResult<HashValue>;
    fn submit_transactions(&self, txs: Vec<SignedUserTransaction>) -> FutureResult<Vec<HashValue>>;
    fn submit_transaction2(&self, tx: SignedUserTransaction2) -> FutureResult<HashValue>;
    fn submit_hex_transaction(&self, tx: String) -> FutureResult<HashValue>;
    fn submit_hex_transaction2(&self, tx: String) -> FutureResult<HashValue>;

    /// return current gas price
    fn gas_price(&self) -> FutureResult<StrView<u64>>;
    /// get all pending txns in txpool of given sender.
    /// no matter the state of txn is ready or in future.
    fn pending_txns(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<SignedUserTransactionView>>;
    fn pending_txns_multi(
        &self,
        addr: AccountAddress,
        max_len: Option<u32>,
    ) -> FutureResult<Vec<MultiSignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    fn pending_txn(&self, txn_hash: HashValue) -> FutureResult<Option<SignedUserTransactionView>>;

    /// get pending txn in txpool by its hash value
    fn pending_txn_multi(
        &self,
        txn_hash: HashValue,
    ) -> FutureResult<Option<MultiSignedUserTransactionView>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    fn next_sequence_number(&self, address: AccountAddress) -> FutureResult<Option<u64>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    fn next_sequence_number_in_batch(
        &self,
        addresses: Vec<AccountAddress>,
    ) -> FutureResult<Option<Vec<(AccountAddress, Option<u64>)>>>;

    /// or `None` if there are no pending transactions from that sender in txpool.
    fn state(&self) -> FutureResult<TxPoolStatus>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender in txpool.
    fn next_sequence_number2(&self, address: AccountAddress2) -> FutureResult<Option<u64>>;
}

/// Build jsonrpsee methods from legacy `TxPoolApi`.
pub fn txpool_methods<T>(api: T) -> std::result::Result<Methods, RegisterMethodError>
where
    T: TxPoolApi + Send + Sync + 'static,
{
    let mut module = RpcModule::new(Arc::new(api));

    module.register_async_method("txpool.submit_transaction", |params, api, _| async move {
        let tx: SignedUserTransaction = params.one()?;
        api.submit_transaction(tx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.submit_transactions", |params, api, _| async move {
        let txs: Vec<SignedUserTransaction> = params.one()?;
        api.submit_transactions(txs)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.submit_transaction2", |params, api, _| async move {
        let tx: SignedUserTransaction2 = params.one()?;
        api.submit_transaction2(tx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.submit_hex_transaction", |params, api, _| async move {
        let tx: String = params.one()?;
        api.submit_hex_transaction(tx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.submit_hex_transaction2", |params, api, _| async move {
        let tx: String = params.one()?;
        api.submit_hex_transaction2(tx)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.gas_price", |_, api, _| async move {
        api.gas_price().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.pending_txns_of_sender", |params, api, _| async move {
        let (addr, max_len): (AccountAddress, Option<u32>) = params.parse()?;
        api.pending_txns(addr, max_len)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method(
        "txpool.pending_txns_of_sender_multi",
        |params, api, _| async move {
            let (addr, max_len): (AccountAddress, Option<u32>) = params.parse()?;
            api.pending_txns_multi(addr, max_len)
                .await
                .map_err(crate::map_jsonrpc_err)
        },
    )?;

    module.register_async_method("txpool.pending_txn", |params, api, _| async move {
        let txn_hash: HashValue = params.one()?;
        api.pending_txn(txn_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.pending_txn_multi", |params, api, _| async move {
        let txn_hash: HashValue = params.one()?;
        api.pending_txn_multi(txn_hash)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.next_sequence_number", |params, api, _| async move {
        let address: AccountAddress = params.one()?;
        api.next_sequence_number(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method(
        "txpool.next_sequence_number_in_batch",
        |params, api, _| async move {
            let addresses: Vec<AccountAddress> = params.one()?;
            api.next_sequence_number_in_batch(addresses)
                .await
                .map_err(crate::map_jsonrpc_err)
        },
    )?;

    module.register_async_method("txpool.state", |_, api, _| async move {
        api.state().await.map_err(crate::map_jsonrpc_err)
    })?;

    module.register_async_method("txpool.next_sequence_number2", |params, api, _| async move {
        let address: AccountAddress2 = params.one()?;
        api.next_sequence_number2(address)
            .await
            .map_err(crate::map_jsonrpc_err)
    })?;

    Ok(module.into())
}
