// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

//! Local Transactions List.

use std::{fmt, sync::Arc};

use crate::pool::{ScoredTransaction, VerifiedTransaction as Transaction};
use crypto::hash::HashValue;
use linked_hash_map::LinkedHashMap;
use tx_pool::{self, VerifiedTransaction};

/// Status of local transaction.
/// Can indicate that the transaction is currently part of the queue (`Pending/Future`)
/// or gives a reason why the transaction was removed.
#[derive(Debug, PartialEq, Clone)]
pub enum Status {
    /// The transaction is currently in the transaction queue.
    Pending(Arc<Transaction>),
    /// Transaction is already mined.
    Mined(Arc<Transaction>),
    /// Transaction didn't get into any block, but some other tx with the same nonce got.
    Culled(Arc<Transaction>),
    /// Transaction is dropped because of limit
    Dropped(Arc<Transaction>),
    /// Replaced because of higher gas price of another transaction.
    Replaced {
        /// Replaced transaction
        old: Arc<Transaction>,
        /// Transaction that replaced this one.
        new: Arc<Transaction>,
    },
    /// Transaction was never accepted to the queue.
    /// It means that it was too cheap to replace any transaction already in the pool.
    Rejected(Arc<Transaction>, String),
    /// Transaction is invalid.
    Invalid(Arc<Transaction>),
    /// Transaction was canceled.
    Canceled(Arc<Transaction>),
}

impl Status {
    fn is_pending(&self) -> bool {
        matches!(self, Status::Pending(_))
    }
}

/// Keeps track of local transactions that are in the queue or were mined/dropped recently.
pub struct LocalTransactionsList {
    max_old: usize,
    transactions: LinkedHashMap<HashValue, Status>,
    pending: usize,
    in_chain: Option<Box<dyn Fn(&HashValue) -> bool + Send + Sync>>,
}

impl fmt::Debug for LocalTransactionsList {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("LocalTransactionsList")
            .field("max_old", &self.max_old)
            .field("transactions", &self.transactions)
            .field("pending", &self.pending)
            .field("in_chain", &self.in_chain.is_some())
            .finish()
    }
}

impl Default for LocalTransactionsList {
    fn default() -> Self {
        Self::new(10)
    }
}

impl LocalTransactionsList {
    /// Create a new list of local transactions.
    pub fn new(max_old: usize) -> Self {
        LocalTransactionsList {
            max_old,
            transactions: Default::default(),
            pending: 0,
            in_chain: None,
        }
    }

    /// Set blockchain checker.
    ///
    /// The function should return true if transaction is included in chain.
    pub fn set_in_chain_checker<F, T>(&mut self, checker: T)
    where
        T: Into<Option<F>>,
        F: Fn(&HashValue) -> bool + Send + Sync + 'static,
    {
        self.in_chain = checker.into().map(|f| Box::new(f) as _);
    }

    /// Returns true if the transaction is already in local transactions.
    pub fn contains(&self, hash: &HashValue) -> bool {
        self.transactions.contains_key(hash)
    }

    /// Return a map of all currently stored transactions.
    pub fn all_transactions(&self) -> &LinkedHashMap<HashValue, Status> {
        &self.transactions
    }

    /// Returns true if there are pending local transactions.
    pub fn has_pending(&self) -> bool {
        self.pending > 0
    }

    fn clear_old(&mut self) {
        let number_of_old = self.transactions.len() - self.pending;
        if self.max_old >= number_of_old {
            return;
        }

        let to_remove: Vec<_> = self
            .transactions
            .iter()
            .filter(|&(_, status)| !status.is_pending())
            .map(|(hash, _)| *hash)
            .take(number_of_old - self.max_old)
            .collect();

        for hash in to_remove {
            self.transactions.remove(&hash);
        }
    }

    fn insert(&mut self, hash: HashValue, status: Status) {
        let result = self.transactions.insert(hash, status);
        if let Some(old) = result {
            if old.is_pending() {
                self.pending -= 1;
            }
        }
    }
}

impl tx_pool::Listener<Transaction> for LocalTransactionsList {
    fn added(&mut self, tx: &Arc<Transaction>, old: Option<&Arc<Transaction>>) {
        if !tx.priority().is_local() {
            return;
        }

        debug!(target: "own_tx", "Imported to the pool (hash {:?})", tx.hash());
        self.clear_old();
        self.insert(*tx.hash(), Status::Pending(tx.clone()));
        self.pending += 1;

        if let Some(old) = old {
            if self.transactions.contains_key(old.hash()) {
                self.insert(
                    *old.hash(),
                    Status::Replaced {
                        old: old.clone(),
                        new: tx.clone(),
                    },
                );
            }
        }
    }

    fn rejected<H: fmt::Debug + fmt::LowerHex>(
        &mut self,
        tx: &Arc<Transaction>,
        reason: &tx_pool::Error<H>,
    ) {
        if !tx.priority().is_local() {
            return;
        }

        debug!(target: "own_tx", "Transaction rejected (hash {:?}). {}", tx.hash(), reason);
        self.insert(
            *tx.hash(),
            Status::Rejected(tx.clone(), format!("{}", reason)),
        );
        self.clear_old();
    }

    fn dropped(&mut self, tx: &Arc<Transaction>, new: Option<&Transaction>) {
        if !tx.priority().is_local() {
            return;
        }

        match new {
            Some(new) => {
                warn!(target: "own_tx", "Transaction pushed out because of limit (hash {:?}, replacement: {:?})", tx.hash(), new.hash())
            }
            None => {
                warn!(target: "own_tx", "Transaction dropped because of limit (hash: {:?})", tx.hash())
            }
        }
        self.insert(*tx.hash(), Status::Dropped(tx.clone()));
        self.clear_old();
    }

    fn invalid(&mut self, tx: &Arc<Transaction>) {
        if !tx.priority().is_local() {
            return;
        }

        warn!(target: "own_tx", "Transaction marked invalid (hash {:?})", tx.hash());
        self.insert(*tx.hash(), Status::Invalid(tx.clone()));
        self.clear_old();
    }

    fn canceled(&mut self, tx: &Arc<Transaction>) {
        if !tx.priority().is_local() {
            return;
        }

        warn!(target: "own_tx", "Transaction canceled (hash {:?})", tx.hash());
        self.insert(*tx.hash(), Status::Canceled(tx.clone()));
        self.clear_old();
    }

    fn culled(&mut self, tx: &Arc<Transaction>) {
        if !tx.priority().is_local() {
            return;
        }

        let is_in_chain = self
            .in_chain
            .as_ref()
            .map(|checker| checker(tx.hash()))
            .unwrap_or(false);
        if is_in_chain {
            info!(target: "own_tx", "Transaction mined (hash {:?})", tx.hash());
            self.insert(*tx.hash(), Status::Mined(tx.clone()));
            return;
        }
        info!(target: "own_tx", "Transaction culled (hash {:?})", tx.hash());
        self.insert(*tx.hash(), Status::Culled(tx.clone()));
    }
}
