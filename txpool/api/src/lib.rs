// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::HashValue;
use starcoin_types::{
    account_address::AccountAddress, block::Block, transaction, transaction::SignedUserTransaction,
};
use std::fmt::Debug;
use std::sync::Arc;
pub type TxnStatusFullEvent = Arc<[(HashValue, transaction::TxStatus)]>;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TxPoolStatus {
    pub txn_count: usize,
    pub txn_max_count: usize,
    pub mem: usize,
    pub mem_max: usize,
    pub senders: usize,
    pub is_full: bool,
}

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
    /// `now` is the current timestamp in secs, if it's None, it default to real world's current timestamp.
    /// It's an Option to make mock time easier.
    fn get_pending_txns(
        &self,
        max_len: Option<u64>,
        now: Option<u64>,
    ) -> Vec<SignedUserTransaction>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64>;

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent>;

    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>>;

    /// notify txpool about chain new blocks
    /// `enacted` is the blocks which enter the main chain.
    /// `retracted` is the blocks which belongs to previous main chain.
    fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()>;

    /// Tx Pool status
    fn status(&self) -> TxPoolStatus;

    fn find_txn(&self, hash: &HashValue) -> Option<SignedUserTransaction>;
    fn txns_of_sender(
        &self,
        sender: &AccountAddress,
        max_len: Option<usize>,
    ) -> Vec<SignedUserTransaction>;
}

#[derive(Clone, Debug)]
pub struct PropagateTransactions {
    txns: Vec<SignedUserTransaction>,
}

impl PropagateTransactions {
    pub fn new(txns: Vec<SignedUserTransaction>) -> Self {
        Self { txns }
    }

    pub fn transaction_to_propagate(&self) -> Vec<SignedUserTransaction> {
        self.txns.clone()
    }
}
