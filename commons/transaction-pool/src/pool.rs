// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use log::{trace, warn};
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use crate::{
    error,
    listener::{Listener, NoopListener},
    options::Options,
    ready::{Readiness, Ready},
    replace::{ReplaceTransaction, ShouldReplace},
    scoring::{self, ScoreWithRef, Scoring},
    status::{LightStatus, Status},
    transactions::{AddResult, Transactions},
    VerifiedTransaction,
};

/// Internal representation of transaction.
///
/// Includes unique insertion id that can be used for scoring explicitly,
/// but internally is used to resolve conflicts in case of equal scoring
/// (newer transactions are preferred).
#[derive(Debug)]
pub struct Transaction<T> {
    /// Sequential id of the transaction
    pub insertion_id: u64,
    /// Shared transaction
    pub transaction: Arc<T>,
}

impl<T> Clone for Transaction<T> {
    fn clone(&self) -> Self {
        Transaction {
            insertion_id: self.insertion_id,
            transaction: self.transaction.clone(),
        }
    }
}

impl<T> ::std::ops::Deref for Transaction<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.transaction
    }
}

/// A transaction pool.
#[derive(Debug)]
pub struct Pool<T: VerifiedTransaction, S: Scoring<T>, L = NoopListener> {
    listener: L,
    scoring: S,
    options: Options,
    mem_usage: usize,

    transactions: HashMap<T::Sender, Transactions<T, S>>,
    by_hash: HashMap<T::Hash, Transaction<T>>,

    best_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,
    worst_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,

    insertion_id: u64,
}

impl<T: VerifiedTransaction, S: Scoring<T> + Default> Default for Pool<T, S> {
    fn default() -> Self {
        Self::with_scoring(S::default(), Options::default())
    }
}

impl<T: VerifiedTransaction, S: Scoring<T> + Default> Pool<T, S> {
    /// Creates a new `Pool` with given options
    /// and default `Scoring` and `Listener`.
    pub fn with_options(options: Options) -> Self {
        Self::with_scoring(S::default(), options)
    }
}

impl<T: VerifiedTransaction, S: Scoring<T>> Pool<T, S> {
    /// Creates a new `Pool` with given `Scoring` and options.
    pub fn with_scoring(scoring: S, options: Options) -> Self {
        Self::new(NoopListener, scoring, options)
    }
}

const INITIAL_NUMBER_OF_SENDERS: usize = 16;

impl<T, S, L> Pool<T, S, L>
where
    T: VerifiedTransaction + Sync,
    S: Scoring<T>,
    L: Listener<T>,
{
    /// Creates new `Pool` with given `Scoring`, `Listener` and options.
    pub fn new(listener: L, scoring: S, options: Options) -> Self {
        let transactions = HashMap::with_capacity(INITIAL_NUMBER_OF_SENDERS);
        let by_hash = HashMap::with_capacity(options.max_count / 16);

        Pool {
            listener,
            scoring,
            options,
            mem_usage: 0,
            transactions,
            by_hash,
            best_transactions: Default::default(),
            worst_transactions: Default::default(),
            insertion_id: 0,
        }
    }

    /// Attempts to import new transaction to the pool, returns a `Arc<T>` or an `Error`.
    ///
    /// NOTE: Since `Ready`ness is separate from the pool it's possible to import stalled transactions.
    /// It's the caller responsibility to make sure that's not the case.
    ///
    /// NOTE: The transaction may push out some other transactions from the pool
    /// either because of limits (see `Options`) or because `Scoring` decides that the transaction
    /// replaces an existing transaction from that sender.
    ///
    /// If any limit is reached the transaction with the lowest `Score` will be compared with the
    /// new transaction via the supplied `ShouldReplace` implementation and may be evicted.
    ///
    /// The `Listener` will be informed on any drops or rejections.
    pub async fn import<R>(&mut self, transaction: T, replace: &R) -> error::Result<Arc<T>, T::Hash>
    where
        R: ShouldReplace<T>,
    {
        let mem_usage = transaction.mem_usage();

        if self.by_hash.contains_key(transaction.hash()) {
            return Err(error::Error::AlreadyImported(transaction.hash().clone()));
        }

        self.insertion_id += 1;
        let transaction = Transaction {
            insertion_id: self.insertion_id,
            transaction: Arc::new(transaction),
        };

        // TODO [ToDr] Most likely move this after the transaction is inserted.
        // Avoid using should_replace, but rather use scoring for that.
        {
            while self.by_hash.len() + 1 > self.options.max_count {
                trace!(
                    "Count limit reached: {} > {}",
                    self.by_hash.len() + 1,
                    self.options.max_count
                );
                if !self.remove_worst(&transaction, replace).await?.is_some() {
                    break;
                }
            }

            while self.mem_usage + mem_usage > self.options.max_mem_usage {
                trace!(
                    "Mem limit reached: {} > {}",
                    self.mem_usage + mem_usage,
                    self.options.max_mem_usage
                );
                if !self.remove_worst(&transaction, replace).await?.is_some() {
                    break;
                }
            }
        }

        let (result, prev_state, current_state) = {
            let transactions = self
                .transactions
                .entry(transaction.sender().clone())
                .or_insert_with(Transactions::default);
            // get worst and best transactions for comparison
            let prev = transactions.worst_and_best();
            let result = transactions.add(transaction, &self.scoring, self.options.max_per_sender);
            let current = transactions.worst_and_best();
            (result, prev, current)
        };

        // update best and worst transactions from this sender (if required)
        self.update_senders_worst_and_best(prev_state, current_state);

        match result {
            AddResult::Ok(tx) => {
                self.listener.added(&tx, None);
                self.finalize_insert(&tx, None);
                Ok(tx.transaction)
            }
            AddResult::PushedOut { new, old } | AddResult::Replaced { new, old } => {
                self.listener.added(&new, Some(&old));
                self.finalize_insert(&new, Some(&old));
                Ok(new.transaction)
            }
            AddResult::TooCheap { new, old } => {
                let error = error::Error::TooCheapToReplace(old.hash().clone(), new.hash().clone());
                self.listener.rejected(&new, &error);
                return Err(error);
            }
            AddResult::TooCheapToEnter(new, score) => {
                let error =
                    error::Error::TooCheapToEnter(new.hash().clone(), format!("{:#x}", score));
                self.listener.rejected(&new, &error);
                return Err(error);
            }
        }
    }

    /// Updates state of the pool statistics if the transaction was added to a set.
    fn finalize_insert(&mut self, new: &Transaction<T>, old: Option<&Transaction<T>>) {
        self.mem_usage += new.mem_usage();
        self.by_hash.insert(new.hash().clone(), new.clone());

        if let Some(old) = old {
            self.finalize_remove(old.hash());
        }
    }

    /// Updates the pool statistics if transaction was removed.
    fn finalize_remove(&mut self, hash: &T::Hash) -> Option<Arc<T>> {
        self.by_hash.remove(hash).map(|old| {
            self.mem_usage -= old.transaction.mem_usage();
            old.transaction
        })
    }

    /// Updates best and worst transactions from a sender.
    fn update_senders_worst_and_best(
        &mut self,
        previous: Option<((S::Score, Transaction<T>), (S::Score, Transaction<T>))>,
        current: Option<((S::Score, Transaction<T>), (S::Score, Transaction<T>))>,
    ) {
        let worst_collection = &mut self.worst_transactions;
        let best_collection = &mut self.best_transactions;

        let is_same = |a: &(S::Score, Transaction<T>), b: &(S::Score, Transaction<T>)| {
            a.0 == b.0 && a.1.hash() == b.1.hash()
        };

        let update = |collection: &mut BTreeSet<_>, (score, tx), remove| {
            if remove {
                collection.remove(&ScoreWithRef::new(score, tx));
            } else {
                collection.insert(ScoreWithRef::new(score, tx));
            }
        };

        match (previous, current) {
            (None, Some((worst, best))) => {
                update(worst_collection, worst, false);
                update(best_collection, best, false);
            }
            (Some((worst, best)), None) => {
                // all transactions from that sender has been removed.
                // We can clear a hashmap entry.
                self.transactions.remove(worst.1.sender());
                update(worst_collection, worst, true);
                update(best_collection, best, true);
            }
            (Some((w1, b1)), Some((w2, b2))) => {
                if !is_same(&w1, &w2) {
                    update(worst_collection, w1, true);
                    update(worst_collection, w2, false);
                }
                if !is_same(&b1, &b2) {
                    update(best_collection, b1, true);
                    update(best_collection, b2, false);
                }
            }
            (None, None) => {}
        }
    }

    /// Attempts to remove the worst transaction from the pool if it's worse than the given one.
    ///
    /// Returns `None` in case we couldn't decide if the transaction should replace the worst transaction or not.
    /// In such case we will accept the transaction even though it is going to exceed the limit.
    async fn remove_worst<R>(
        &mut self,
        transaction: &Transaction<T>,
        replace: &R,
    ) -> error::Result<Option<Transaction<T>>, T::Hash>
    where
        R: ShouldReplace<T>,
    {
        let to_remove = match self.worst_transactions.iter().next_back() {
            // No elements to remove? and the pool is still full?
            None => {
                warn!("The pool is full but there are no transactions to remove.");
                return Err(error::Error::TooCheapToEnter(
                    transaction.hash().clone(),
                    "unknown".into(),
                ));
            }
            Some(old) => {
                let txs = &self.transactions;
                let get_replace_tx = |tx| {
                    let sender_txs = txs
                        .get(transaction.sender())
                        .map(|txs| txs.iter().as_slice());
                    ReplaceTransaction::new(tx, sender_txs)
                };
                let old_replace = get_replace_tx(&old.transaction);
                let new_replace = get_replace_tx(transaction);

                match replace.should_replace(old_replace, new_replace).await {
                    // We can't decide which of them should be removed, so accept both.
                    scoring::Choice::InsertNew => None,
                    // New transaction is better than the worst one so we can replace it.
                    scoring::Choice::ReplaceOld => Some(old.clone()),
                    // otherwise fail
                    scoring::Choice::RejectNew => {
                        let err = error::Error::TooCheapToEnter(
                            transaction.hash().clone(),
                            format!("{:#x}", old.score),
                        );
                        self.listener.rejected(transaction, &err);
                        return Err(err);
                    }
                }
            }
        };

        if let Some(to_remove) = to_remove {
            // Remove from transaction set
            self.remove_from_set(to_remove.transaction.sender(), |set, scoring| {
                set.remove(&to_remove.transaction, scoring)
            });
            self.listener
                .dropped(&to_remove.transaction, Some(transaction));
            self.finalize_remove(to_remove.transaction.hash());
            Ok(Some(to_remove.transaction))
        } else {
            Ok(None)
        }
    }

    fn remove_from_set<R, F: FnOnce(&mut Transactions<T, S>, &S) -> R>(
        &mut self,
        sender: &T::Sender,
        f: F,
    ) -> Option<R> {
        let (prev, next, result) = if let Some(set) = self.transactions.get_mut(sender) {
            let prev = set.worst_and_best();
            let result = f(set, &self.scoring);
            (prev, set.worst_and_best(), result)
        } else {
            return None;
        };

        self.update_senders_worst_and_best(prev, next);
        Some(result)
    }

    /// Clears pool from all transactions.
    /// This causes a listener notification that all transactions were dropped.
    /// NOTE: the drop-notification order will be arbitrary.
    pub fn clear(&mut self) {
        self.mem_usage = 0;
        self.transactions.clear();
        self.best_transactions.clear();
        self.worst_transactions.clear();

        for (_hash, tx) in self.by_hash.drain() {
            self.listener.dropped(&tx.transaction, None)
        }
    }

    /// Removes single transaction from the pool.
    /// Depending on the `is_invalid` flag the listener
    /// will either get a `cancelled` or `invalid` notification.
    pub fn remove(&mut self, hash: &T::Hash, is_invalid: bool) -> Option<Arc<T>> {
        if let Some(tx) = self.finalize_remove(hash) {
            self.remove_from_set(tx.sender(), |set, scoring| set.remove(&tx, scoring));
            if is_invalid {
                self.listener.invalid(&tx);
            } else {
                self.listener.canceled(&tx);
            }
            Some(tx)
        } else {
            None
        }
    }

    /// Removes all stalled transactions from given sender.
    async fn remove_stalled<R: Ready<T>>(&mut self, sender: &T::Sender, ready: &mut R) -> usize {
        let (prev, current, removed) = if let Some(set) = self.transactions.get_mut(sender) {
            let prev = set.worst_and_best();
            let result = set.cull(ready, &self.scoring).await;

            (prev, set.worst_and_best(), result)
        } else {
            return 0;
        };

        self.update_senders_worst_and_best(prev, current);

        let len = removed.len();
        for tx in removed {
            self.finalize_remove(tx.hash());
            self.listener.culled(&tx);
        }
        len
    }

    /// Removes all stalled transactions from given sender list (or from all senders).
    pub async fn cull<R: Ready<T>>(
        &mut self,
        senders: Option<&[T::Sender]>,
        mut ready: R,
    ) -> usize {
        let mut removed = 0;
        match senders {
            Some(senders) => {
                for sender in senders {
                    removed += self.remove_stalled(sender, &mut ready).await;
                }
            }
            None => {
                let senders = self.transactions.keys().cloned().collect::<Vec<_>>();
                for sender in senders {
                    removed += self.remove_stalled(&sender, &mut ready).await;
                }
            }
        }

        removed
    }

    /// Returns a transaction if it's part of the pool or `None` otherwise.
    pub fn find(&self, hash: &T::Hash) -> Option<Arc<T>> {
        self.by_hash.get(hash).map(|t| t.transaction.clone())
    }

    /// Returns worst transaction in the queue (if any).
    pub fn worst_transaction(&self) -> Option<Arc<T>> {
        self.worst_transactions
            .iter()
            .next_back()
            .map(|x| x.transaction.transaction.clone())
    }

    /// Returns true if the pool is at it's capacity.
    pub fn is_full(&self) -> bool {
        self.by_hash.len() >= self.options.max_count || self.mem_usage >= self.options.max_mem_usage
    }

    /// Returns senders ordered by priority of their transactions.
    pub fn senders(&self) -> impl Iterator<Item = &T::Sender> {
        self.best_transactions
            .iter()
            .map(|tx| tx.transaction.sender())
    }

    /// Returns an iterator of pending (ready) transactions.
    /// NOTE: the transactions are not removed from the queue.
    pub async fn pending<R: Ready<T>>(&self, ready: R, max_len: usize) -> Vec<Arc<T>> {
        let best_transactions = self.best_transactions.clone();
        self.ordered_pending(ready, best_transactions, max_len)
            .await
    }

    /// Returns pending (ready) transactions from given sender.
    pub async fn pending_from_sender<R: Ready<T>>(
        &self,
        ready: R,
        sender: &T::Sender,
        max_len: usize,
    ) -> Vec<Arc<T>> {
        let best_txn = match self
            .transactions
            .get(sender)
            .and_then(|transactions| transactions.worst_and_best())
        {
            None => return vec![],
            Some((_, best)) => {
                let s = ScoreWithRef::new(best.0, best.1);
                s
            }
        };
        let best_transactions = {
            let mut set = BTreeSet::new();
            set.insert(best_txn);
            set
        };
        self.ordered_pending(ready, best_transactions, max_len)
            .await
    }

    async fn ordered_pending<R: Ready<T>>(
        &self,
        mut ready: R,
        mut best_transactions: BTreeSet<ScoreWithRef<T, S::Score>>,
        max_len: usize,
    ) -> Vec<Arc<T>> {
        let mut result = vec![];

        while !best_transactions.is_empty() && result.len() < max_len {
            let best = {
                let best = best_transactions
                    .iter()
                    .next()
                    .expect("current_best is not empty; qed")
                    .clone();
                best_transactions
                    .take(&best)
                    .expect("Just taken from iterator; qed")
            };

            let tx_state = ready.is_ready(&best.transaction).await;
            // Add the next best sender's transaction when applicable
            match tx_state {
                Readiness::Ready | Readiness::Stale => {
                    // retrieve next one from the same sender.
                    let next = self
                        .transactions
                        .get(best.transaction.sender())
                        .and_then(|s| s.find_next(&best.transaction, &self.scoring));
                    if let Some((score, tx)) = next {
                        best_transactions.insert(ScoreWithRef::new(score, tx));
                    }
                }
                _ => (),
            }

            if tx_state == Readiness::Ready {
                result.push(best.transaction.transaction);
            } else {
                trace!(
                    "[{:?}] Ignoring {:?} transaction.",
                    best.transaction.hash(),
                    tx_state
                );
            }
        }

        result
    }

    /// Returns unprioritized list of ready transactions.
    /// NOTE: Current implementation will iterate over all transactions from particular sender
    /// ordered by nonce, but that might change in the future.
    ///
    /// NOTE: the transactions are not removed from the queue.
    /// You might remove them later by calling `cull`.
    pub async fn unordered_pending<R: Ready<T>>(
        &self,
        mut ready: R,
        max_len: usize,
    ) -> Vec<Arc<T>> {
        let mut result = vec![];
        'outer: for (_sender, transactions) in self.transactions.iter() {
            for tx in transactions.iter() {
                match ready.is_ready(tx).await {
                    Readiness::Ready => {
                        result.push(tx.transaction.clone());
                        if result.len() >= max_len {
                            break 'outer;
                        }
                    }
                    state => trace!("[{:?}] Ignoring {:?} transaction.", tx.hash(), state),
                }
            }
        }
        result
    }

    /// Update score of transactions of a particular sender.
    pub fn update_scores(&mut self, sender: &T::Sender, event: S::Event) {
        let res = if let Some(set) = self.transactions.get_mut(sender) {
            let prev = set.worst_and_best();
            set.update_scores(&self.scoring, event);
            let current = set.worst_and_best();
            Some((prev, current))
        } else {
            None
        };

        if let Some((prev, current)) = res {
            self.update_senders_worst_and_best(prev, current);
        }
    }

    /// Computes the full status of the pool (including readiness).
    pub async fn status<R: Ready<T>>(&self, mut ready: R) -> Status {
        let mut status = Status::default();

        for (_sender, transactions) in &self.transactions {
            let len = transactions.len();
            for (idx, tx) in transactions.iter().enumerate() {
                match ready.is_ready(tx).await {
                    Readiness::Stale => status.stalled += 1,
                    Readiness::Ready => status.pending += 1,
                    Readiness::Future => {
                        status.future += len - idx;
                        break;
                    }
                }
            }
        }

        status
    }

    /// Returns light status of the pool.
    pub fn light_status(&self) -> LightStatus {
        LightStatus {
            mem_usage: self.mem_usage,
            transaction_count: self.by_hash.len(),
            senders: self.transactions.len(),
        }
    }

    /// Returns current pool options.
    pub fn options(&self) -> Options {
        self.options.clone()
    }

    /// Borrows listener instance.
    pub fn listener(&self) -> &L {
        &self.listener
    }

    /// Borrows scoring instance.
    pub fn scoring(&self) -> &S {
        &self.scoring
    }

    /// Borrows listener mutably.
    pub fn listener_mut(&mut self) -> &mut L {
        &mut self.listener
    }
}
