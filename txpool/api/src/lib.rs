// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use starcoin_crypto::hash::HashValue;
use starcoin_types::{transaction, transaction::SignedUserTransaction};
use std::sync::Arc;

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

    /// subscribe
    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, transaction::TxStatus)>>>>;

    /// commit block
    async fn chain_new_blocks(
        self,
        enacted: Vec<HashValue>,
        retracted: Vec<HashValue>,
    ) -> Result<()>;

    /// rollback
    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()>;
}
