// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures_channel::mpsc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::HashValue;
use starcoin_types::multi_transaction::{
    MultiAccountAddress, MultiSignedUserTransaction, MultiTransactionError,
};
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader},
    transaction,
};
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
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
    ) -> Vec<Result<(), transaction::TransactionError>> {
        let local_peer_id = Some("local-rpc".to_string());
        let multi_txns = txns.into_iter().map(|txn| txn.into()).collect();
        let rets = self.add_txns_multi_signed(multi_txns, false, local_peer_id);
        let mut results = vec![];
        for ret in rets {
            match ret {
                Ok(()) => results.push(Ok(())),
                Err(err) => match err {
                    MultiTransactionError::VM1(err) => results.push(Err(err)),
                    _ => panic!("should be vm1 type"),
                },
            }
        }
        results
    }

    fn add_txns_multi_signed(
        &self,
        txns: Vec<MultiSignedUserTransaction>,
        bypass_vm1_limit: bool,
        peer_id: Option<String>,
    ) -> Vec<Result<(), MultiTransactionError>>;

    /// Removes transaction from the pool.
    ///
    /// Attempts to "cancel" a transaction. If it was not propagated yet (or not accepted by other peers)
    /// there is a good chance that the transaction will actually be removed.
    fn remove_txn(
        &self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Option<MultiSignedUserTransaction>;

    /// Get all pending txns which is ok to be packaged to mining.
    /// `now` is the current timestamp in secs, if it's None, it default to real world's current timestamp.
    /// It's an Option to make mock time easier.
    fn get_pending_txns(
        &self,
        max_len: Option<u64>,
        now: Option<u64>,
    ) -> Vec<MultiSignedUserTransaction>;

    /// Get all pending txns which is ok to be packaged to mining with a specific header state.
    fn get_pending_with_header(
        &self,
        max_len: u64,
        current_timestamp_secs: Option<u64>,
        header: &BlockHeader,
    ) -> Vec<MultiSignedUserTransaction>;

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number(&self, address: AccountAddress) -> Option<u64>;

    /// Returns next valid sequence number for given sender with a specific header state.
    fn next_sequence_number_with_header(
        &self,
        address: AccountAddress,
        header: &BlockHeader,
    ) -> Option<u64>;

    /// subscribe
    fn subscribe_txns(&self) -> mpsc::UnboundedReceiver<TxnStatusFullEvent>;

    fn subscribe_pending_txn(&self) -> mpsc::UnboundedReceiver<Arc<[HashValue]>>;

    /// notify txpool about chain new blocks
    /// `enacted` is the blocks which enter the main chain.
    /// `retracted` is the blocks which belongs to previous main chain.
    fn chain_new_block(&self, enacted: Vec<Block>, retracted: Vec<Block>) -> Result<()>;

    /// Tx Pool status
    fn status(&self) -> TxPoolStatus;

    fn find_txn(&self, hash: &HashValue) -> Option<MultiSignedUserTransaction>;
    fn txns_of_sender(
        &self,
        sender: &MultiAccountAddress,
        max_len: Option<usize>,
    ) -> Vec<MultiSignedUserTransaction>;

    /// Returns next valid sequence number for given sender (vm2 AccountAddress)
    /// or `None` if there are no pending transactions from that sender.
    fn next_sequence_number2(&self, address: AccountAddress2) -> Option<u64>;

    /// Returns next valid sequence number for given sender (vm2 AccountAddress) with a specific header state.
    fn next_sequence_number2_with_header(
        &self,
        address: AccountAddress2,
        header: &BlockHeader,
    ) -> Option<u64>;
}

#[derive(Clone, Debug)]
pub struct PropagateTransactions {
    txns: Vec<MultiSignedUserTransaction>,
}

impl PropagateTransactions {
    pub fn new(txns: Vec<MultiSignedUserTransaction>) -> Self {
        Self { txns }
    }

    pub fn transaction_to_propagate(&self) -> Vec<MultiSignedUserTransaction> {
        self.txns.clone()
    }
}
