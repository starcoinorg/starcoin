// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use starcoin_crypto::hash::HashValue;
use starcoin_types::{
    account_address::AccountAddress, block::Block, transaction, transaction::SignedUserTransaction,
};
use std::sync::Arc;

pub type TxnStatusFullEvent = Arc<Vec<(HashValue, transaction::TxStatus)>>;

pub trait TxPoolSyncService: Clone + Send + Sync + Unpin {
    fn add_txns(
        &self,
        txns: Vec<SignedUserTransaction>,
    ) -> Vec<Result<(), transaction::TransactionError>>;

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(&self, txn_hash: HashValue, is_invalid: bool) -> Option<SignedUserTransaction>;

    /// Get all pending txns which is ok to be packaged to mining.
    fn get_pending_txns(&self, max_len: Option<u64>) -> Vec<SignedUserTransaction>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64>;

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent>;

    /// rollback
    fn rollback(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()>;
}
