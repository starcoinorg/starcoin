// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Transaction Queue

use super::{
    client, listener, local_transactions::LocalTransactionsList, ready, replace, scoring, verifier,
    PendingOrdering, PendingSettings, PrioritizationStrategy, SeqNumber, TxStatus,
};
use crate::pool::ready::Expiration;
use crate::{pool, pool::PoolTransaction};
use crypto::hash::HashValue;
use futures_channel::mpsc;
use parking_lot::RwLock;
use starcoin_txpool_api::TxPoolStatus;
use std::{
    cmp,
    collections::{BTreeMap, HashMap},
    fmt,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};
use tx_pool::{self, Verifier};
use types::{account_address::AccountAddress as Address, transaction};

type Listener = (
    LocalTransactionsList,
    (
        listener::TransactionsPoolNotifier,
        (listener::Logger, listener::StatusLogger),
    ),
);
type Pool = tx_pool::Pool<pool::VerifiedTransaction, scoring::SeqNumberAndGasPrice, Listener>;

/// Max cache time in milliseconds for pending transactions.
///
/// Pending transactions are cached and will only be computed again
/// if last cache has been created earler than `TIMESTAMP_CACHE` ms ago.
/// This timeout applies only if there are local pending transactions
/// since it only affects transaction Condition.
const TIMESTAMP_CACHE: u64 = 1000;

/// How many senders at once do we attempt to process while culling.
///
/// When running with huge transaction pools, culling can take significant amount of time.
/// To prevent holding `write()` lock on the pool for this long period, we split the work into
/// chunks and allow other threads to utilize the pool in the meantime.
/// This parameter controls how many (best) senders at once will be processed.
const CULL_SENDERS_CHUNK: usize = 1024;

/// Transaction queue status.
#[derive(Debug, Clone, PartialEq)]
pub struct Status {
    /// Verifier options.
    pub options: verifier::Options,
    /// Current status of the transaction pool.
    pub status: tx_pool::LightStatus,
    /// Current limits of the transaction pool.
    pub limits: tx_pool::Options,
}

impl Status {
    /// helper func to check pool status is full or not.
    /// should keep sync with Pool::is_full
    pub fn is_full(&self) -> bool {
        self.status.transaction_count >= self.limits.max_count
            || self.status.mem_usage >= self.limits.max_mem_usage
    }
}

impl fmt::Display for Status {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            fmt,
            "Pool: {current}/{max} ({senders} senders; {mem}/{mem_max} kB)]",
            current = self.status.transaction_count,
            max = self.limits.max_count,
            senders = self.status.senders,
            mem = self.status.mem_usage / 1024,
            mem_max = self.limits.max_mem_usage / 1024,
        )
    }
}

#[allow(clippy::from_over_into)]
impl Into<TxPoolStatus> for Status {
    fn into(self) -> TxPoolStatus {
        TxPoolStatus {
            txn_count: self.status.transaction_count,
            txn_max_count: self.limits.max_count,
            mem: self.status.mem_usage / 1024,
            mem_max: self.limits.max_mem_usage / 1024,
            senders: self.status.senders,
            is_full: self.is_full(),
        }
    }
}

#[derive(Debug)]
struct CachedPending {
    block_number: u64,
    current_timestamp: u64,
    nonce_cap: Option<SeqNumber>,
    has_local_pending: bool,
    pending: Option<Vec<Arc<pool::VerifiedTransaction>>>,
    max_len: usize,
}

impl CachedPending {
    /// Creates new `CachedPending` without cached set.
    pub fn none() -> Self {
        CachedPending {
            block_number: 0,
            current_timestamp: 0,
            has_local_pending: false,
            pending: None,
            nonce_cap: None,
            max_len: 0,
        }
    }

    /// Remove cached pending set.
    pub fn clear(&mut self) {
        self.pending = None;
    }

    /// Returns cached pending set (if any) if it's valid.
    pub fn pending(
        &self,
        block_number: u64,
        current_timestamp: u64,
        nonce_cap: Option<&SeqNumber>,
        max_len: usize,
    ) -> Option<Vec<Arc<pool::VerifiedTransaction>>> {
        // First check if we have anything in cache.
        let pending = self.pending.as_ref()?;

        if block_number != self.block_number {
            return None;
        }

        // In case we don't have any local pending transactions
        // there is no need to invalidate the cache because of timestamp.
        // Timestamp only affects local `PendingTransactions` with `Condition::Timestamp`.
        if self.has_local_pending && current_timestamp > self.current_timestamp + TIMESTAMP_CACHE {
            return None;
        }

        // It's fine to return limited set even if `nonce_cap` is `None`.
        // The worst thing that may happen is that some transactions won't get propagated in current round,
        // but they are not really valid in current block anyway. We will propagate them in the next round.
        // Also there is no way to have both `Some` with different numbers since it depends on the block number
        // and a constant parameter in schedule (`nonce_cap_increment`)
        if self.nonce_cap.is_none() && nonce_cap.is_some() {
            return None;
        }

        // It's fine to just take a smaller subset, but not other way around.
        if max_len > self.max_len {
            return None;
        }

        Some(pending.iter().take(max_len).cloned().collect())
    }
}

#[derive(Debug)]
struct RecentlyRejected {
    inner: RwLock<HashMap<HashValue, transaction::TransactionError>>,
    limit: usize,
}

impl RecentlyRejected {
    fn new(limit: usize) -> Self {
        RecentlyRejected {
            limit,
            inner: RwLock::new(HashMap::with_capacity(MIN_REJECTED_CACHE_SIZE)),
        }
    }

    fn clear(&self) {
        self.inner.write().clear();
    }

    fn get(&self, hash: &HashValue) -> Option<transaction::TransactionError> {
        self.inner.read().get(hash).cloned()
    }

    fn insert(&self, hash: HashValue, err: &transaction::TransactionError) {
        if self.inner.read().contains_key(&hash) {
            return;
        }

        let mut inner = self.inner.write();
        inner.insert(hash, err.clone());

        // clean up
        if inner.len() > self.limit {
            // randomly remove half of the entries
            let to_remove: Vec<_> = inner.keys().take(self.limit / 2).cloned().collect();
            for key in to_remove {
                inner.remove(&key);
            }
        }
    }
}

/// Minimal size of rejection cache, by default it's equal to queue size.
const MIN_REJECTED_CACHE_SIZE: usize = 2048;

/// Ethereum Transaction Queue
///
/// Responsible for:
/// - verifying incoming transactions
/// - maintaining a pool of verified transactions.
/// - returning an iterator for transactions that are ready to be included in block (pending)
#[derive(Debug)]
pub struct TransactionQueue {
    insertion_id: Arc<AtomicUsize>,
    pool: RwLock<Pool>,
    options: RwLock<verifier::Options>,
    cached_pending: RwLock<CachedPending>,
    recently_rejected: RecentlyRejected,
}

impl TransactionQueue {
    /// Create new queue with given pool limits and initial verification options.
    pub fn new(
        limits: tx_pool::Options,
        verification_options: verifier::Options,
        strategy: PrioritizationStrategy,
    ) -> Self {
        let max_count = limits.max_count;
        TransactionQueue {
            insertion_id: Default::default(),
            pool: RwLock::new(tx_pool::Pool::new(
                Default::default(),
                scoring::SeqNumberAndGasPrice(strategy),
                limits,
            )),
            options: RwLock::new(verification_options),
            cached_pending: RwLock::new(CachedPending::none()),
            recently_rejected: RecentlyRejected::new(cmp::max(
                MIN_REJECTED_CACHE_SIZE,
                max_count / 4,
            )),
        }
    }

    /// Update verification options
    ///
    /// Some parameters of verification may vary in time (like block gas limit or minimal gas price).
    pub fn set_verifier_options(&self, options: verifier::Options) {
        *self.options.write() = options;
    }

    /// Sets the in-chain transaction checker for pool listener.
    pub fn set_in_chain_checker<F>(&self, f: F)
    where
        F: Fn(&HashValue) -> bool + Send + Sync + 'static,
    {
        self.pool.write().listener_mut().0.set_in_chain_checker(f)
    }

    /// Import a set of transactions to the pool.
    ///
    /// Given blockchain and state access (Client)
    /// verifies and imports transactions to the pool.
    pub fn import<T, C>(
        &self,
        client: C,
        transactions: T,
    ) -> Vec<Result<(), transaction::TransactionError>>
    where
        T: IntoIterator<Item = PoolTransaction>,
        C: client::AccountSeqNumberClient + client::Client,
    {
        // Run verification
        trace_time!("pool::verify_and_import");
        let options = self.options.read().clone();

        let transaction_to_replace = {
            if options.no_early_reject {
                None
            } else {
                let pool = self.pool.write();
                if pool.is_full() {
                    pool.worst_transaction()
                        .map(|worst| (pool.scoring().clone(), worst))
                } else {
                    None
                }
            }
        };

        let verifier = verifier::Verifier::new(
            client.clone(),
            options,
            self.insertion_id.clone(),
            transaction_to_replace,
        );

        let replace =
            replace::ReplaceByScoreAndReadiness::new(self.pool.read().scoring().clone(), client);

        let mut results = Vec::new();
        for transaction in transactions.into_iter() {
            let hash = transaction.hash();

            if self.pool.read().find(&hash).is_some() {
                results.push(Err(transaction::TransactionError::AlreadyImported));
            }

            if let Some(err) = self.recently_rejected.get(&hash) {
                trace!(target: "txqueue", "[{:?}] Rejecting recently rejected: {:?}", &hash, err);
                results.push(Err(err));
            }

            let imported = verifier
                .verify_transaction(transaction)
                .and_then(|verified| {
                    self.pool
                        .write()
                        .import(verified, &replace)
                        .map_err(convert_error)
                });

            results.push(match imported {
                Ok(_) => Ok(()),
                Err(err) => {
                    self.recently_rejected.insert(hash, &err);
                    Err(err)
                }
            });
        }

        // Notify about imported transactions.
        (self.pool.write().listener_mut().1).0.notify();

        if results.iter().any(|r| r.is_ok()) {
            self.cached_pending.write().clear();
        }
        results
    }

    pub fn txns_of_sender(
        &self,
        sender: &Address,
        max_len: usize,
    ) -> Vec<Arc<pool::VerifiedTransaction>> {
        // always ready
        let ready = Expiration::new(0);
        self.pool
            .read()
            .pending_from_sender(ready, sender)
            .take(max_len)
            .collect()
    }

    /// Returns current pending transactions ordered by priority.
    ///
    /// NOTE: This may return a cached version of pending transaction set.
    /// Re-computing the pending set is possible with `#collect_pending` method,
    /// but be aware that it's a pretty expensive operation.
    pub fn pending<C>(
        &self,
        client: C,
        settings: PendingSettings,
    ) -> Vec<Arc<pool::VerifiedTransaction>>
    where
        C: client::AccountSeqNumberClient,
    {
        let PendingSettings {
            block_number,
            current_timestamp,
            max_len,
            ordering,
        } = settings;

        let ready = Self::ready(client, block_number, current_timestamp);

        match ordering {
            // In case we don't have a cached set, but we don't care about order
            // just return the unordered set.
            PendingOrdering::Unordered => self
                .pool
                .read()
                .unordered_pending(ready)
                .take(max_len)
                .collect(),
            PendingOrdering::Priority => self.pool.read().pending(ready).take(max_len).collect(),
        }
    }

    fn ready<C>(
        client: C,
        block_number: u64,
        current_timestamp: u64,
    ) -> ((ready::Expiration, ready::Condition), ready::State<C>)
    where
        C: client::AccountSeqNumberClient,
    {
        let pending_readiness = ready::Condition::new(block_number, current_timestamp);
        // don't mark any transactions as stale at this point.
        let state_readiness = ready::State::new(client, None);

        (
            (ready::Expiration::new(current_timestamp), pending_readiness),
            state_readiness,
        )
    }

    /// Culls all stalled transactions from the pool.
    pub fn cull<C: client::AccountSeqNumberClient>(&self, client: C, now: u64) {
        trace_time!("pool::cull");
        // We want to clear stale transactions from the queue as well.
        // (Transactions that are occuping the queue for a long time without being included)
        let stale_id = {
            let current_id = self.insertion_id.load(atomic::Ordering::Relaxed);
            // wait at least for half of the queue to be replaced
            let gap = self.pool.read().options().max_count / 2;
            // but never less than 100 transactions
            let gap = cmp::max(100, gap);

            current_id.checked_sub(gap)
        };

        self.recently_rejected.clear();

        let mut removed = 0;
        let senders: Vec<_> = {
            let pool = &self.pool.read();
            pool.senders().cloned().collect()
        };
        for chunk in senders.chunks(CULL_SENDERS_CHUNK) {
            trace_time!("pool::cull::chunk");
            let state_readiness = ready::State::new(client.clone(), stale_id);
            // also remove expired txns.
            let readiness = (ready::Expiration::new(now), state_readiness);
            removed += self.pool.write().cull(Some(chunk), readiness);
        }
        debug!(target: "txqueue", "Removed {} stalled transactions. {}", removed, self.status());
    }

    /// Returns next valid sequence number for given sender
    /// or `None` if there are no pending transactions from that sender.
    pub fn next_sequence_number<C: client::AccountSeqNumberClient>(
        &self,
        client: C,
        address: &Address,
    ) -> Option<SeqNumber> {
        // Also we ignore stale transactions in the queue.
        let stale_id = None;

        let state_readiness = ready::State::new(client, stale_id);

        self.pool
            .read()
            .pending_from_sender(state_readiness, address)
            .last()
            .map(|tx| tx.signed().sequence_number().saturating_add(1))
    }

    /// Retrieve a transaction from the pool.
    ///
    /// Given transaction hash looks up that transaction in the pool
    /// and returns a shared pointer to it or `None` if it's not present.
    pub fn find(&self, hash: &HashValue) -> Option<Arc<pool::VerifiedTransaction>> {
        self.pool.read().find(hash)
    }

    /// Remove a set of transactions from the pool.
    ///
    /// Given an iterator of transaction hashes
    /// removes them from the pool.
    /// That method should be used if invalid transactions are detected
    /// or you want to cancel a transaction.
    pub fn remove<'a, T: IntoIterator<Item = &'a HashValue>>(
        &self,
        hashes: T,
        is_invalid: bool,
    ) -> Vec<Option<Arc<pool::VerifiedTransaction>>> {
        let results = {
            let mut removed = vec![];
            let pool = &mut self.pool.write();
            for hash in hashes.into_iter() {
                removed.push(pool.remove(hash, is_invalid));
            }
            removed
        };

        if results.iter().any(Option::is_some) {
            self.cached_pending.write().clear();
        }

        results
    }

    /// Clear the entire pool.
    pub fn clear(&self) {
        self.pool.write().clear();
    }

    /// Penalize given senders.
    pub fn penalize<'a, T: IntoIterator<Item = &'a Address>>(&self, senders: T) {
        for sender in senders {
            self.pool.write().update_scores(sender, ());
        }
    }

    pub(crate) fn inner_status<C>(&self, client: C, block_number: u64, current_timestamp: u64)
    where
        C: client::AccountSeqNumberClient,
    {
        let ready = Self::ready(client, block_number, current_timestamp);
        let status = self.pool.read().status(ready);
        debug!("txpool queue inner status: {:?}", status);
    }

    /// Returns a status of the queue.
    pub fn status(&self) -> Status {
        let pool = &self.pool.read();
        let status = pool.light_status();
        let limits = pool.options();
        let options = self.options.read().clone();

        Status {
            options,
            status,
            limits,
        }
    }

    /// Check if there are any local transactions in the pool.
    ///
    /// Returns `true` if there are any transactions in the pool
    /// that has been marked as local.
    ///
    /// Local transactions are the ones from accounts managed by this node
    /// and transactions submitted via local RPC (`eth_sendRawTransaction`)
    pub fn has_local_pending_transactions(&self) -> bool {
        self.pool.read().listener().0.has_pending()
    }

    /// Returns status of recently seen local transactions.
    pub fn local_transactions(&self) -> BTreeMap<HashValue, pool::local_transactions::Status> {
        self.pool
            .read()
            .listener()
            .0
            .all_transactions()
            .iter()
            .map(|(a, b)| (*a, b.clone()))
            .collect()
    }

    /// Add a listener to be notified about all transactions the pool
    pub fn add_pending_listener(&self, f: mpsc::UnboundedSender<Arc<[HashValue]>>) {
        (self.pool.write().listener_mut().1)
            .0
            .add_pending_listener(f);
    }

    /// Add a listener to be notified about all transactions the pool
    pub fn add_full_listener(&self, f: mpsc::UnboundedSender<Arc<[(HashValue, TxStatus)]>>) {
        (self.pool.write().listener_mut().1).0.add_full_listener(f);
    }

    /// Check if pending set is cached.
    #[cfg(test)]
    pub fn is_pending_cached(&self) -> bool {
        self.cached_pending.read().pending.is_some()
    }
}

fn convert_error<H: fmt::Debug + fmt::LowerHex>(
    err: tx_pool::Error<H>,
) -> transaction::TransactionError {
    use tx_pool::Error;

    match err {
        Error::AlreadyImported(..) => transaction::TransactionError::AlreadyImported,
        Error::TooCheapToEnter(..) => transaction::TransactionError::LimitReached,
        Error::TooCheapToReplace(..) => transaction::TransactionError::TooCheapToReplace {
            prev: None,
            new: None,
        },
    }
}
