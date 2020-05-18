// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use starcoin_crypto::hash::HashValue;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::{transaction, transaction::SignedUserTransaction};
use std::sync::Arc;

pub trait TxPoolSyncService: Clone + Send + Sync {
    fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>>;

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(self, txn_hash: HashValue, is_invalid: bool) -> Option<SignedUserTransaction>;

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(self, max_len: Option<u64>) -> Vec<SignedUserTransaction>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(self, address: AccountAddress) -> Option<u64>;

    /// subscribe
    fn subscribe_txns(
        self,
    ) -> mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>>;

    /// rollback
    fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait TxPoolAsyncService: Clone + std::marker::Unpin + Send + Sync {
    /// TODO: should be deprecated, use add_txns instead.
    async fn add(self, txn: SignedUserTransaction) -> Result<bool>;

    /// Add all the `txns` into txn pool
    async fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>>;

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    async fn remove_txn(
        self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Result<Option<SignedUserTransaction>>;

    /// Get all pending txns which is ok to be packaged to mining.
    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    async fn next_sequence_number(self, address: AccountAddress) -> Result<Option<u64>>;

    /// subscribe
    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>>>;

    /// rollback
    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()>;
}
